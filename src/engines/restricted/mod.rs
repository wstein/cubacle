//! The 3x3x3 solved with a restricted alphabet of moves.
//!
//! The solution may use only the moves of an [`Alphabet`]: the half turns, the
//! generators of a subgroup such as `<R,U>`, quarter turns, or any list. The
//! engine searches the cube as 54 sticker colors, so it needs no model of
//! pieces and takes any move the cube has, slices and wide turns included.
//!
//! * `table`: the exact table of the alphabet, a breadth first search from the
//!   solved cube to some depth (the whole group, when it is small, as for the
//!   half turns); a state in it has its exact distance.
//! * `ida`: iterative deepening that uses the table as its bound and ends at
//!   any state the table knows, so the answer is shortest in the alphabet.
//! * `letters`: the moves as permutations and the relations between them that
//!   prune the search (derived from the alphabet, not assumed).
//! * `invariants`: the quick proof that a cube is out of reach (orbits,
//!   parity, twist and flip sums), so that it is reported at once.
//! * `enumerate`: listing the solutions of each length instead of one.
//! * `expand`: for big alphabets (the quarter turns), the default engine's
//!   answer written in the letters; not shortest, and said so.
//!
//! The default engine's coordinate tables are not used as a bound. They are
//! indexed by symmetry classes of the 18-move metric and bound the first phase
//! only; a table of the alphabet's own, searched from the solved cube, is
//! exact for any alphabet and is the bound here.

mod enumerate;
mod expand;
mod ida;
mod invariants;
mod letters;
mod table;

use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::puzzle::{
    Alphabet, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver,
};
use enumerate::Lister;
pub use enumerate::{EnumerateEnd, EnumerateRequest};
use ida::{Ended, Ida};
use invariants::{Invariants, Reach};
use letters::{Letters, State, N};
use table::Table;

/// Entries of the table when nothing else is asked for: about a second to
/// build, depth 9 for two generators.
pub const DEFAULT_TABLE_ENTRIES: usize = 2_000_000;

/// The longest solution searched for when the caller sets no limit.
const DEFAULT_MAX_MOVES: u8 = 40;

/// What a listing found and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// Solutions handed to the caller.
    pub found: usize,
    /// The length of the shortest solution (0 when the search was cancelled
    /// before it was known).
    pub shortest: usize,
    pub end: EnumerateEnd,
    /// Nodes searched, both for the shortest length and for the listing.
    pub nodes: u64,
}

/// What is built once per solver.
struct Built {
    letters: Letters,
    table: Table,
    invariants: Invariants,
    words: Option<Vec<Vec<usize>>>,
}

/// A 3x3x3 solver for one alphabet.
pub struct SubsetSolver {
    alphabet: Alphabet,
    helper: Option<Arc<dyn Solver + Send + Sync>>,
    table_entries: usize,
    built: OnceLock<Built>,
}

impl std::fmt::Debug for SubsetSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubsetSolver")
            .field("alphabet", &self.alphabet.label())
            .finish_non_exhaustive()
    }
}

fn state_of(facelets: &Facelets) -> State {
    let mut state = [0u8; N];
    for (slot, face) in state.iter_mut().zip(facelets.stickers()) {
        *slot = *face as u8;
    }
    state
}

impl SubsetSolver {
    /// A solver for `alphabet`, which must be restricted, non-empty and for
    /// the 3x3x3.
    pub fn new(alphabet: Alphabet) -> Result<SubsetSolver, String> {
        if alphabet.kind() != PuzzleKind::Cube3 {
            return Err("a move set is solved on the 3x3x3 here".to_owned());
        }
        if alphabet.is_unrestricted() || alphabet.is_empty() {
            return Err("the move set restricts nothing".to_owned());
        }
        Letters::new(&alphabet)?;
        Ok(SubsetSolver {
            alphabet,
            helper: None,
            table_entries: DEFAULT_TABLE_ENTRIES,
            built: OnceLock::new(),
        })
    }

    /// The solver whose answer is rewritten for big alphabets (the default
    /// 3x3x3 engine, handed in, never a global).
    pub fn with_helper(mut self, helper: Arc<dyn Solver + Send + Sync>) -> SubsetSolver {
        self.helper = Some(helper);
        self
    }

