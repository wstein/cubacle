//! The reduction solver for the 6x6x6 (and later the 7x7x7) as a
//! [`Solver`].

use std::sync::{Arc, OnceLock};
use std::time::Instant;

use super::seven::Seven;
use super::six::Six;
use super::wings;
use crate::puzzle::{
    normalise, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver,
};
use crate::reduction::par;

type Engine = Arc<dyn Solver + Send + Sync>;

/// The stage-by-stage account of one solve, for measuring.
#[derive(Debug, Clone, Default)]
pub struct Account {
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

type WingStage<'a> = Box<dyn Fn(&Facelets) -> Result<Vec<Move>, wings::Refused> + 'a>;

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
    pub fn solve_accounted(&self, cube: &Facelets) -> (SolveResult, Account) {
        let started = Instant::now();
        let mut account = Account::default();
        let fail = |failure: Failure, message: String, account: &Account| {
            (
                SolveResult {
                    outcome: Err(failure),
                    message: Some(message),
                    metrics: SolveMetrics {
                        elapsed: started.elapsed(),
                        probes: None,
                    },
                },
                account.clone(),
            )
        };
        if cube.kind() != self.kind || !matches!(self.kind, PuzzleKind::Cube6 | PuzzleKind::Cube7) {
            return fail(
                Failure::InvalidInput,
                format!("this solver handles the {:?} only", self.kind),
                &account,
            );
        }
        let mut all: Vec<Move> = Vec::new();
        let mut cur = cube.clone();
        let note = |account: &mut Account, name: &str, moves: &[Move]| {
            account.stages.push((name.to_owned(), moves.len()));
        };
        let centres = if self.kind == PuzzleKind::Cube6 {
            self.six().centres(&cur, 40)
        } else {
            self.seven().centres(&cur).map(|(c, d)| (c, d, true))
        };
        let Some((after, done, ok)) = centres else {
            return fail(
                Failure::Internal,
                "the centre stages found nothing".into(),
                &account,
            );
        };
        if !ok {
            return fail(Failure::Internal, "step 60 found nothing".into(), &account);
        }
        for (name, names) in &done {
            let seq: MoveSequence = names.join(" ").parse().expect("catalogue moves");
            note(&mut account, name, seq.moves());
            all.extend_from_slice(seq.moves());
        }
        cur = after;

        let stages: Vec<(&str, WingStage<'_>)> = if self.kind == PuzzleKind::Cube6 {
            vec![
                (
                    "inner wings (4x4x4)",
                    Box::new(|c| wings::inner_wings(&self.four, c)),
                ),
                (
                    "outer wings (5x5x5)",
                    Box::new(|c| wings::orbit_wings(&self.five, c, 0)),
                ),
                ("3x3x3", Box::new(|c| wings::finish(&self.three, c))),
            ]
        } else {
            vec![
                (
                    "inner wings (5x5x5)",
                    Box::new(|c| wings::orbit_wings(&self.five, c, 1)),
                ),
                (
                    "outer wings (5x5x5)",
                    Box::new(|c| wings::orbit_wings(&self.five, c, 0)),
                ),
                ("3x3x3", Box::new(|c| wings::finish(&self.three, c))),
            ]
        };
        for (name, run) in stages {
            match run(&cur) {
                Ok(moves) => {
                    cur = cur.apply(&MoveSequence::new(moves.clone())).expect("fits");
                    note(&mut account, name, &moves);
                    all.extend(moves);
                }
                Err(r) => {
                    return fail(
                        r.failure,
                        format!("{}: {}", r.stage, r.message.unwrap_or_default()),
                        &account,
                    )
                }
            }
        }
        let solution = normalise(self.kind.size(), &all, false);
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
                    let bad_centres = format!("{bad:?}");
                    format!(
                        "the answer does not solve the cube (stages leave it {}solved, {bad_centres} centre stickers off)",
                        if cur.is_solved() { "" } else { "un" }
                    )
                },
                &account,
            );
        }
        (
            SolveResult {
                outcome: Ok(MoveSequence::new(solution)),
                message: None,
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    probes: None,
                },
            },
            account,
        )
    }
}

impl Solver for DwaltonSolver {
    fn kind(&self) -> PuzzleKind {
        self.kind
    }

    fn solve(&self, facelets: &Facelets, _options: &SolveOptions) -> SolveResult {
        self.solve_accounted(facelets).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::four::FourSolver;
    use crate::engines::three::Min2PhaseSolver;
    use crate::generate::scramble;

    #[test]
    fn inner_wings_keep_the_centres() {
        let three = Arc::new(Min2PhaseSolver::new());
        let four: Engine = Arc::new(FourSolver::new(three));
        let six = Six::new(par::threads());
        for _ in 0..12 {
            let cube = Facelets::solved(PuzzleKind::Cube6)
                .apply(&scramble(PuzzleKind::Cube6, 80))
                .unwrap();
            let (cube, done, _) = six.centres(&cube, 40).unwrap();
            assert!(super::super::six_stage5::centres_reduced(&cube));
            let moves = wings::inner_wings(&four, &cube).unwrap();
            let after = cube.apply(&MoveSequence::new(moves.clone())).unwrap();
            assert!(
                super::super::six_stage5::centres_reduced(&after),
                "the inner wings moved the centres; stage {:?}",
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
