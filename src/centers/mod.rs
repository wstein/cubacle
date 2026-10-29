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
//! solution of `T` that leaves the centers uniform, with its end shortened by
//! a beam search (see `refine`). The engine is used only through [`Solver`].

mod direct;
mod only;
mod plan;
mod prefix;
mod refine;
mod search;

pub use only::{solve_to as solve_centers_only, CenterGoal};
pub use plan::{Method, Plan, PlanError};
use prefix::{frame_moves, from_engine, keeps_frame};

use std::time::Instant;

use crate::puzzle::{
    center_orbits, interned, normalize, relative_state, Facelets, MoveSequence, PuzzleKind,
    SolveOptions, SolveResult, Solver, PHASE_MOVES,
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
/// The beams of the 4x4x4, whose one center orbit a wide beam sorts in about
/// four moves fewer than a narrow one.
const BEAMS_FOUR: [(bool, usize); 2] = [(false, 16_000), (true, 16_000)];
/// The engine's sorting is only tried against a way (beam, or the target's
/// moves undone) longer than this: it is the least the engine's sorting takes
/// on a cube of the size (measured, after the end is shortened), so a shorter
/// way is not worth a solve.
fn engine_floor(kind: PuzzleKind) -> usize {
    match kind {
        PuzzleKind::Cube4 => 10,
        PuzzleKind::Cube5 => 25,
        PuzzleKind::Cube6 => 80,
        _ => 120,
    }
}
/// The longest sequence a beam search builds.
const BEAM_DEPTH: usize = 40;

/// The named pieces of a fix-up: name and moves.
type Pieces = Vec<(String, usize)>;

/// The name of the last stage of a route that ends in a fix-up.
pub const STAGE_NAME: &str = "center fix-up";

/// The name of the only stage of a direct route.
pub const DIRECT_STAGE_NAME: &str = "direct route";

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

fn beams(kind: PuzzleKind) -> &'static [(bool, usize)] {
    if kind == PuzzleKind::Cube4 {
        &BEAMS_FOUR
    } else {
        &BEAMS
    }
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
    // Close states: the few moves between them beat any detour. The plan
    // then solves the start to itself (nothing to do) and makes the route as
    // its tail.
    if let Some(route) = direct::route(start, target) {
        let len = route.len();
        return Ok(Some(Plan {
            goal: start.clone(),
            tail: route,
            tail_stages: tail_stages(&[(DIRECT_STAGE_NAME.to_string(), len)]),
            method: Method::Direct,
            spent: began.elapsed(),
        }));
    }
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
            // every beam is run and the shortest sequence wins
            beams(kind)
                .iter()
                .filter_map(|&(own, width)| {
                    model
                        .beam(&base, own, width, BEAM_DEPTH.min(room))
                        .filter(sound)
                })
                .min_by_key(MoveSequence::len)
                .map(|w| (w, Method::Beam))
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
    // The engine's own sorting, its end shortened: the only way when nothing
    // above exists, and a rival to a beam or a long known way.
    let rival = found
        .as_ref()
        .is_none_or(|(w, method)| *method != Method::Search && w.len() > engine_floor(kind));
    let engine = if rival {
        match from_engine(solver, &model, start, &base, options, began) {
            Ok((prefix, pieces, cut)) => {
                let mut moves = frame.moves().to_vec();
                moves.extend_from_slice(prefix.moves());
                // the undoing of the frame moves comes last
                let mut pieces = pieces;
                if !frame.is_empty() {
                    pieces.push((STAGE_NAME.to_string(), frame.len()));
                }
                let method = if cut { Method::Refined } else { Method::Engine };
                Some((MoveSequence::new(moves), method, Some(pieces)))
            }
            // another way exists: the engine is only an attempt to be shorter
            Err(_) if found.is_some() => None,
            Err(error) => return Err(error),
        }
    } else {
        None
    };
    let (sorting, method, undoing) = match (found, engine) {
        (Some((w, method)), Some((e, ..))) if w.len() <= e.len() => (w, method, None),
        (_, Some(engine)) => engine,
        (Some((w, method)), None) => (w, method, None),
        (None, None) => unreachable!("the engine either answers or fails the plan"),
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
