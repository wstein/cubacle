//! The reduction solver for the 6x6x6 (and later the 7x7x7) as a
//! [`Solver`].

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::groups;
use super::seven::Seven;
use super::six::Six;
use super::wings;
use crate::puzzle::{
    normalize, Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver,
};
use crate::reduction::{improve, par};

type Engine = Arc<dyn Solver + Send + Sync>;

/// The stage-by-stage tally of one solve, for measuring.
#[derive(Debug, Clone, Default)]
pub struct StageTally {
    /// Stage name and number of moves, in order.
    pub stages: Vec<(String, usize)>,
}

/// Solves the 6x6x6 by reduction, after dwalton76/rubiks-cube-NxNxN-solver.
pub struct DwaltonSolver {
    kind: PuzzleKind,
    three: Engine,
    four: Engine,
    five: Engine,
    threads: usize,
    six: OnceLock<Six>,
    seven: OnceLock<Seven>,
}

type WingStage<'a> = Box<dyn Fn(&Facelets, wings::Stint) -> Result<Vec<Move>, wings::Refused> + 'a>;

/// What a solve spends on a shorter answer: a probe count (see
/// [`DwaltonSolver::tail`]) and, if the first solution had no limit, the time
/// the improving tail may take.
#[derive(Clone, Copy)]
struct Improve {
    probes: u64,
    within: Option<Duration>,
}

/// The moves of each stage after the centers.
type Tail = Vec<(&'static str, Vec<Move>)>;
type TailResult = Result<Tail, (Failure, String)>;

impl std::fmt::Debug for DwaltonSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DwaltonSolver")
            .field("kind", &self.kind)
            .field("threads", &self.threads)
            .finish_non_exhaustive()
    }
}

impl DwaltonSolver {
    /// A solver for `kind` (6x6x6) that finishes with `three`, pairs the inner
    /// wings with `four` and the outer ones with `five`.
    pub fn new(kind: PuzzleKind, three: Engine, four: Engine, five: Engine) -> DwaltonSolver {
        DwaltonSolver {
            kind,
            three,
            four,
            five,
            threads: par::threads(),
            six: OnceLock::new(),
            seven: OnceLock::new(),
        }
    }

    pub fn with_threads(mut self, threads: usize) -> DwaltonSolver {
        self.threads = threads.max(1);
        self
    }

    fn six(&self) -> &Six {
        self.six.get_or_init(|| Six::new(self.threads))
    }

    fn seven(&self) -> &Seven {
        self.seven.get_or_init(|| Seven::new(self.threads))
    }

    /// Solves and reports the moves of each stage.
    pub fn solve_tallied(&self, cube: &Facelets) -> (SolveResult, StageTally) {
        self.tallied(cube, None, None)
    }

    /// The stages after the centers (wing pairing, the 3x3x3), each with its
    /// moves. With `probes`, the wing stages keep that many reductions and
    /// the 3x3x3 stage makes that many probes at least.
    fn tail(&self, cur: &Facelets, deadline: Option<Instant>, probes: Option<u64>) -> TailResult {
        let stages: Vec<(&'static str, WingStage<'_>)> = if self.kind == PuzzleKind::Cube6 {
            vec![
                (
                    "inner wings (4x4x4)",
                    Box::new(|c, s| wings::inner_wings(&self.four, c, s)),
                ),
                (
                    "outer wings (5x5x5)",
                    Box::new(|c, s| wings::orbit_wings(&self.five, c, 0, s)),
                ),
                ("3x3x3", Box::new(|c, s| wings::finish(&self.three, c, s))),
            ]
        } else {
            vec![
                (
                    "inner wings (5x5x5)",
                    Box::new(|c, s| wings::orbit_wings(&self.five, c, 1, s)),
                ),
                (
                    "outer wings (5x5x5)",
                    Box::new(|c, s| wings::orbit_wings(&self.five, c, 0, s)),
                ),
                ("3x3x3", Box::new(|c, s| wings::finish(&self.three, c, s))),
            ]
        };
        let mut cur = cur.clone();
        let mut parts: Tail = Vec::new();
        for (name, run) in stages {
            let left = match deadline {
                Some(d) if Instant::now() >= d => {
                    return Err((
                        Failure::BudgetExhausted,
                        format!("the time limit was reached before the stage {name}"),
                    ))
                }
                Some(d) => Some(
                    d.saturating_duration_since(Instant::now())
                        .max(Duration::from_millis(1)),
                ),
                None => None,
            };
            match run(&cur, (left, probes)) {
                Ok(moves) => {
                    cur = cur.apply(&MoveSequence::new(moves.clone())).expect("fits");
                    parts.push((name, moves));
                }
                Err(r) => {
                    return Err((
                        r.failure,
                        format!("{}: {}", r.stage, r.message.unwrap_or_default()),
                    ))
                }
            }
        }
        Ok(parts)
    }

