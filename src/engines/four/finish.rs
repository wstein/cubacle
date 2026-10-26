//! From a reduction to a solution: the 3x3x3 hand-off, the final rotation and
//! the replay of the answer.
//!
//! New code (split from `solver.rs` so that `--improve` can finish several
//! reductions of one cube).

use std::time::{Duration, Instant};

use super::cube::Cube;
use super::facelets;
use super::output::{of_face_turn, to_move, to_real};
use super::reduce::{after, Reduction};
use super::solver::FourSolver;
use super::stages::stages;
use super::tables::Tables;
use crate::puzzle::normalize;
use crate::puzzle::{Face, Facelets, Failure, Move, MoveSequence, SolveOptions, SolveStage};
use crate::reduction::improve::Plan;

/// The longest 3x3x3 hand-off asked for (the reference asks for 21).
const THREE_MAX_MOVES: u8 = 21;
/// What the 3x3x3 stage may take when the caller sets no time budget.
const HAND_OFF_BUDGET: Duration = Duration::from_secs(10);
/// What the 3x3x3 stage may take when the caller sets a move limit: the
/// reduction is fixed, so a limit the tail cannot meet would otherwise keep
/// the 3x3x3 solver searching until the budget ends. A tail that can meet
/// the limit is found in a few milliseconds.
const LIMITED_HAND_OFF_BUDGET: Duration = Duration::from_secs(1);

/// A finished answer: the solution, the 3x3x3 probes made and the stages.
pub(super) struct Finished {
    pub solution: MoveSequence,
    pub probes: Option<u64>,
    pub stages: Vec<SolveStage>,
}

/// Why a reduction could not be finished.
pub(super) struct Miss {
    pub failure: Failure,
    pub message: String,
    pub probes: Option<u64>,
}

fn miss(failure: Failure, message: &str) -> Miss {
    Miss {
        failure,
        message: message.to_owned(),
        probes: None,
    }
}

/// Whether the centers of every face show the face's own color.
fn centers_home(cube: &Facelets) -> bool {
    let s = cube.stickers();
    Face::ALL
        .iter()
        .enumerate()
        .all(|(f, &face)| [5, 6, 9, 10].iter().all(|&i| s[f * 16 + i] == face))
}

