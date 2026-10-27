//! Exit statuses, limits and the text of failed solves.

use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cubacle::puzzle::{Failure, PuzzleKind};

/// Exit status of a failed command (success is 0).
pub(crate) const EXIT_ERROR: u8 = 1;
/// Exit status of a solve that reached its time limit.
pub(crate) const EXIT_LIMIT: u8 = 3;

/// Set by a command whose error is a limit, not a fault of the input.
pub(crate) static LIMIT_REACHED: AtomicBool = AtomicBool::new(false);

/// The longest a solve of the 4x4x4 to 7x7x7 may run without `--max-time`.
/// Measured on a loaded machine the slowest of three random 7x7x7 solves took
/// 18 s (the 4x4x4 and 5x5x5 under 5 s, the 6x6x6 under 7 s), so this is
/// about seven times the worst.
pub(crate) const DEFAULT_MAX_TIME_MS: u64 = 120_000;

/// The text of a face center error, as `solve` prints it after `error: `.
pub(crate) const CENTERS_OFF_FACE: &str = "the face centers are not on their own faces (a middle-slice turn moved them); this solver needs the centers of each face in place";

/// The error of a solve that reached its time limit.
pub(crate) fn limit_message(limit: Duration, kind: PuzzleKind) -> String {
    let millis = limit.as_millis();
    let seconds = if millis.is_multiple_of(1000) {
        (millis / 1000).to_string()
    } else {
        format!("{:.3}", millis as f64 / 1000.0)
            .trim_end_matches('0')
            .to_owned()
    };
    let size = kind.size();
    format!(
        "time limit of {seconds} s reached on the {size}x{size}x{size} (raise it with --max-time MS)"
    )
}

/// The text of a failed solve, as `solve` and `solve --now` print it.
pub(crate) fn failure_message(failure: Failure, message: Option<String>) -> String {
    let reason = match failure {
        Failure::CentersOffFace => {
            return format!(
            "{CENTERS_OFF_FACE}\nhint: turn the cube so the centers match, or undo the slice turn"
        )
        }
        Failure::InvalidInput => "the cube state is impossible",
        Failure::BudgetExhausted => "the time budget ran out",
        Failure::Exhausted => "no solution within the move limit",
        Failure::Internal => "internal error",
    };
    format!(
        "{reason}{}",
        message.map_or_else(String::new, |m| format!(" ({m})"))
    )
}
