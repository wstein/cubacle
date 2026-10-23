//! From a reduction to a solution: the 3x3x3 hand-off and the replay of the
//! answer.
//!
//! New code (split from `solver.rs` so that `--improve` can finish several
//! reductions of one cube).

use std::time::{Duration, Instant};

use super::reduction::Reduction;
use super::solver::{join, move_of, three_by_three, FiveSolver};
use crate::puzzle::{normalise, stage_ends};
use crate::puzzle::{Facelets, Failure, Move, MoveSequence, SolveOptions, SolveStage};
use crate::reduction::improve::Plan;

/// The longest 3x3x3 hand-off the search is asked for.
const THREE_MAX_MOVES: u8 = 25;

/// What the 3x3x3 stage may take when the caller sets no time budget. A tight
/// move limit can make the 3x3x3 search run for minutes; it ends as `Exhausted`.
const HAND_OFF_BUDGET: Duration = Duration::from_secs(10);

/// Phase 2 probes the 3x3x3 stage makes at least, unless the caller asks for
/// another number (as the 4x4x4 does).
const HAND_OFF_PROBES: u64 = 500;

/// The stages of the answer as a learner sees them, each with the last of the
/// five phases it ends with. Phases 1 and 2 bring each pair of opposite
/// colours onto its own axis, phase 3 puts the wings into slots, phases 4 and
/// 5 pair them and finish the centres. The 3x3x3 stage follows.
const GROUPS: [(&str, usize); 3] = [
    ("centres staged", 2),
    ("wings staged", 3),
    ("centres and wings paired", 5),
];

/// The name of the stage that the 3x3x3 solver makes.
const THREE_STAGE: &str = "3x3 stage";

/// A finished answer: the solution, the probes made and the stages.
pub(super) struct Finished {
    pub solution: MoveSequence,
    pub probes: Option<u64>,
    pub stages: Vec<SolveStage>,
}

/// The stages of an answer: the learner-level groups of the five phases and,
/// if the 3x3x3 stage ran, that one. `raw` is the reduction followed by the
/// 3x3x3 tail, unmerged, so that the phase ends in it are exact; `fin` is the
/// answer after normalising.
fn stages(
    start: &Facelets,
    raw: &[Move],
    phase_ends: &[usize],
    fin: &[Move],
    three: bool,
) -> Vec<SolveStage> {
    let head = phase_ends.last().copied().unwrap_or(0);
    let mut names: Vec<&'static str> = GROUPS.iter().map(|&(name, _)| name).collect();
    let mut ends: Vec<usize> = GROUPS
        .iter()
        .map(|&(_, last)| phase_ends.get(last - 1).copied().unwrap_or(head))
        .collect();
    if three {
        names.push(THREE_STAGE);
        ends.push(raw.len());
    } else if let Some(last) = ends.last_mut() {
        // the reduction alone ends with the last group
        *last = raw.len();
    }
    let mapped = stage_ends(start, raw, &ends, fin, &[]);
    names
        .into_iter()
        .zip(mapped)
        .map(|(name, end)| SolveStage { name, end })
        .collect()
}

/// Why a reduction could not be finished.
pub(super) struct Miss {
    pub failure: Failure,
    pub message: String,
}

fn miss(failure: Failure, message: &str) -> Miss {
    Miss {
        failure,
        message: message.to_owned(),
    }
}

impl FiveSolver {
    /// The solution through `reduction` (move numbers of the five-phase
    /// solver): the 3x3x3 tail the injected solver finds under `options`
    /// (probes, move limit, deadline) joined to it. Also the probes made.
    pub(super) fn finish(
        &self,
        facelets: &Facelets,
        reduction: &Reduction,
        options: &SolveOptions,
        deadline: Option<Instant>,
    ) -> Result<Finished, Miss> {
        let head: Vec<Move> = reduction.moves.iter().map(|&c| move_of(c)).collect();
        if options.reduce_only {
            let fin = normalise(5, &head, false);
            let stages = stages(facelets, &head, &reduction.ends, &fin, false);
            return Ok(Finished {
                solution: MoveSequence::new(fin),
                probes: None,
                stages,
            });
        }
        let Ok(reduced) = facelets.apply(&MoveSequence::new(head.clone())) else {
            return Err(miss(
                Failure::Internal,
                "the reduction moves do not fit the cube",
            ));
        };

        let limit = options.max_moves.map(usize::from);
        if limit.is_some_and(|max| head.len() > max) {
            return Err(miss(
                Failure::Exhausted,
                "the reduction alone is longer than the move limit",
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
            u8::try_from(max - head.len()).map_or(THREE_MAX_MOVES, |n| n.min(THREE_MAX_MOVES))
        });
        let hand_off = SolveOptions {
            time_budget: Some(
                remaining.map_or(HAND_OFF_BUDGET, |r| r.max(Duration::from_millis(1))),
            ),
            max_moves: Some(tail_limit),
            min_probes: options.min_probes.or(Some(HAND_OFF_PROBES)),
            slices: false,
            reduce_only: false,
        };
        let finish = self.three.solve(&three_by_three(&reduced), &hand_off);
        let probes = finish.metrics.probes;
        let tail = match finish.outcome {
            Ok(tail) => tail,
            Err(Failure::BudgetExhausted) if deadline.is_none() => {
                return Err(miss(
                    Failure::Exhausted,
                    "no 3x3x3 solution within the move limit was found in 10 s (set a time budget to allow longer)",
                ));
            }
            Err(failure) => {
                let message = finish.message.unwrap_or_else(|| "the 3x3x3 failed".into());
                return Err(Miss {
                    failure,
                    message: format!("3x3x3 stage: {message}"),
                });
            }
        };
        if tail.moves().iter().any(|m| m.depth != 1 || m.wide) {
            return Err(miss(
                Failure::Internal,
                "the 3x3x3 stage returned a move this cube cannot make as a 3x3x3 turn",
            ));
        }
        let fin = normalise(5, &join(head.clone(), tail.moves()), options.slices);
        let mut raw = head;
        raw.extend_from_slice(tail.moves());
        let stages = stages(facelets, &raw, &reduction.ends, &fin, true);
        let solution = MoveSequence::new(fin);
        if !facelets.apply(&solution).is_ok_and(|end| end.is_solved()) {
            return Err(miss(
                Failure::Internal,
                "the solution does not solve the cube (the 3x3x3 stage was given an unsolvable state?)",
            ));
        }
        if limit.is_some_and(|max| solution.len() > max) {
            return Err(miss(
                Failure::Exhausted,
                "the solution is longer than the move limit",
            ));
        }
        Ok(Finished {
            solution,
            probes,
            stages,
        })
    }

    /// The shortest solution through any of `reductions`, each finished with
    /// the probe minimum of `plan`, in order. The first must succeed (its
    /// failure is the answer's); a later one that fails or finds the budget
    /// spent ends the loop with the best so far. Probes are summed.
    pub(super) fn finish_best(
        &self,
        facelets: &Facelets,
        reductions: &[Reduction],
        options: &SolveOptions,
        deadline: Option<Instant>,
        plan: Plan,
    ) -> Result<Finished, Miss> {
        let tail_options = SolveOptions {
            min_probes: Some(plan.probes.max(HAND_OFF_PROBES)),
            ..*options
        };
        let mut best: Option<Finished> = None;
        let mut probes = 0u64;
        for (n, reduction) in reductions.iter().enumerate() {
            match self.finish(facelets, reduction, &tail_options, deadline) {
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
