//! Targets whose centers show different colors on one face.
//!
//! The reduction engines (4x4x4 to 7x7x7) are color based: the relative state
//! `target^-1 * start` only tells a center which face to go to, so they may
//! leave the centers of a face in any order, and a target whose centers of one
//! orbit differ in color on one face cannot be aimed at directly (the target
//! of an inner slice move, a center spot, an imported pattern).
//!
//! The way out is to aim at a nearby target that has uniform centers and to
//! finish with the moves that lead from there to the real one:
//!
//! 1. find moves `w` that make the centers of the target `T` uniform, so
//!    that `U = T w` is a state the engines can aim at;
//! 2. solve the start to `U` as for any uniform target;
//! 3. append `w^-1`, which takes `U` to `T`: the center fix-up.
//!
//! `w` depends on the target alone. It is the shortest sequence of layer moves
//! (a search with a table per orbit and color) when that is short, else a
//! short one from a beam search, else the shortest prefix of the engine's own
//! solution of `T` that leaves the centers uniform. The engine is used only
//! through [`Solver`].

mod search;

use std::time::{Duration, Instant};

use crate::puzzle::{
    center_orbits, interned, normalize, relative_state, Face, Facelets, Failure, Move,
    MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, SolveStage, Solver, Turn,
    PHASE_MOVES,
};

/// Search steps the shortest-sequence search may take before the engine's
/// solution is used instead; it counts work, so the answer is the same on
/// every machine.
const SEARCH_NODES: u64 = 3_000_000;
/// The longest fix-up the search looks for.
const SEARCH_DEPTH: usize = 12;
/// The beam searches tried when the exact one finds nothing: whether the
/// colors go to their own faces (else to any face), and the beam width.
const BEAMS: [(bool, usize); 3] = [(true, 250), (true, 1000), (false, 500)];
/// The longest sequence a beam search builds.
const BEAM_DEPTH: usize = 40;

/// The name of the last stage of a route that ends in a fix-up.
pub const STAGE_NAME: &str = "center fix-up";

/// The stages of a fix-up from its named pieces (name, moves): a piece over
/// [`PHASE_MOVES`] moves is cut into equal parts `name (1/2)`. Each stage
/// is given with the number of fix-up moves up to its end.
fn tail_stages(pieces: &[(String, usize)]) -> Vec<(&'static str, usize)> {
    let mut out = Vec::new();
    let mut done = 0;
    for (name, len) in pieces.iter().filter(|(_, len)| *len > 0) {
        let parts = len.div_ceil(PHASE_MOVES);
        for part in 0..parts {
            // equal parts: the first ones take the rest
            let size = len / parts + usize::from(part < len % parts);
            done += size;
            let shown = if parts == 1 {
                interned(name.clone())
            } else {
                interned(format!("{name} ({}/{parts})", part + 1))
            };
            out.push((shown, done));
        }
    }
    out
}

/// Whether every face of `state` shows one color in each of its center
/// orbits. Cubes below 4x4x4 have nothing to sort and are uniform.
pub fn uniform(state: &Facelets) -> bool {
    let kind = state.kind();
    if kind.size() < 4 {
        return true;
    }
    center_orbits(kind).iter().all(|orbit| {
        (0..6).all(|face| {
            let mut on = orbit
                .iter()
                .filter(|&&s| s / kind.face_len() == face)
                .map(|&s| state.stickers()[s]);
            let first = on.next();
            on.all(|color| Some(color) == first)
        })
    })
}

