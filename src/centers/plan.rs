//! The route to a target with mixed centers: how it was found, why none exists, and its limits.

use std::time::Duration;

use crate::puzzle::{
    Facelets, Failure, MoveSequence, SolveMetrics, SolveOptions, SolveResult, SolveStage,
};

/// How the moves `w` were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// The route between start and target itself, found when they are close:
    /// no uniform goal, no fix-up.
    Direct,
    /// The shortest sequence of layer moves.
    Search,
    /// A short sequence of layer moves from a beam search, not proven shortest.
    Beam,
    /// A prefix of the engine's solution of the target.
    Engine,
    /// A prefix of the engine's solution of the target, its last moves
    /// replaced by a shorter finish from a beam search.
    Refined,
    /// The moves that built the target, undone.
    Known,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Method::Direct => "direct route from start to target",
            Method::Search => "shortest layer moves",
            Method::Beam => "layer moves from a beam search",
            Method::Engine => "prefix of the engine's solution of the target",
            Method::Refined => {
                "prefix of the engine's solution of the target, finished by a beam search"
            }
            Method::Known => "the moves that built the target, undone",
        })
    }
}

/// Why no fix-up could be planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// The target cannot exist as a cube of this size.
    Impossible(String),
    /// The time budget ran out while planning.
    BudgetExhausted,
    /// The engine could not sort the centers of the target.
    Failed(String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Impossible(why) => {
                write!(f, "the target state is impossible ({why})")
            }
            PlanError::BudgetExhausted => {
                write!(
                    f,
                    "the time budget ran out while fixing the target's centers"
                )
            }
            PlanError::Failed(why) => {
                write!(f, "the centers of the target could not be fixed up ({why})")
            }
        }
    }
}

impl std::error::Error for PlanError {}

/// The route to a target with mixed centers: solve to [`Plan::goal`], then
/// make the [`Plan::tail`] moves.
#[derive(Debug, Clone)]
pub struct Plan {
    pub(super) goal: Facelets,
    pub(super) tail: MoveSequence,
    /// The stages of the tail: name and moves of the tail up to its end.
    pub(super) tail_stages: Vec<(&'static str, usize)>,
    pub(super) method: Method,
    pub(super) spent: Duration,
}

impl Plan {
    /// The state with uniform centers to solve to first.
    pub fn goal(&self) -> &Facelets {
        &self.goal
    }

    /// The stages of the fix-up: name and the number of its moves up to the
    /// end of the stage.
    pub fn tail_stages(&self) -> &[(&'static str, usize)] {
        &self.tail_stages
    }

    /// The center fix-up: the moves from the goal to the target.
    pub fn tail(&self) -> &MoveSequence {
        &self.tail
    }

    pub fn method(&self) -> Method {
        self.method
    }

    /// `options` for the solve to the goal: the move limit and the time
    /// budget are those of the whole route, less the tail and the planning.
    /// A limit the tail alone exceeds, or a budget spent, is a failed result.
    pub fn limit(&self, options: &SolveOptions) -> Result<SolveOptions, SolveResult> {
        let failed = |failure, message: String| SolveResult {
            outcome: Err(failure),
            message: Some(message),
            metrics: SolveMetrics {
                elapsed: self.spent,
                probes: None,
                ..SolveMetrics::default()
            },
            stages: Vec::new(),
        };
        let mut limited = *options;
        if let Some(most) = options.max_moves {
            let room = usize::from(most).checked_sub(self.tail.len());
            match room.filter(|&room| room > 0) {
                Some(room) => limited.max_moves = Some(room as u8),
                None => {
                    return Err(failed(
                        Failure::Exhausted,
                        format!(
                            "the center fix-up alone takes {} moves, over the limit of {most}",
                            self.tail.len()
                        ),
                    ))
                }
            }
        }
        if let Some(budget) = options.time_budget {
            match budget.checked_sub(self.spent).filter(|d| !d.is_zero()) {
                Some(left) => limited.time_budget = Some(left),
                None => {
                    return Err(failed(
                        Failure::BudgetExhausted,
                        "the budget went into planning the center fix-up".into(),
                    ))
                }
            }
        }
        Ok(limited)
    }

    /// The result of the solve to the goal, extended by the fix-up, with the
    /// fix-up as its last stage.
    pub fn complete(&self, result: SolveResult) -> SolveResult {
        let mut result = result;
        result.metrics.elapsed += self.spent;
        result.metrics.evidence = Default::default();
        let Ok(solution) = &result.outcome else {
            return result;
        };
        let reached = solution.len();
        let mut moves = solution.moves().to_vec();
        moves.extend_from_slice(self.tail.moves());
        let total = moves.len();
        if reached == 0 {
            // nothing was solved first (a direct route): no empty stages
            result.stages.clear();
        } else if result.stages.is_empty() {
            result.stages.push(SolveStage {
                name: "reduction to the target",
                end: reached,
            });
        }
        for &(name, end) in &self.tail_stages {
            result.stages.push(SolveStage {
                name,
                end: reached + end,
            });
        }
        debug_assert_eq!(result.stages.last().map(|s| s.end), Some(total));
        result.outcome = Ok(MoveSequence::new(moves));
        result
    }
}
impl PlanError {
    /// The failed result that stands for this error.
    pub fn into_result(self) -> SolveResult {
        let failure = match self {
            PlanError::Impossible(_) => Failure::InvalidInput,
            PlanError::BudgetExhausted => Failure::BudgetExhausted,
            PlanError::Failed(_) => Failure::Exhausted,
        };
        SolveResult {
            outcome: Err(failure),
            message: Some(self.to_string()),
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}