    /// Solves within `budget`, if there is one, and reports the moves of each
    /// stage. The clock is looked at inside the center searches (every few
    /// thousand nodes) and between the stages; the engines of the wing
    /// stages get what is left.
    fn tallied(
        &self,
        cube: &Facelets,
        budget: Option<Duration>,
        better: Option<Improve>,
    ) -> (SolveResult, StageTally) {
        let started = Instant::now();
        let deadline = budget.and_then(|budget| started.checked_add(budget));
        let expired = || deadline.is_some_and(|d| Instant::now() >= d);
        let mut tally = StageTally::default();
        let fail = |failure: Failure, message: String, tally: &StageTally| {
            (
                SolveResult {
                    outcome: Err(failure),
                    message: Some(message),
                    metrics: SolveMetrics {
                        elapsed: started.elapsed(),
                        probes: None,
                    },
                    stages: Vec::new(),
                },
                tally.clone(),
            )
        };
        if cube.kind() != self.kind || !matches!(self.kind, PuzzleKind::Cube6 | PuzzleKind::Cube7) {
            return fail(
                Failure::InvalidInput,
                format!("this solver handles the {:?} only", self.kind),
                &tally,
            );
        }
        // The 7x7x7 keeps its six face centers fixed; with a middle-slice turn
        // in the scramble the center stages would search for ever.
        if self.kind == PuzzleKind::Cube7 {
            let stickers = cube.stickers();
            if (0..6).any(|face| stickers[face * 49 + 24] != Face::ALL[face]) {
                let (failure, why) = crate::puzzle::centers_off_face(cube);
                return fail(failure, why, &tally);
            }
        }
        // The Orbit64 codec checks that the pieces form a reachable cube; do it
        // before the search so an impossible cube is reported at once and plainly.
        if let Err(error) = crate::notation::orbit64::encode(cube) {
            return fail(Failure::InvalidInput, error.to_string(), &tally);
        }
        let mut all: Vec<Move> = Vec::new();
        // each step of the engine with the length of `all` at its end
        let mut marks: Vec<(&'static str, usize)> = Vec::new();
        let mut cur = cube.clone();
        let note = |tally: &mut StageTally, name: &str, moves: &[Move]| {
            tally.stages.push((name.to_owned(), moves.len()));
        };
        let centers = if self.kind == PuzzleKind::Cube6 {
            self.six().centers(&cur, 40, deadline)
        } else {
            self.seven()
                .centers(&cur, deadline)
                .map(|(c, d)| (c, d, true))
        };
        let timed_out = "the time limit was reached in the center stages";
        if expired() {
            return fail(Failure::BudgetExhausted, timed_out.into(), &tally);
        }
        let Some((after, done, ok)) = centers else {
            return fail(
                Failure::Internal,
                "the center stages found nothing".into(),
                &tally,
            );
        };
        if !ok {
            return fail(Failure::Internal, "step 60 found nothing".into(), &tally);
        }
        for (name, names) in &done {
            let seq: MoveSequence = names.join(" ").parse().expect("catalog moves");
            note(&mut tally, name, seq.moves());
            all.extend_from_slice(seq.moves());
            marks.push((name, all.len()));
        }
        cur = after;

        // The first solution is probe 1; with a probe count, the same tail is
        // done again with more reductions and a stronger 3x3x3 stage, and the
        // shorter total wins (the first on a tie).
        let first = match self.tail(&cur, deadline, None) {
            Ok(parts) => parts,
            Err((failure, message)) => return fail(failure, message, &tally),
        };
        let total = |parts: &Tail| {
            let mut joined = all.clone();
            joined.extend(parts.iter().flat_map(|(_, moves)| moves.iter().copied()));
            normalize(self.kind.size(), &joined, false).len()
        };
        let mut chosen = first;
        let mut improved = None;
        if let Some(Improve { probes: n, within }) = better {
            let before = total(&chosen);
            let limit = within.map_or(deadline, |w| Instant::now().checked_add(w));
            // A timeout here keeps the first solution.
            if let Ok(better) = self.tail(&cur, limit, Some(n)) {
                if total(&better) < before {
                    chosen = better;
                }
            }
            improved = Some((before, total(&chosen)));
        }
        for (name, moves) in chosen {
            note(&mut tally, name, &moves);
            all.extend(moves);
            marks.push((name, all.len()));
        }
        let solution = normalize(self.kind.size(), &all, false);
        let ok = cube
            .apply(&MoveSequence::new(solution.clone()))
            .is_ok_and(|c| c.is_solved());
        if !ok {
            return fail(
                Failure::Internal,
                {
                    let n = self.kind.size();
                    let st = cur.stickers();
                    let bad: Vec<_> = (0..6)
                        .flat_map(|f| {
                            (1..n - 1).flat_map(move |r| (1..n - 1).map(move |c| (f, r, c)))
                        })
                        .filter(|&(f, r, c)| st[f * n * n + r * n + c] as usize != f)
                        .map(|(f, r, c)| (f, r, c, st[f * n * n + r * n + c] as usize))
                        .collect();
                    let bad_centers = format!("{bad:?}");
                    format!(
                        "the answer does not solve the cube (stages leave it {}solved, {bad_centers} center stickers off)",
                        if cur.is_solved() { "" } else { "un" }
                    )
                },
                &tally,
            );
        }
        let mut stages = groups::stages(cube, &all, &marks, &solution);
        groups::repair(
            cube,
            &all,
            &groups::raw_ends(&marks),
            &solution,
            &mut stages,
        );
        (
            SolveResult {
                outcome: Ok(MoveSequence::new(solution)),
                message: improved
                    .map(|(first, last)| format!("improved from {first} to {last} moves")),
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    probes: None,
                },
                stages,
            },
            tally,
        )
    }
}

