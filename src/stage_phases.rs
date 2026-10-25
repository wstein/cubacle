//! The phases of a solution, as `solve` and `solve --now` both report them.

use cubacle::phases::{from_stages, phases, Phase};
use cubacle::puzzle::{Facelets, MoveSequence, SolveStage};

/// The engine's own stages where it reports them, else the state-based cut of
/// `moves` applied to `start`.
pub(crate) fn of(stages: &[SolveStage], start: &Facelets, moves: &MoveSequence) -> Vec<Phase> {
    from_stages(stages, moves.len())
        .or_else(|| phases(start, moves).ok())
        .unwrap_or_default()
}
