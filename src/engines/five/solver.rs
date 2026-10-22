use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::model::{colours_of, id_of, sequence, MoveId, KIND};
use super::reduce::{is_reduced, reduce, simplify, Reduction, Tables};
use crate::puzzle::{
    Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};

/// The longest 3x3x3 hand-off the search is asked for.
const THREE_MAX_MOVES: u8 = 25;

/// The 5x5x5 reduction solver.
///
/// It reduces the centres and the wings itself and hands the resulting
/// 3x3x3 to the solver it was built with. That solver is injected through
/// the [`Solver`] trait, so any 3x3x3 engine (or a stub) will do.
///
/// Its 3-cycle library is built on first use or by [`Solver::prepare`] and
/// is shared by clones.
#[derive(Clone)]
pub struct FiveSolver {
    three: Arc<dyn Solver + Send + Sync>,
    tables: Arc<OnceLock<Tables>>,
}

impl fmt::Debug for FiveSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FiveSolver")
            .field("tables_built", &self.tables.get().is_some())
            .finish()
    }
}

impl FiveSolver {
    /// A solver that finishes with `three`, which must handle the 3x3x3.
    pub fn new(three: Arc<dyn Solver + Send + Sync>) -> FiveSolver {
        FiveSolver {
            three,
            tables: Arc::new(OnceLock::new()),
        }
    }

    fn tables(&self) -> &Tables {
        self.tables.get_or_init(Tables::build)
    }
}

impl Solver for FiveSolver {
    fn kind(&self) -> PuzzleKind {
        KIND
    }

    fn prepare(&self) {
        self.tables();
        self.three.prepare();
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let done = |outcome: Result<MoveSequence, Failure>, message: Option<String>| SolveResult {
            outcome,
            message,
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes: None,
            },
        };
        let fail = |failure: Failure, message: &str| done(Err(failure), Some(message.to_owned()));

        if facelets.kind() != KIND {
            let size = facelets.kind().size();
            return fail(
                Failure::InvalidInput,
                &format!("expected a 5x5x5, got a {size}x{size}x{size}"),
            );
        }
        let tables = self.tables();
        let mut colours = colours_of(facelets);
        let mut moves = match reduce(tables, &mut colours) {
            Ok(moves) => moves,
            Err(Reduction::Impossible(reason)) => return fail(Failure::InvalidInput, &reason),
            Err(Reduction::Stuck(stage)) => {
                return fail(
                    Failure::Exhausted,
                    &format!("the reduction found no way on for {stage}"),
                )
            }
        };
        debug_assert!(is_reduced(tables, &colours));

        let remaining = match options.time_budget {
            Some(budget) => match budget.checked_sub(started.elapsed()) {
                Some(left) if left > Duration::ZERO => Some(left),
                _ => {
                    return fail(
                        Failure::BudgetExhausted,
                        "the reduction used up the budget before the 3x3x3 stage",
                    )
                }
            },
            None => None,
        };
        let limit = options.max_moves.map(usize::from);
        let used = simplify(&moves).len();
        if limit.is_some_and(|max| used >= max) {
            return fail(
                Failure::Exhausted,
                "the reduction alone exceeds the move limit",
            );
        }
        let hand_off = SolveOptions {
            time_budget: remaining,
            max_moves: Some(limit.map_or(THREE_MAX_MOVES, |max| {
                u8::try_from(max - used)
                    .unwrap_or(u8::MAX)
                    .min(THREE_MAX_MOVES)
            })),
        };
        let reduced = tables.model.three_by_three(&colours);
        let finish = self.three.solve(&reduced, &hand_off);
        let tail = match finish.outcome {
            Ok(tail) => tail,
            Err(failure) => {
                let message = finish.message.unwrap_or_else(|| "the 3x3x3 failed".into());
                return fail(failure, &format!("3x3x3 stage: {message}"));
            }
        };
        // Believe the 3x3x3 stage only after replaying its answer on the cube it got.
        if !reduced.apply(&tail).is_ok_and(|end| end.is_solved()) {
            return fail(
                Failure::InvalidInput,
                "the corners and middle edges do not form a solvable 3x3x3",
            );
        }
        let Some(tail) = tail
            .moves()
            .iter()
            .map(id_of)
            .collect::<Option<Vec<MoveId>>>()
        else {
            return fail(
                Failure::InvalidInput,
                "the 3x3x3 stage returned a move this cube lacks",
            );
        };
        moves.extend(tail);

        let solution = sequence(&simplify(&moves));
        if !replays(facelets, &solution) {
            return fail(
                Failure::Exhausted,
                "internal error: the solution does not solve the cube",
            );
        }
        if limit.is_some_and(|max| solution.len() > max) {
            return fail(
                Failure::Exhausted,
                "the solution is longer than the move limit",
            );
        }
        done(Ok(solution), None)
    }
}

