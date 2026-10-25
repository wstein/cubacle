//! `solve --now --icons`: the mistake as two pictures, the way forward as moves.

use cubacle::phases::Phase;
use cubacle::puzzle::{Facelets, MoveSequence, SolveStage};
use cubacle::recover::{Advice, Finding, Plan};
use cubacle::render::differing_stickers;

use crate::stage_phases;
use crate::steps::{clipped, end_line, Block, Pictures, SolutionIcons};
use crate::views::Views;

/// The printed solution's first `count` moves done on the original cube.
fn after(original: &Facelets, solution: &MoveSequence, count: usize) -> Option<Facelets> {
    let count = count.min(solution.len());
    let head: MoveSequence = solution.moves()[..count].iter().copied().collect();
    original.apply(&head).ok()
}

/// The count of differing stickers and where the amber marks show them: with
/// the other views every one is marked, in whichever picture has its face.
fn differ_line(differ: usize, all: usize, views: Views) -> String {
    let marked = if views.shows_every_face() {
        "all marked amber in graphics rows, in the view that shows them"
    } else {
        "marked amber in graphics rows on the visible faces only, add --views folded for the rest"
    };
    format!("Stickers that differ: {differ} of {all} ({marked}).\n")
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

/// The heading of the block that undoes moves: `Rollback (undo 3 moves)`.
fn rollback_heading(count: usize) -> String {
    let unit = if count == 1 { "move" } else { "moves" };
    format!("Rollback (undo {count} {unit})")
}

/// The heading of the rest of the printed solution, `first` and `last`
/// counted from 1: `Continue (moves 7-21 of the printed solution)`.
fn continue_heading(first: usize, last: usize) -> String {
    if first == last {
        format!("Continue (move {first} of the printed solution)")
    } else {
        format!("Continue (moves {first}-{last} of the printed solution)")
    }
}

/// `Moves: ` and the moves as one string, after a blank line.
fn moves_line(options: &SolutionIcons, original: &Facelets, parts: &[&MoveSequence]) -> String {
    let all: MoveSequence = parts
        .iter()
        .flat_map(|part| part.moves().iter().copied())
        .collect();
    format!(
        "\nMoves: {}\n",
        options.notation.write(original.kind(), &all)
    )
}

/// What the pictures of `solve --now --icons` show, as text for standard output:
/// the cube as it should be after the last correct move against the cube as it
/// is; for a way back the rollback and the rest of the printed solution, one
/// picture per move as `solve --icons` draws them; the end state; the moves as
/// one line. `printed` are the phases of the printed solution.
pub(crate) fn render(
    options: &SolutionIcons,
    (original, current): (&Facelets, &Facelets),
    relative: &Facelets,
    plan: &Plan,
    printed: &[Phase],
    fresh_stages: &[SolveStage],
) -> Result<String, String> {
    let pictures = Pictures::new(original.kind().size(), options).collapsing_empty_phases();
    let mut out = String::new();
    if let Some(right) = right_moves(plan) {
        if let Some(should) = after(original, &plan.solution, right) {
            let was = format!("Should be, after move {right} (the last correct one)");
            out.push_str(&pictures.states_against((&was, &should), current)?);
            let differ = differing_stickers(current, &should);
            let all = current.stickers().len();
            out.push_str(&differ_line(differ, all, options.views));
        }
    }
    let Some(moves) = plan.moves() else {
        return Ok(out);
    };
    let total = plan.solution.len();
    match &plan.advice {
        Advice::Recover(_) if plan.diagnosis.rotation.is_some() => {
            out.push_str("The way back is for the cube turned as said above: no move pictures.\n");
            return Ok(out);
        }
        Advice::Recover(route) => {
            let rollback = Block {
                base: 0,
                total: moves.len(),
                prefix: "R",
                end: false,
            };
            out.push('\n');
            out.push_str(&rollback_heading(moves.len()));
            out.push('\n');
            out.push_str(&pictures.steps_block(current, moves, &[], false, rollback)?);
            let done = route.continue_after.min(total);
            let rest: MoveSequence = plan.solution.moves()[done..].iter().copied().collect();
            if !rest.is_empty() {
                let from = after(original, &plan.solution, done)
                    .ok_or("internal error: the printed solution does not apply")?;
                let block = Block {
                    base: done,
                    total,
                    prefix: "",
                    end: false,
                };
                out.push('\n');
                out.push_str(&continue_heading(done + 1, total));
                out.push('\n');
                let phases = clipped(printed, done);
                out.push_str(&pictures.steps_block(&from, &rest, &phases, false, block)?);
            }
            let end = after(original, &plan.solution, total)
                .ok_or("internal error: the printed solution does not apply")?;
            out.push('\n');
            out.push_str(&end_line(&end)?);
            out.push_str(&pictures.start_state(&end));
            out.push_str(&moves_line(options, original, &[moves, &rest]));
        }
        _ => {
            let found = stage_phases::of(fresh_stages, relative, moves);
            out.push_str(&pictures.steps(current, moves, &found, false)?);
            out.push_str(&moves_line(options, original, &[moves]));
        }
    }
    Ok(out)
}

/// The steps of a fresh solution of the cube as it is (`solve --now --fresh`).
pub(crate) fn render_fresh(
    options: &SolutionIcons,
    (current, relative): (&Facelets, &Facelets),
    moves: &MoveSequence,
    stages: &[SolveStage],
) -> Result<String, String> {
    let pictures = Pictures::new(current.kind().size(), options).collapsing_empty_phases();
    let found = stage_phases::of(stages, relative, moves);
    pictures.steps(current, moves, &found, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_line_says_where_the_marks_are() {
        for views in [Views::Folded, Views::Both, Views::Net, Views::All] {
            let line = differ_line(12, 54, views);
            assert!(line.contains("all marked amber"));
            assert!(!line.contains("only"));
        }
        assert!(differ_line(12, 54, Views::Urf).contains("visible faces only"));
    }
}
