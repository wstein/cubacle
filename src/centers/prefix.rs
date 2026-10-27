//! The engine-prefix route: frame turns and the shortest prefix of the engine's solution.

use std::time::Instant;

use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, SolveOptions, SolveStage, Solver, Turn,
};

use super::{refine, search, uniform, Pieces, PlanError, STAGE_NAME};

/// The odd cube's face centers must be those of the start: the engines keep
/// the frame, and a goal in another frame could not be reached.
pub(super) fn keeps_frame(goal: &Facelets, start: &Facelets) -> bool {
    let kind = start.kind();
    if kind.size().is_multiple_of(2) {
        return true;
    }
    let middle = kind.size() / 2;
    (0..6).all(|face| {
        let at = face * kind.face_len() + middle * kind.size() + middle;
        goal.stickers()[at] == start.stickers()[at]
    })
}

/// The shortest prefix of the engine's solution of `target` that leaves the
/// centers uniform, and the pieces of its undoing: the stages of the solution,
/// last first, each named after the stage it takes back.
pub(super) fn from_engine(
    solver: &dyn Solver,
    model: &search::Model,
    start: &Facelets,
    target: &Facelets,
    options: &SolveOptions,
    started: Instant,
) -> Result<(MoveSequence, Pieces, bool), PlanError> {
    let mut asked = SolveOptions::default().with_slices(options.slices);
    if let Some(budget) = options.time_budget {
        asked = asked.with_time_budget(
            budget
                .checked_sub(started.elapsed())
                .filter(|d| !d.is_zero())
                .ok_or(PlanError::BudgetExhausted)?,
        );
    }
    let result = solver.solve(target, &asked);
    let stages = result.stages;
    let solution = match result.outcome {
        Ok(solution) => solution,
        Err(Failure::InvalidInput) => {
            return Err(PlanError::Impossible(
                result
                    .message
                    .unwrap_or_else(|| "no engine accepts it".into()),
            ))
        }
        Err(Failure::BudgetExhausted) => return Err(PlanError::BudgetExhausted),
        Err(failure) => {
            return Err(PlanError::Failed(format!(
                "{failure:?}{}",
                result
                    .message
                    .map_or_else(String::new, |m| format!(": {m}"))
            )))
        }
    };
    let mut states = vec![target.clone()];
    for mv in solution.moves() {
        let next = states[states.len() - 1]
            .apply(&MoveSequence::new(vec![*mv]))
            .map_err(|e| PlanError::Failed(e.to_string()))?;
        let done = uniform(&next) && keeps_frame(&next, start);
        states.push(next);
        if done {
            break;
        }
    }
    let used = states.len() - 1;
    if used == solution.len() && !(uniform(&states[used]) && keeps_frame(&states[used], start)) {
        return Err(PlanError::Failed(
            "the engine's solution never leaves the centers uniform".into(),
        ));
    }
    let sound = |state: &Facelets| keeps_frame(state, start);
    let over = || refine::out_of_time(started, options.time_budget);
    if let Some(cut) = refine::shorten(model, &states, &sound, &over) {
        let mut moves = solution.moves()[..cut.keep].to_vec();
        moves.extend_from_slice(cut.finish.moves());
        let mut pieces = vec![(STAGE_NAME.to_string(), cut.finish.len())];
        pieces.extend(undo_pieces(&stages, cut.keep));
        return Ok((MoveSequence::new(moves), pieces, true));
    }
    Ok((
        MoveSequence::new(solution.moves()[..used].to_vec()),
        undo_pieces(&stages, used),
        false,
    ))
}

/// The pieces of the undoing of the first `used` moves of a solution with
/// `stages`: the part of each stage inside the prefix, last stage first.
pub(super) fn undo_pieces(stages: &[SolveStage], used: usize) -> Vec<(String, usize)> {
    if stages.is_empty() {
        return vec![(STAGE_NAME.to_string(), used)];
    }
    let mut pieces = Vec::new();
    let mut from = 0;
    for (number, stage) in stages.iter().enumerate() {
        let end = stage.end.min(used);
        if end > from {
            pieces.push((
                format!("undo phase {}: {}", number + 1, stage.name),
                end - from,
            ));
        }
        from = from.max(end);
    }
    if from < used {
        pieces.push((STAGE_NAME.to_string(), used - from));
    }
    pieces.reverse();
    pieces
}

/// The turns of the middle layers that bring the face centers of `target`
/// into the frame of `start`. Odd cubes only; an empty list when they agree.
/// The face centers of an odd cube move only with a middle layer, and the
/// engines keep them where they are, so a target built with middle layers
/// has to be put into the frame of the start first.
pub(super) fn frame_moves(start: &Facelets, target: &Facelets) -> Result<MoveSequence, PlanError> {
    let kind = target.kind();
    if keeps_frame(target, start) {
        return Ok(MoveSequence::new(Vec::new()));
    }
    let layer = (kind.size() / 2 + 1) as u8;
    let turns = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];
    let moves: Vec<Move> = [Face::R, Face::U, Face::F]
        .into_iter()
        .flat_map(|face| turns.map(|turn| Move::new(face, layer, false, turn)))
        .collect();
    let mut level = vec![(target.clone(), Vec::new())];
    for _ in 0..4 {
        let mut next = Vec::new();
        for (state, path) in &level {
            for mv in &moves {
                let state = state
                    .apply(&MoveSequence::new(vec![*mv]))
                    .map_err(|e| PlanError::Failed(e.to_string()))?;
                let mut path = path.clone();
                path.push(*mv);
                if keeps_frame(&state, start) {
                    return Ok(MoveSequence::new(path));
                }
                next.push((state, path));
            }
        }
        level = next;
    }
    Err(PlanError::Impossible(
        "the face centers of the target are not those of the start in any orientation".into(),
    ))
}