/// Whether performing `solution` on `facelets` gives the solved cube.
fn replays(facelets: &Facelets, solution: &MoveSequence) -> bool {
    facelets
        .apply(solution)
        .is_ok_and(|end| end.stickers() == Facelets::solved(KIND).stickers())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::super::model::support::scrambled;
    use super::*;
    use crate::engines::three::Min2PhaseSolver;
    use crate::puzzle::{Face, SolveMetrics};

    /// Records the 3x3x3 it is asked to solve and answers with a fixed result.
    struct Stub {
        seen: Mutex<Vec<Facelets>>,
        answer: Result<&'static str, Failure>,
    }

    impl Solver for Stub {
        fn kind(&self) -> PuzzleKind {
            PuzzleKind::Cube3
        }

        fn solve(&self, facelets: &Facelets, _: &SolveOptions) -> SolveResult {
            self.seen.lock().unwrap().push(facelets.clone());
            SolveResult {
                outcome: self
                    .answer
                    .map(|text| text.parse::<MoveSequence>().unwrap()),
                message: None,
                metrics: SolveMetrics::default(),
            }
        }
    }

    #[test]
    fn the_3x3x3_hand_off_gets_the_reduced_cube_through_the_trait() {
        // Outer turns only change the hidden 3x3x3 as a 3x3x3 would change.
        let cube = Facelets::solved(PuzzleKind::Cube5)
            .apply(&"R U".parse().unwrap())
            .unwrap();
        let stub = Arc::new(Stub {
            seen: Mutex::new(Vec::new()),
            answer: Ok("U' R'"),
        });
        let solver = FiveSolver::new(stub.clone());
        let result = solver.solve(&cube, &SolveOptions::default());
        assert_eq!(result.outcome.unwrap().to_string(), "U' R'");
        let seen = stub.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        let expected = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R U".parse().unwrap())
            .unwrap();
        assert_eq!(seen[0], expected);
    }

    #[test]
    fn a_failing_3x3x3_stage_is_reported_not_hidden() {
        let stub = Arc::new(Stub {
            seen: Mutex::new(Vec::new()),
            answer: Err(Failure::InvalidInput),
        });
        let solver = FiveSolver::new(stub);
        let cube = Facelets::solved(PuzzleKind::Cube5);
        let result = solver.solve(&cube, &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
        assert!(result.message.unwrap().contains("3x3x3"));
    }

    #[test]
    fn a_wrong_hand_off_answer_never_becomes_a_solution() {
        // The stub claims "R" solves a cube that "R" does not solve.
        let stub = Arc::new(Stub {
            seen: Mutex::new(Vec::new()),
            answer: Ok("R"),
        });
        let result = FiveSolver::new(stub).solve(
            &Facelets::solved(PuzzleKind::Cube5),
            &SolveOptions::default(),
        );
        assert!(!result.is_solved());
    }

    fn real() -> FiveSolver {
        static THREE: std::sync::OnceLock<Min2PhaseSolver> = std::sync::OnceLock::new();
        let three = THREE.get_or_init(Min2PhaseSolver::new).clone();
        FiveSolver::new(Arc::new(three))
    }

    #[test]
    fn scrambled_cubes_are_solved_and_replay() {
        let solver = real();
        for seed in 0..25 {
            let cube = scrambled(seed, 150);
            let result = solver.solve(&cube, &SolveOptions::default());
            let solution = result
                .solution()
                .unwrap_or_else(|| panic!("seed {seed}: {result:?}"));
            assert!(cube.apply(solution).unwrap().is_solved(), "seed {seed}");
            assert!(solution.moves().iter().all(|m| !m.wide && m.depth <= 2));
        }
    }

    #[test]
    fn generated_states_are_reachable_and_solve_back() {
        // Reachability of every generated state is argued in generate/five.rs;
        // this is the check that the engine really solves them.
        let solver = real();
        for _ in 0..30 {
            let cube = crate::generate::random_state(PuzzleKind::Cube5).unwrap();
            let result = solver.solve(&cube, &SolveOptions::default());
            let solution = result.solution().expect("a generated state is reachable");
            assert!(cube.apply(solution).unwrap().is_solved());
        }
    }

    #[test]
    fn a_solved_cube_needs_no_moves_beyond_the_trivial() {
        let solved = Facelets::solved(PuzzleKind::Cube5);
        let result = real().solve(&solved, &SolveOptions::default());
        assert!(result.solution().unwrap().is_empty());
    }

    #[test]
    fn wrong_size_and_impossible_cubes_fail_as_invalid_input() {
        let solver = real();
        let three = Facelets::solved(PuzzleKind::Cube3);
        assert_eq!(
            solver.solve(&three, &SolveOptions::default()).outcome,
            Err(Failure::InvalidInput)
        );
        // A corner twisted in place (URF becomes FUR): the centres and wings
        // are fine, the 3x3x3 stage refuses it.
        let mut stickers = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
        (stickers[24], stickers[25], stickers[54]) = (Face::F, Face::U, Face::R);
        let cube = Facelets::from_stickers(stickers).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
        // Stickers of different cubies swapped: the 3x3x3 engine would answer
        // with moves that do not solve it, which must not get through.
        let mut stickers = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
        (stickers[24], stickers[25], stickers[50]) = (Face::R, Face::F, Face::U);
        let cube = Facelets::from_stickers(stickers).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
    }

    #[test]
    fn limits_are_honoured() {
        let solver = real();
        let cube = scrambled(3, 150);
        let tight = SolveOptions::default().with_max_moves(5);
        assert_eq!(solver.solve(&cube, &tight).outcome, Err(Failure::Exhausted));
        let none = SolveOptions::default().with_time_budget(std::time::Duration::ZERO);
        assert_eq!(
            solver.solve(&cube, &none).outcome,
            Err(Failure::BudgetExhausted)
        );
    }
}
