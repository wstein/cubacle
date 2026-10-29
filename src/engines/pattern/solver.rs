//! Solving a 4x4x4 or bigger cube with a restricted alphabet of layer turns.
//!
//! No table-based engine takes an arbitrary alphabet on these cubes, so the
//! cube is searched as a product of pattern macros made only of the alphabet's
//! moves (see [`PatternSearch`]). It solves the cubes that are such products
//! (the classic patterns, and what a few macros build); for any other cube the
//! answer says that nothing was found, which is not a proof that the alphabet
//! cannot solve it.

use std::sync::OnceLock;
use std::time::Instant;

use super::{Cap, PatternSearch, Probe};
use crate::puzzle::{
    Alphabet, Facelets, Failure, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};

/// A solver for the 4x4x4 to 7x7x7 that moves with an alphabet only.
pub struct PatternSolver {
    alphabet: Alphabet,
    cap: Cap,
    search: OnceLock<Result<PatternSearch, String>>,
}

impl std::fmt::Debug for PatternSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternSolver")
            .field("alphabet", &self.alphabet.label())
            .finish_non_exhaustive()
    }
}

impl PatternSolver {
    /// A solver for `alphabet`, a restricted alphabet for a cube of 4x4x4
    /// or more, searching within `cap`.
    pub fn new(alphabet: Alphabet, cap: Cap) -> Result<PatternSolver, String> {
        if alphabet.is_unrestricted() || alphabet.is_empty() {
            return Err("the move set restricts nothing".to_owned());
        }
        if alphabet.kind().size() < 4 {
            return Err("pattern macros are for the 4x4x4 and bigger cubes".to_owned());
        }
        Ok(PatternSolver {
            alphabet,
            cap,
            search: OnceLock::new(),
        })
    }

    fn search(&self) -> &Result<PatternSearch, String> {
        self.search.get_or_init(|| {
            let allow = |mv: &crate::puzzle::Move| self.alphabet.contains(mv);
            PatternSearch::new(self.alphabet.kind(), self.cap, Some(&allow)).map_err(|error| {
                format!(
                    "the moves {:?} cannot be searched: {error}",
                    self.alphabet.label()
                )
            })
        })
    }

    fn failed(failure: Failure, message: String, started: Instant) -> SolveResult {
        SolveResult {
            outcome: Err(failure),
            message: Some(message),
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes: None,
                ..SolveMetrics::default()
            },
            stages: Vec::new(),
        }
    }
}

impl Solver for PatternSolver {
    fn kind(&self) -> PuzzleKind {
        self.alphabet.kind()
    }

    fn prepare(&self) {
        let _ = self.search();
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        if facelets.kind() != self.kind() {
            return Self::failed(
                Failure::InvalidInput,
                "the cube has another size than the solver".to_owned(),
                started,
            );
        }
        if facelets.is_solved() {
            return SolveResult {
                outcome: Ok(Default::default()),
                message: None,
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    probes: None,
                    ..SolveMetrics::default()
                },
                stages: Vec::new(),
            };
        }
        let search = match self.search() {
            Ok(search) => search,
            Err(message) => return Self::failed(Failure::Exhausted, message.clone(), started),
        };
        let deadline = options.time_budget.map(|budget| started + budget);
        match search.solve(facelets, deadline) {
            Probe::Found(solution) => {
                if options
                    .max_moves
                    .is_some_and(|limit| solution.len() > usize::from(limit))
                {
                    return Self::failed(
                        Failure::Exhausted,
                        format!(
                            "the shortest product of macros has {} moves, over the limit",
                            solution.len()
                        ),
                        started,
                    );
                }
                SolveResult {
                    message: Some(format!(
                        "{} moves as a product of pattern macros of {}",
                        solution.len(),
                        self.alphabet.label()
                    )),
                    outcome: Ok(solution),
                    metrics: SolveMetrics {
                        elapsed: started.elapsed(),
                        probes: None,
                        ..SolveMetrics::default()
                    },
                    stages: Vec::new(),
                }
            }
            Probe::Missing { complete: true } => Self::failed(
                Failure::Exhausted,
                format!(
                    "no product of pattern macros made of {} solves this cube; that does not mean the cube cannot be solved with them",
                    self.alphabet.label()
                ),
                started,
            ),
            Probe::Missing { complete: false } if search.capped() => Self::failed(
                Failure::Exhausted,
                format!(
                    "nothing found within the memory cap of {} MB; that does not mean the cube cannot be solved with {}",
                    self.cap.max_bytes >> 20,
                    self.alphabet.label()
                ),
                started,
            ),
            Probe::Missing { complete: false } => Self::failed(
                Failure::BudgetExhausted,
                "the time budget ran out".to_owned(),
                started,
            ),
        }
    }
}
