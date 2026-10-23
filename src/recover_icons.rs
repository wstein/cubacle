//! `recover --icons`: the mistake as two pictures, the way forward as steps.

use cubacle::phases::{from_stages, phases};
use cubacle::puzzle::{Facelets, MoveSequence, SolveStage};
use cubacle::recover::{Advice, Finding, Plan};
use cubacle::render::differing_stickers;

use crate::steps::{Pictures, SolutionIcons};

/// The printed solution's first `count` moves done on the original cube.
fn after(original: &Facelets, solution: &MoveSequence, count: usize) -> Option<Facelets> {
    let count = count.min(solution.len());
    let head: MoveSequence = solution.moves()[..count].iter().copied().collect();
    original.apply(&head).ok()
}

/// How many printed moves were right: up to the first mistake, or the cube
/// the search came nearest to.
fn right_moves(plan: &Plan) -> Option<usize> {
    match &plan.diagnosis.finding {
        Finding::Mistakes { mistakes, .. } => mistakes.first().map(|m| m.step.saturating_sub(1)),
        Finding::Near { done, .. } => Some(*done),
        _ => None,
    }
}

/// What the pictures of `recover --icons` show, as text for standard output:
/// the cube as it should be after the last correct move against the cube as it
/// is, then the moves to do now one picture each, as `solve --icons` draws them.
pub(crate) fn render(
    options: &SolutionIcons,
    original: &Facelets,
    current: &Facelets,
    plan: &Plan,
    fresh_stages: &[SolveStage],
) -> Result<String, String> {
    let pictures = Pictures::new(original.kind().size(), options).collapsing_empty_phases();
    let mut out = String::new();
    if let Some(right) = right_moves(plan) {
        if let Some(should) = after(original, &plan.solution, right) {
            let was = format!("Should be, after step {right} (the last correct one)");
            out.push_str(&pictures.states_against((&was, &should), ("Is now", current))?);
            let differ = differing_stickers(current, &should);
            let all = current.stickers().len();
            out.push_str(&format!(
                "Stickers that differ: {differ} of {all} (marked amber in graphics rows, on the visible faces).\n"
            ));
        }
    }
    let Some(moves) = plan.moves() else {
        return Ok(out);
    };
    if plan.diagnosis.rotation.is_some() && matches!(plan.advice, Advice::Recover(_)) {
        out.push_str("The way back is for the cube turned as said above: no step pictures.\n");
        return Ok(out);
    }
    // Only a fresh solve has solver stages; a way back has none.
    let found = if matches!(plan.advice, Advice::Fresh) {
        from_stages(fresh_stages, moves.len())
            .or_else(|| phases(current, moves).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    out.push_str(&pictures.steps(current, moves, &found)?);
    Ok(out)
}

/// The steps of a fresh solution of the cube as it is (`recover --fresh`).
pub(crate) fn render_fresh(
    options: &SolutionIcons,
    current: &Facelets,
    moves: &MoveSequence,
    stages: &[SolveStage],
) -> Result<String, String> {
    let pictures = Pictures::new(current.kind().size(), options).collapsing_empty_phases();
    let found = from_stages(stages, moves.len())
        .or_else(|| phases(current, moves).ok())
        .unwrap_or_default();
    pictures.steps(current, moves, &found)
}
