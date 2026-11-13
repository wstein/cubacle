//! Solve a scramble to the solved cube for the WebAssembly facade.

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::puzzle::{Facelets, Failure, MoveSequence, PuzzleKind, SolveOptions, Solver};
use std::time::Duration;

/// Bytes before the text of a result: the total length and the status.
pub const HEADER: usize = 5;

/// The result text is the solution.
pub const STATUS_OK: u8 = 0;
/// The size or the scramble text is not accepted; the text says why.
pub const STATUS_BAD_INPUT: u8 = 1;
/// The solver found no solution; the text says why.
pub const STATUS_NO_SOLUTION: u8 = 2;
/// ABI 2 only: the time budget ran out before a solution was found; the text says so.
pub const STATUS_TIMEOUT: u8 = 3;

/// Wall-clock budget of one `cubacle_solve` call in ABI 1, in milliseconds. A
/// search that has not found a solution when it runs out ends with
/// [`STATUS_NO_SOLUTION`] and a text that names the budget.
pub const DEFAULT_BUDGET_MS: u64 = 2000;

/// ABI 1: solves with the default budget; a budget that runs out is status 2.
#[doc(hidden)]
pub fn solve_text(size: u32, scramble: &str, max_moves: u32) -> (u8, String) {
    let (status, text) = solve_text_budget(size, scramble, max_moves, 0);
    if status == STATUS_TIMEOUT {
        (STATUS_NO_SOLUTION, text)
    } else {
        (status, text)
    }
}

/// ABI 2: solves within `budget_ms` milliseconds (0: [`DEFAULT_BUDGET_MS`]); a
/// budget that runs out before a solution is found is [`STATUS_TIMEOUT`].
#[doc(hidden)]
pub fn solve_text_budget(
    size: u32,
    scramble: &str,
    max_moves: u32,
    budget_ms: u32,
) -> (u8, String) {
    let kind = match size {
        2 => PuzzleKind::Cube2,
        3 => PuzzleKind::Cube3,
        other => return (STATUS_BAD_INPUT, format!("size {other} is not 2 or 3")),
    };
    let moves: MoveSequence = match scramble.parse() {
        Ok(moves) => moves,
        Err(error) => return (STATUS_BAD_INPUT, format!("scramble: {error}")),
    };
    let cube = match Facelets::solved(kind).apply(&moves) {
        Ok(cube) => cube,
        Err(error) => return (STATUS_BAD_INPUT, format!("scramble: {error}")),
    };
    let options = SolveOptions::default()
        .with_max_moves(max_moves.min(255) as u8)
        .with_time_budget(Duration::from_millis(if budget_ms == 0 {
            DEFAULT_BUDGET_MS
        } else {
            u64::from(budget_ms)
        }));
    let result = match kind {
        PuzzleKind::Cube2 => PocketSolver::new().solve(&cube, &options),
        _ => Min2PhaseSolver::shared().solve(&cube, &options),
    };
    if let Some(solution) = result.solution() {
        return (STATUS_OK, solution.to_string());
    }
    let status = if result.outcome == Err(Failure::BudgetExhausted) {
        STATUS_TIMEOUT
    } else {
        STATUS_NO_SOLUTION
    };
    (
        status,
        result.message.unwrap_or_else(|| "no solution".to_string()),
    )
}
