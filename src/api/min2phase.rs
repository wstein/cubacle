use std::time::Instant;

use super::{
    Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};
use crate::{solve_with_budget, solve_with_stats, SolveOutcome};

/// The longest solution the engine searches for when no limit is given.
const DEFAULT_MAX_MOVES: u8 = 21;

/// The 3x3x3 two-phase engine behind the [`Solver`] facade.
#[derive(Debug, Clone, Copy, Default)]
pub struct Min2PhaseSolver;

impl Min2PhaseSolver {
    pub fn new() -> Min2PhaseSolver {
        Min2PhaseSolver
    }
}

impl Solver for Min2PhaseSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn prepare(&self) {
        // The engine builds its tables on first use; a trivial solve forces them.
        let _ = solve_with_stats(&Facelets::solved(PuzzleKind::Cube3).to_string(), 1);
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        if facelets.kind() != PuzzleKind::Cube3 {
            return failed(
                Failure::InvalidInput,
                format!(
                    "expected a 3x3x3, got a {0}x{0}x{0}",
                    facelets.kind().size()
                ),
                started,
                None,
            );
        }

        let text = facelets.to_string();
        let max_moves = options.max_moves.unwrap_or(DEFAULT_MAX_MOVES);
        let stats = match options.time_budget {
            Some(budget) => solve_with_budget(&text, max_moves, budget),
            None => solve_with_stats(&text, max_moves),
        };
        let probes = Some(stats.probes);

        if !stats.solution.starts_with("Error") {
            return match stats.solution.parse::<MoveSequence>() {
                Ok(solution) => SolveResult {
                    outcome: Ok(solution),
                    message: None,
                    metrics: metrics(started, probes),
                },
                Err(error) => failed(
                    Failure::InvalidInput,
                    format!("engine produced unreadable moves: {error}"),
                    started,
                    probes,
                ),
            };
        }

        let failure = match stats.outcome {
            SolveOutcome::DeadlineNoSolution => Failure::BudgetExhausted,
            _ if is_invalid_cube(&stats.solution) => Failure::InvalidInput,
            _ => Failure::Exhausted,
        };
        failed(failure, stats.solution, started, probes)
    }
}

/// Engine error codes 1 to 6 describe an impossible cube; the rest are limits.
fn is_invalid_cube(error: &str) -> bool {
    error
        .strip_prefix("Error ")
        .and_then(|code| code.trim().parse::<u8>().ok())
        .is_some_and(|code| (1..=6).contains(&code))
}

fn metrics(started: Instant, probes: Option<u64>) -> SolveMetrics {
    SolveMetrics {
        elapsed: started.elapsed(),
        probes,
    }
}

fn failed(failure: Failure, message: String, started: Instant, probes: Option<u64>) -> SolveResult {
    SolveResult {
        outcome: Err(failure),
        message: Some(message),
        metrics: metrics(started, probes),
    }
}