    /// The most states the table may hold, which bounds its memory (about 70
    /// bytes a state while it is built).
    pub fn with_table_entries(mut self, entries: usize) -> SubsetSolver {
        self.table_entries = entries.max(1000);
        self
    }

    pub fn alphabet(&self) -> &Alphabet {
        &self.alphabet
    }

    fn built(&self, deadline: Option<Instant>) -> &Built {
        self.built.get_or_init(|| {
            let letters = Letters::new(&self.alphabet).expect("checked in new");
            let solved = state_of(&Facelets::solved(PuzzleKind::Cube3));
            let table = Table::build(&letters, &solved, self.table_entries, deadline);
            let invariants = Invariants::new(&letters, &solved);
            let words = expand::words(&letters);
            Built {
                letters,
                table,
                invariants,
                words,
            }
        })
    }

    /// Lists the solutions of `facelets`: all of the shortest length, and with
    /// `request.longest` those of each longer length up to it, shortest first.
    /// `emit` gets each solution as it is found and says whether to go on;
    /// `cancel` is asked at every node. Only one order of commuting moves is
    /// listed, unless `request.raw`.
    pub fn enumerate(
        &self,
        facelets: &Facelets,
        request: &EnumerateRequest,
        options: &SolveOptions,
        emit: &mut dyn FnMut(&MoveSequence) -> bool,
        cancel: &dyn Fn() -> bool,
    ) -> Result<Listing, SolveResult> {
        let started = Instant::now();
        if facelets.kind() != PuzzleKind::Cube3 {
            return Err(Self::failed(
                Failure::InvalidInput,
                "the cube is not a 3x3x3".to_owned(),
                started,
            ));
        }
        let deadline = options.time_budget.map(|budget| started + budget);
        let built = self.built(deadline);
        let state = state_of(facelets);
        self.reach(built, &state, started)?;
        let limit = options.max_moves.unwrap_or(DEFAULT_MAX_MOVES);
        let mut search = Ida::new(&built.letters, &built.table, deadline).with_cancel(cancel);
        let shortest = match search.solve(&state, limit) {
            Ended::Found(path) => path.len(),
            Ended::NotInGroup => {
                return Err(Self::failed(
                    Failure::Exhausted,
                    format!(
                        "not solvable with {}: the cube is not in the group these moves make",
                        self.alphabet.label()
                    ),
                    started,
                ));
            }
            Ended::NoneWithin(depth) => {
                return Err(Self::failed(
                    Failure::Exhausted,
                    format!(
                        "no solution of at most {depth} moves in {}",
                        self.alphabet.label()
                    ),
                    started,
                ));
            }
            Ended::TimedOut if search.cancelled => {
                return Ok(Listing {
                    found: 0,
                    shortest: 0,
                    end: EnumerateEnd::Cancelled,
                    nodes: search.nodes,
                });
            }
            Ended::TimedOut => {
                return Err(Self::failed(
                    Failure::BudgetExhausted,
                    format!("the time budget ran out ({} nodes)", search.nodes),
                    started,
                ));
            }
        };
        let last = request
            .longest
            .map_or(shortest, |longest| usize::from(longest).max(shortest))
            .min(usize::from(limit).max(shortest));
        let mut lister = Lister::new(&built.letters, &built.table, deadline, cancel, request);
        let mut length = shortest;
        while length <= last && lister.ended.is_none() {
            lister.list(&state, length as u8, &mut |letters| {
                let moves: Vec<Move> = letters
                    .iter()
                    .map(|&i| built.letters.letters[i].mv)
                    .collect();
                emit(&MoveSequence::new(moves))
            });
            length += 1;
        }
        Ok(Listing {
            found: lister.found,
            shortest,
            end: lister.ended.unwrap_or(EnumerateEnd::Complete),
            nodes: search.nodes + lister.nodes,
        })
    }

    /// Whether the cube can be solved at all with the alphabet.
    fn reach(&self, built: &Built, state: &State, started: Instant) -> Result<(), SolveResult> {
        match built.invariants.check(state) {
            Ok(()) => Ok(()),
            Err(Reach::NoCube(why)) => Err(Self::failed(Failure::InvalidInput, why, started)),
            Err(Reach::Outside(why)) => Err(Self::failed(
                Failure::Exhausted,
                format!("not solvable with {}: {why}", self.alphabet.label()),
                started,
            )),
        }
    }