impl FourSolver {
    /// The solution through `reduction`: its moves, the 3x3x3 tail the
    /// injected solver finds under `options` (probes, move limit, deadline),
    /// the final rotation. Also the 3x3x3 probes made.
    pub(super) fn finish(
        &self,
        t: &Tables,
        cube: &Cube,
        facelets: &Facelets,
        reduction: &Reduction,
        options: &SolveOptions,
        deadline: Option<Instant>,
    ) -> Result<Finished, Miss> {
        let model_moves = reduction.model_moves();
        let model = after(t, cube, &model_moves);
        let Some(three) = facelets::three_by_three(&model) else {
            return Err(miss(
                Failure::Internal,
                "the reduced cube has no 3x3x3 (the centers are not solved)",
            ));
        };
        let (length1, add1, sym) = (
            usize::from(reduction.cand.length1),
            reduction.cand.add1,
            usize::from(reduction.cand.sym),
        );
        let head = to_real(&t.syms, &model_moves, length1, add1, sym);
        let head_moves: Vec<Move> = head.iter().map(|&c| to_move(c)).collect();
        if options.reduce_only {
            // Keep the cube's frame: the big cubes lift these moves.
            let head = normalize(4, &head_moves, false);
            let Some(head) = self
                .swaps
                .finish_with(&self.rotations, facelets, &head, centers_home)
            else {
                return Err(miss(Failure::Internal, "the reduced centers are not home"));
            };
            let fin = normalize(4, &head, false);
            let stages = stages(
                &self.rotations,
                facelets,
                reduction,
                &head_moves,
                &fin,
                false,
            );
            return Ok(Finished {
                solution: MoveSequence::new(fin),
                probes: None,
                stages,
            });
        }
        let Ok(reduced) = facelets.apply(&MoveSequence::new(head_moves.clone())) else {
            return Err(miss(
                Failure::Internal,
                "the reduction moves do not fit the cube",
            ));
        };
        let Some(rotation) = self.rotations.find(&reduced, centers_home) else {
            return Err(miss(
                Failure::Internal,
                "the reduced cube is not a rotation of the standard color scheme",
            ));
        };

        let limit = options.max_moves.map(usize::from);
        let fixed = head_moves.len() + rotation.len();
        if limit.is_some_and(|max| fixed > max) {
            return Err(miss(
                Failure::Exhausted,
                "the reduction and the final rotation alone are longer than the move limit",
            ));
        }
        let remaining = match deadline {
            Some(d) => {
                let now = Instant::now();
                if now >= d {
                    return Err(miss(
                        Failure::BudgetExhausted,
                        "the reduction used up the time budget before the 3x3x3 stage",
                    ));
                }
                Some(d - now)
            }
            None => None,
        };
        let tail_limit = limit.map_or(THREE_MAX_MOVES, |max| {
            u8::try_from(max - fixed).map_or(THREE_MAX_MOVES, |n| n.min(THREE_MAX_MOVES))
        });
        // The cap on the 3x3x3 stage, if one applies, and whether it (not the
        // caller's deadline) is what ends the stage.
        let cap = if limit.is_some() {
            LIMITED_HAND_OFF_BUDGET
        } else {
            HAND_OFF_BUDGET
        };
        let stage_budget = match remaining {
            Some(r) => r.max(Duration::from_millis(1)).min(if limit.is_some() {
                cap
            } else {
                Duration::MAX
            }),
            None => cap,
        };
        let capped = remaining.is_none_or(|r| limit.is_some() && r > cap);
        let hand_off = SolveOptions {
            time_budget: Some(stage_budget),
            max_moves: Some(tail_limit),
            min_probes: options.min_probes.or(Some(self.hand_off_probes)),
            slices: false,
            reduce_only: false,
        };
        let finish = self.three.solve(&three, &hand_off);
        let probes = finish.metrics.probes;
        let tail = match finish.outcome {
            Ok(tail) => tail,
            Err(Failure::BudgetExhausted) if capped => {
                return Err(Miss {
                    failure: Failure::Exhausted,
                    message: format!(
                        "no 3x3x3 solution within the move limit was found in {} s{}",
                        cap.as_secs(),
                        if limit.is_some() {
                            " (the reduction is fixed; a larger limit may help)"
                        } else {
                            " (set a time budget to allow longer)"
                        }
                    ),
                    probes,
                });
            }
            Err(failure) => {
                let message = finish.message.unwrap_or_else(|| "the 3x3x3 failed".into());
                return Err(Miss {
                    failure,
                    message: format!("3x3x3 stage: {message}"),
                    probes,
                });
            }
        };
        let mut all = model_moves;
        for mv in tail.moves() {
            let Some(code) = of_face_turn(mv) else {
                return Err(miss(
                    Failure::Internal,
                    "the 3x3x3 stage returned a move this cube cannot make as a 3x3x3 turn",
                ));
            };
            all.push(code);
        }
        let real = to_real(&t.syms, &all, length1, add1, sym);
        let raw: Vec<Move> = real.iter().map(|&c| to_move(c)).collect();
        let moves = normalize(4, &raw, false);
        let Some(moves) = self.swaps.finish(&self.rotations, facelets, &moves) else {
            return Err(miss(
                Failure::Internal,
                "the solution does not solve the cube (the 3x3x3 stage was given an unsolvable state?)",
            ));
        };
        let fin = normalize(4, &moves, options.slices);
        let stages = stages(&self.rotations, facelets, reduction, &raw, &fin, true);
        let solution = MoveSequence::new(fin);
        if limit.is_some_and(|max| solution.len() > max) {
            return Err(miss(
                Failure::Exhausted,
                "the solution is longer than the move limit",
            ));
        }
        if !facelets.apply(&solution).is_ok_and(|end| end.is_solved()) {
            return Err(miss(
                Failure::Internal,
                "the solution does not solve the cube",
            ));
        }
        Ok(Finished {
            solution,
            probes,
            stages,
        })
    }
}

impl FourSolver {
    /// The shortest solution through any of `reductions`, each finished with
    /// the probe minimum of `plan`, in order. The first must succeed (its
    /// failure is the answer's); a later one that fails or finds the budget
    /// spent ends the loop with the best so far. Probes are summed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_best(
        &self,
        t: &Tables,
        cube: &Cube,
        facelets: &Facelets,
        reductions: &[Reduction],
        options: &SolveOptions,
        deadline: Option<Instant>,
        plan: Plan,
    ) -> Result<Finished, Miss> {
        let tail_options = SolveOptions {
            min_probes: Some(plan.probes.max(self.hand_off_probes)),
            ..*options
        };
        let mut best: Option<Finished> = None;
        let mut probes = 0u64;
        for (n, reduction) in reductions.iter().enumerate() {
            match self.finish(t, cube, facelets, reduction, &tail_options, deadline) {
                Ok(done) => {
                    probes += done.probes.unwrap_or(0);
                    if best
                        .as_ref()
                        .is_none_or(|b| done.solution.len() < b.solution.len())
                    {
                        best = Some(done);
                    }
                }
                Err(miss) if n == 0 => return Err(miss),
                Err(_) => break,
            }
        }
        best.map(|done| Finished {
            probes: Some(probes),
            ..done
        })
        .ok_or_else(|| miss(Failure::Internal, "no reduction to finish"))
    }
}