/// How the moves `w` were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// The shortest sequence of layer moves.
    Search,
    /// A short sequence of layer moves from a beam search, not proven shortest.
    Beam,
    /// A prefix of the engine's solution of the target.
    Engine,
    /// The moves that built the target, undone.
    Known,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Method::Search => "shortest layer moves",
            Method::Beam => "layer moves from a beam search",
            Method::Engine => "prefix of the engine's solution of the target",
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
    goal: Facelets,
    tail: MoveSequence,
    /// The stages of the tail: name and moves of the tail up to its end.
    tail_stages: Vec<(&'static str, usize)>,
    method: Method,
    spent: Duration,
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
        let Ok(solution) = &result.outcome else {
            return result;
        };
        let reached = solution.len();
        let mut moves = solution.moves().to_vec();
        moves.extend_from_slice(self.tail.moves());
        let total = moves.len();
        if result.stages.is_empty() {
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

/// The odd cube's face centers must be those of the start: the engines keep
/// the frame, and a goal in another frame could not be reached.
fn keeps_frame(goal: &Facelets, start: &Facelets) -> bool {
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
fn from_engine(
    solver: &dyn Solver,
    start: &Facelets,
    target: &Facelets,
    options: &SolveOptions,
    started: Instant,
) -> Result<(MoveSequence, Vec<(String, usize)>), PlanError> {
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
    let mut state = target.clone();
    for (at, mv) in solution.moves().iter().enumerate() {
        state = state
            .apply(&MoveSequence::new(vec![*mv]))
            .map_err(|e| PlanError::Failed(e.to_string()))?;
        if uniform(&state) && keeps_frame(&state, start) {
            return Ok((
                MoveSequence::new(solution.moves()[..=at].to_vec()),
                undo_pieces(&stages, at + 1),
            ));
        }
    }
    Err(PlanError::Failed(
        "the engine's solution never leaves the centers uniform".into(),
    ))
}

/// The pieces of the undoing of the first `used` moves of a solution with
/// `stages`: the part of each stage inside the prefix, last stage first.
fn undo_pieces(stages: &[SolveStage], used: usize) -> Vec<(String, usize)> {
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
fn frame_moves(start: &Facelets, target: &Facelets) -> Result<MoveSequence, PlanError> {
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

fn model_of(kind: PuzzleKind) -> search::Model {
    search::Model::new(kind)
}

/// The moves from `target` to the solved cube when `hint`, the moves that
/// build the target from the solved cube, does that; shortened where its
/// layers allow.
fn known_way(target: &Facelets, hint: Option<&MoveSequence>) -> Option<MoveSequence> {
    let kind = target.kind();
    let hint = hint.filter(|m| m.fits(kind))?;
    let shortened = MoveSequence::new(normalize(kind.size(), hint.moves(), true));
    let hint = if shortened.len() < hint.len() {
        &shortened
    } else {
        hint
    };
    let way = search::inverse(hint);
    target
        .apply(&way)
        .ok()
        .filter(Facelets::is_solved)
        .map(|_| way)
}

/// The plan to reach `target` from `start`, or `None` when the target has
/// uniform centers (or the cube has none to sort) and the plain relative
/// state does. `solver` is the engine of the cube's size. `hint` may be the
/// moves that build the target from the solved cube, when they are known: the
/// plan is then never longer than undoing them.
pub fn plan(
    solver: &dyn Solver,
    start: &Facelets,
    target: &Facelets,
    options: &SolveOptions,
    hint: Option<&MoveSequence>,
) -> Result<Option<Plan>, PlanError> {
    if target.kind() != start.kind() || uniform(target) || start == target {
        return Ok(None);
    }
    let began = Instant::now();
    let kind = target.kind();
    debug_assert!(kind >= PuzzleKind::Cube4);
    if !model_of(kind).balanced(target) {
        return Err(PlanError::Impossible(
            "a center orbit shows a color more or less than four times".into(),
        ));
    }
    let known = known_way(target, hint);
    // the target in the frame of the start
    let frame = frame_moves(start, target)?;
    let base = target
        .apply(&frame)
        .map_err(|e| PlanError::Failed(e.to_string()))?;
    // only a sequence shorter than the known way is worth finding
    let room = known
        .as_ref()
        .map_or(usize::MAX, |w| w.len().saturating_sub(frame.len() + 1));
    let model = model_of(kind);
    let sound = |w: &MoveSequence| {
        base.apply(w)
            .is_ok_and(|goal| uniform(&goal) && keeps_frame(&goal, start))
    };
    // colors to their own faces, then colors to any face (another frame),
    // the second only if it can be shorter
    let own = model
        .shortest(&base, true, SEARCH_DEPTH.min(room), SEARCH_NODES)
        .filter(sound);
    let shorter = own
        .as_ref()
        .map_or(room, |w| w.len().saturating_sub(1).min(room));
    let any = (own.is_none() || shorter > 0)
        .then(|| model.shortest(&base, false, SEARCH_DEPTH.min(shorter), SEARCH_NODES))
        .flatten()
        .filter(sound);
    let found = any
        .or(own)
        .map(|w| (w, Method::Search))
        .or_else(|| {
            BEAMS.iter().find_map(|&(own, width)| {
                model
                    .beam(&base, own, width, BEAM_DEPTH.min(room))
                    .filter(sound)
                    .map(|w| (w, Method::Beam))
            })
        })
        .map(|(w, method)| {
            let mut moves = frame.moves().to_vec();
            moves.extend_from_slice(w.moves());
            (MoveSequence::new(moves), method)
        });
    let found = match (found, known) {
        (Some((w, method)), Some(known)) if known.len() < w.len() => {
            let _ = (w, method);
            Some((known, Method::Known))
        }
        (Some(found), _) => Some(found),
        (None, Some(known)) => Some((known, Method::Known)),
        (None, None) => None,
    };
    let (sorting, method, undoing) = match found {
        Some((w, method)) => (w, method, None),
        None => {
            let (prefix, pieces) = from_engine(solver, start, &base, options, began)?;
            let mut moves = frame.moves().to_vec();
            moves.extend_from_slice(prefix.moves());
            // the undoing of the frame moves comes last
            let mut pieces = pieces;
            if !frame.is_empty() {
                pieces.push((STAGE_NAME.to_string(), frame.len()));
            }
            (MoveSequence::new(moves), Method::Engine, Some(pieces))
        }
    };
    let pieces = undoing.unwrap_or_else(|| vec![(STAGE_NAME.to_string(), sorting.len())]);
    let goal = target
        .apply(&sorting)
        .map_err(|e| PlanError::Failed(e.to_string()))?;
    relative_state(start, &goal).map_err(|e| PlanError::Impossible(e.to_string()))?;
    Ok(Some(Plan {
        goal,
        tail: search::inverse(&sorting),
        tail_stages: tail_stages(&pieces),
        method,
        spent: began.elapsed(),
    }))
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

/// Solves `start` to a `target` with mixed centers with `engine`, as the
/// plan says; for a target with uniform centers this is the plain solve of
/// the relative state. `hint` is as for [`plan`]. The answer is not replayed
/// here.
pub fn solve_to(
    engine: &dyn Solver,
    start: &Facelets,
    target: &Facelets,
    options: &SolveOptions,
    hint: Option<&MoveSequence>,
) -> SolveResult {
    let plan = match plan(engine, start, target, options, hint) {
        Ok(plan) => plan,
        Err(error) => return error.into_result(),
    };
    let goal = plan.as_ref().map_or(target, Plan::goal);
    let relative = match relative_state(start, goal) {
        Ok(relative) => relative,
        Err(error) => {
            return PlanError::Impossible(error.to_string()).into_result();
        }
    };
    match plan {
        None => engine.solve(&relative, options),
        Some(plan) => match plan.limit(options) {
            Ok(limited) => plan.complete(engine.solve(&relative, &limited)),
            Err(failed) => failed,
        },
    }
}

#[cfg(test)]
mod tests;