    fn failed(failure: Failure, message: String, started: Instant) -> SolveResult {
        SolveResult {
            outcome: Err(failure),
            message: Some(message),
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes: None,
            },
            stages: Vec::new(),
        }
    }

    fn answer(
        &self,
        built: &Built,
        facelets: &Facelets,
        path: &[usize],
        message: String,
        started: Instant,
    ) -> SolveResult {
        let moves: Vec<Move> = path.iter().map(|&i| built.letters.letters[i].mv).collect();
        let solution = MoveSequence::new(moves);
        let solved = facelets.apply(&solution).is_ok_and(|end| end.is_solved());
        if !solved {
            return Self::failed(
                Failure::Internal,
                "the answer does not solve the cube".to_owned(),
                started,
            );
        }
        SolveResult {
            outcome: Ok(solution),
            message: Some(message),
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes: None,
            },
            stages: Vec::new(),
        }
    }
}

impl Solver for SubsetSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn prepare(&self) {
        self.built(None);
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        if facelets.kind() != PuzzleKind::Cube3 {
            return Self::failed(
                Failure::InvalidInput,
                "the cube is not a 3x3x3".to_owned(),
                started,
            );
        }
        let deadline = options.time_budget.map(|budget| started + budget);
        let built = self.built(deadline);
        let state = state_of(facelets);
        if let Err(failed) = self.reach(built, &state, started) {
            return failed;
        }
        // A big alphabet: the default engine's answer, rewritten.
        if let (Some(words), Some(helper)) = (&built.words, &self.helper) {
            let inner = helper.solve(facelets, options);
            let solution = match &inner.outcome {
                Ok(solution) => solution.clone(),
                Err(_) => {
                    return SolveResult {
                        metrics: SolveMetrics {
                            elapsed: started.elapsed(),
                            ..inner.metrics
                        },
                        stages: Vec::new(),
                        ..inner
                    };
                }
            };
            if let Some(path) = expand::rewrite(&built.letters, words, &solution) {
                if options
                    .max_moves
                    .is_some_and(|limit| path.len() > usize::from(limit))
                {
                    return Self::failed(
                        Failure::Exhausted,
                        format!(
                            "the answer rewritten in {} has {} moves, over the limit",
                            self.alphabet.label(),
                            path.len()
                        ),
                        started,
                    );
                }
                let message = format!(
                    "{} moves in {}, rewritten from {} face turns; not proven shortest",
                    path.len(),
                    self.alphabet.label(),
                    solution.len()
                );
                return self.answer(built, facelets, &path, message, started);
            }
        }
        let limit = options.max_moves.unwrap_or(DEFAULT_MAX_MOVES);
        let mut search = Ida::new(&built.letters, &built.table, deadline);
        match search.solve(&state, limit) {
            Ended::Found(path) => {
                let message = format!(
                    "{} moves in {}, shortest in the set ({} nodes; table of {} states to depth {}{})",
                    path.len(),
                    self.alphabet.label(),
                    search.nodes,
                    built.table.len(),
                    built.table.depth(),
                    if built.table.is_complete() {
                        ", the whole group"
                    } else {
                        ""
                    }
                );
                self.answer(built, facelets, &path, message, started)
            }
            Ended::NotInGroup => Self::failed(
                Failure::Exhausted,
                format!(
                    "not solvable with {}: the cube is not in the group these moves make ({} states)",
                    self.alphabet.label(),
                    built.table.len()
                ),
                started,
            ),
            Ended::NoneWithin(depth) => Self::failed(
                Failure::Exhausted,
                format!(
                    "no solution of at most {depth} moves in {} (the cube may not be solvable with them)",
                    self.alphabet.label()
                ),
                started,
            ),
            Ended::TimedOut => Self::failed(
                Failure::BudgetExhausted,
                format!("the time budget ran out ({} nodes)", search.nodes),
                started,
            ),
        }
    }
}

#[cfg(test)]
mod list_tests;
#[cfg(test)]
mod tests;