impl Solver for DwaltonSolver {
    fn kind(&self) -> PuzzleKind {
        self.kind
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let better = options.min_probes.map(|probes| Improve {
            probes,
            within: None,
        });
        self.tallied(facelets, options.time_budget, better).0
    }

    /// The first solution without a time limit, then the improving tail
    /// within the budget (default one second). The work is a function of the
    /// budget, as on the 4x4x4 and 5x5x5; only the clock can cut it short.
    fn solve_improving(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let budget = options.time_budget.unwrap_or(improve::DEFAULT_BUDGET);
        let better = Improve {
            probes: improve::plan(budget).candidates as u64,
            within: Some(budget),
        };
        self.tallied(facelets, None, Some(better)).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::four::FourSolver;
    use crate::engines::three::Min2PhaseSolver;
    use crate::generate::scramble;

    #[test]
    fn inner_wings_keep_the_centers() {
        let three = Arc::new(Min2PhaseSolver::new());
        let four: Engine = Arc::new(FourSolver::new(three));
        let six = Six::new(par::threads());
        for _ in 0..12 {
            let cube = Facelets::solved(PuzzleKind::Cube6)
                .apply(&scramble(PuzzleKind::Cube6, 80))
                .unwrap();
            let (cube, done, _) = six.centers(&cube, 40, None).unwrap();
            assert!(super::super::six_stage5::centers_reduced(&cube));
            let moves = wings::inner_wings(&four, &cube, (None, None)).unwrap();
            let after = cube.apply(&MoveSequence::new(moves.clone())).unwrap();
            assert!(
                super::super::six_stage5::centers_reduced(&after),
                "the inner wings moved the centers; stage {:?}",
                done.last()
            );
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn lifted_moves_move_the_inner_blocks_like_the_four() {
        let four = Facelets::solved(PuzzleKind::Cube4);
        let six = Facelets::solved(PuzzleKind::Cube6);
        for name in ["Fw'", "Uw", "Rw2", "Dw", "Lw'", "Bw"] {
            let m: Move = name.parse().unwrap();
            let l = super::super::fake::lift(&m, 3);
            let a = four.apply(&MoveSequence::new(vec![m])).unwrap();
            let b = six.apply(&MoveSequence::new(l)).unwrap();
            for f in 0..6 {
                for (fi, fr) in [(5, 14), (6, 15), (9, 20), (10, 21)] {
                    let ci = a.stickers()[f * 16 + fi];
                    let cr = b.stickers()[f * 36 + fr];
                    assert_eq!(ci, cr, "{name} face {f} cell {fi}");
                }
            }
        }
    }
}
