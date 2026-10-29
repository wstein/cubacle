//! A target that constrains the centers only, on a 4x4x4 to 7x7x7.
//!
//! The target is a goal set: every center sticker has one color, the edges and
//! corners are open and stay wherever the moves put them. Two routes meet it:
//!
//! 1. an exact search over the layer moves with the center tables of
//!    [`search::Model`], which proves the answer shortest when it is short;
//! 2. the engine's own center stage (the reduction of the 4x4x4 and 5x5x5, the
//!    center stages of the 6x6x6 and 7x7x7, through [`SolveOptions::centers_only`]),
//!    the shortest prefix of its moves that meets the target, and a beam search
//!    that shortens the end (see `refine`).
//!
//! The shorter answer wins. The engine is used only through [`Solver`].

use std::time::Instant;

use crate::puzzle::{
    centers_off_face, normalize, Face, Facelets, Failure, Mask, MoveSequence, SolveMetrics,
    SolveOptions, SolveResult, Solver, Target,
};

use super::search::{Aim, Model};
use super::{refine, uniform};

/// The longest answer the exact search looks for, in moves, and the steps it
/// may take (it counts work, so the answer is the same on every machine).
const EXACT_DEPTH: usize = 12;
const EXACT_NODES: u64 = 2_000_000;
/// The largest cube size whose engine moves are shortened by a beam search.
const SHORTEN_UP_TO: usize = 5;
/// The finish after the engine's centers are uniform but in another frame.
const FRAME_DEPTH: usize = 8;
const FRAME_NODES: u64 = 1_000_000;
const FRAME_BEAM: usize = 2000;

/// What a centers-only target asks: which face each color goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CenterGoal {
    aim: Aim,
}

impl CenterGoal {
    /// Reads `target` for a solve from `start`, or says why the target is not
    /// a centers-only one this route can reach.
    ///
    /// The centers of every face must be fully specified and of one color, the
    /// six colors on six faces. On an odd cube the face centers never move, so
    /// they must be those of `start`.
    pub fn read(start: &Facelets, target: &Target) -> Result<CenterGoal, String> {
        let kind = target.kind();
        let n = kind.size();
        if start.kind() != kind {
            return Err("the start and the target are of different sizes".into());
        }
        if n < 4 {
            return Err(format!(
                "unsupported: a centers-only target needs a cube of 4x4x4 to 7x7x7, not a {n}x{n}x{n}"
            ));
        }
        let centers = Mask::centers(kind);
        let care = target.care();
        let outside = care.outside(&centers).len();
        if outside > 0 {
            return Err(format!(
                "unsupported: on a {n}x{n}x{n} a target with open stickers can only constrain the centers (the named subset `centers`); this one also constrains {outside} edge or corner stickers"
            ));
        }
        let open = centers.len() - care.len();
        if open > 0 {
            return Err(format!(
                "unsupported: the centers of a centers-only target must all be specified; {open} of {} center stickers are open",
                centers.len()
            ));
        }
        let face_len = kind.face_len();
        let mut color_of = [usize::MAX; 6];
        for position in centers.positions() {
            let face = position / face_len;
            let set = target.sets()[position];
            if set.count_ones() != 1 {
                return Err(
                    "unsupported: a center sticker of the target allows more than one color".into(),
                );
            }
            let color = set.trailing_zeros() as usize;
            if color_of[face] != usize::MAX && color_of[face] != color {
                return Err(format!(
                    "unsupported: the centers of face {} show different colors in the target; a target with mixed centers must be complete",
                    Face::ALL[face].letter()
                ));
            }
            color_of[face] = color;
        }
        let mut faces = [u8::MAX; 6];
        for (face, &color) in color_of.iter().enumerate() {
            if faces[color] != u8::MAX {
                return Err(format!(
                    "contradictory: the target puts color {} on the centers of two faces",
                    Face::ALL[color].letter()
                ));
            }
            faces[color] = face as u8;
        }
        if !n.is_multiple_of(2) {
            let middle = face_len / 2;
            for (face, &color) in color_of.iter().enumerate() {
                if start.stickers()[face * face_len + middle].index() != color {
                    let (failure, why) = centers_off_face(start);
                    return Err(if failure == Failure::CentersOffFace {
                        format!("unreachable: the target wants another color on a face center than the start has, and the face centers of a {n}x{n}x{n} stay where they are ({why})")
                    } else {
                        why
                    });
                }
            }
        }
        Ok(CenterGoal {
            aim: Aim::Faces(faces),
        })
    }
}

fn result(
    outcome: Result<MoveSequence, Failure>,
    message: String,
    started: Instant,
) -> SolveResult {
    SolveResult {
        outcome,
        message: Some(message),
        metrics: SolveMetrics {
            elapsed: started.elapsed(),
            probes: None,
            ..SolveMetrics::default()
        },
        stages: Vec::new(),
    }
}

/// Solves `start` to a `target` that constrains the centers only: moves after
/// which every center shows its target color. `engine` is the solver of the
/// cube's size. The answer is replayed against the target before it is given.
pub fn solve_to(
    engine: &dyn Solver,
    start: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> SolveResult {
    let started = Instant::now();
    let goal = match CenterGoal::read(start, target) {
        Ok(goal) => goal,
        Err(message) => return result(Err(Failure::InvalidInput), message, started),
    };
    let kind = start.kind();
    if target.matches(start) {
        return result(
            Ok(MoveSequence::new(Vec::new())),
            "shortest: 0 moves, the centers are already as the target says".into(),
            started,
        );
    }
    let model = Model::new(kind);
    // The exact search first: the answer is proven shortest when it is found.
    if let Some(found) = model.shortest(start, goal.aim, EXACT_DEPTH, EXACT_NODES) {
        if target.reached_by(start, &found) {
            let len = found.len();
            return result(
                Ok(found),
                format!("shortest: {len} moves, proved by an exact search over the layer moves of the centers"),
                started,
            );
        }
    }
    let reached = |state: &Facelets| target.matches(state);
    let over = || refine::out_of_time(started, options.time_budget);
    // The engine's centers stage.
    let mut asked = SolveOptions::default()
        .with_slices(options.slices)
        .with_centers_only(true);
    if let Some(budget) = options.time_budget {
        match budget
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
        {
            Some(left) => asked = asked.with_time_budget(left),
            None => {
                return result(
                    Err(Failure::BudgetExhausted),
                    "the time budget ran out before the engine started".into(),
                    started,
                )
            }
        }
    }
    let staged = engine.solve(start, &asked);
    let moves = match staged.outcome {
        Ok(moves) => moves,
        Err(failure) => {
            let message = staged
                .message
                .unwrap_or_else(|| format!("the engine's center stage failed ({failure:?})"));
            return result(Err(failure), message, started);
        }
    };
    // The shortest prefix that meets the target; else the first with uniform
    // centers (or the whole stage), finished in the frame of the target.
    let mut states = vec![start.clone()];
    let mut uniform_at = None;
    let mut met = false;
    for mv in moves.moves() {
        let next = match states[states.len() - 1].apply(&MoveSequence::new(vec![*mv])) {
            Ok(next) => next,
            Err(error) => return result(Err(Failure::Internal), error.to_string(), started),
        };
        let done = reached(&next);
        if uniform_at.is_none() && uniform(&next) {
            uniform_at = Some(states.len());
        }
        states.push(next);
        if done {
            met = true;
            break;
        }
    }
    let mut path: Vec<_> = moves.moves()[..states.len() - 1].to_vec();
    if !met {
        let from = uniform_at.unwrap_or(states.len() - 1);
        path.truncate(from);
        states.truncate(from + 1);
        let state = &states[from];
        let finish = model
            .shortest(state, goal.aim, FRAME_DEPTH, FRAME_NODES)
            .or_else(|| model.beam(state, goal.aim, FRAME_BEAM, 2 * FRAME_DEPTH));
        let Some(finish) = finish else {
            return result(
                Err(Failure::Internal),
                "the engine's centers are not in the frame of the target and no short turn puts them there".into(),
                started,
            );
        };
        for mv in finish.moves() {
            let next = states[states.len() - 1]
                .apply(&MoveSequence::new(vec![*mv]))
                .expect("a model move fits the cube");
            states.push(next);
            path.push(*mv);
        }
    }
    if !reached(&states[states.len() - 1]) {
        return result(
            Err(Failure::Internal),
            "the engine's center stage does not meet the target".into(),
            started,
        );
    }
    // The beam shortens the end of the engine's moves: by half on the 4x4x4,
    // by a seventh on the 5x5x5. On the 6x6x6 and 7x7x7 it gains one or two
    // tenths for 5 s more and about 1 GB, so it is left out there.
    let shortened = kind.size() <= SHORTEN_UP_TO;
    if shortened {
        if let Some(cut) = refine::shorten(&model, &states, goal.aim, &reached, &over) {
            path.truncate(cut.keep);
            path.extend_from_slice(cut.finish.moves());
        }
    }
    let path = MoveSequence::new(normalize(kind.size(), &path, true));
    if !target.reached_by(start, &path) {
        return result(
            Err(Failure::Internal),
            "internal error: the centers answer does not meet the target".into(),
            started,
        );
    }
    let len = path.len();
    result(
        Ok(path),
        format!(
            "best found: {len} moves, the engine's center stage cut to the target{}, not proven shortest",
            if shortened {
                " and shortened by a beam search"
            } else {
                ""
            }
        ),
        started,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleKind;

    fn centers_target(kind: PuzzleKind) -> Target {
        Target::masked_by(&Facelets::solved(kind), &Mask::centers(kind))
    }

    fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
        Facelets::solved(kind)
            .apply(&moves.parse().unwrap())
            .unwrap()
    }

    #[test]
    fn the_solved_centers_read_as_the_own_aim() {
        for kind in [PuzzleKind::Cube4, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let goal = CenterGoal::read(&Facelets::solved(kind), &centers_target(kind)).unwrap();
            assert_eq!(goal.aim, Aim::OWN, "{kind:?}");
        }
    }

    #[test]
    fn a_turned_color_assignment_is_an_aim_on_even_cubes_only() {
        let kind = PuzzleKind::Cube4;
        let turned = cube(kind, "Rw Lw'");
        let target = Target::masked_by(&turned, &Mask::centers(kind));
        let goal = CenterGoal::read(&Facelets::solved(kind), &target).unwrap();
        assert_ne!(goal.aim, Aim::OWN);
        let kind = PuzzleKind::Cube5;
        let mut sets = vec![0b11_1111u8; kind.facelet_count()];
        for position in Mask::centers(kind).positions() {
            // U shows R's color and R shows U's: a face center that moved
            let face = position / kind.face_len();
            sets[position] = 1 << [1, 0, 2, 3, 4, 5][face];
        }
        let swapped = Target::from_sets(sets).unwrap();
        let error = CenterGoal::read(&Facelets::solved(kind), &swapped).unwrap_err();
        assert!(
            error.contains("unreachable") && error.contains("face center"),
            "{error}"
        );
    }

    #[test]
    fn other_masks_are_refused_with_the_supported_subset_named() {
        let kind = PuzzleKind::Cube4;
        let start = Facelets::solved(kind);
        let one_face = Target::masked(&start, |p| p < kind.face_len());
        let error = CenterGoal::read(&start, &one_face).unwrap_err();
        assert!(
            error.contains("`centers`") && error.contains("12"),
            "{error}"
        );
        let partly = Target::masked(&start, |p| p == 5);
        let error = CenterGoal::read(&start, &partly).unwrap_err();
        assert!(error.contains("must all be specified"), "{error}");
        let error = CenterGoal::read(
            &Facelets::solved(PuzzleKind::Cube3),
            &centers_target(PuzzleKind::Cube3),
        )
        .unwrap_err();
        assert!(error.contains("4x4x4 to 7x7x7"), "{error}");
    }

    #[test]
    fn mixed_centers_on_a_face_are_refused() {
        let kind = PuzzleKind::Cube4;
        let mixed = Target::masked_by(&cube(kind, "2R"), &Mask::centers(kind));
        let error = CenterGoal::read(&Facelets::solved(kind), &mixed).unwrap_err();
        assert!(error.contains("different colors"), "{error}");
    }

    #[test]
    fn a_color_on_two_faces_is_contradictory() {
        let kind = PuzzleKind::Cube4;
        let mut sets = vec![0b11_1111u8; kind.facelet_count()];
        for position in Mask::centers(kind).positions() {
            sets[position] = 1 << (position / kind.face_len()).min(4);
        }
        let target = Target::from_sets(sets).unwrap();
        let error = CenterGoal::read(&Facelets::solved(kind), &target).unwrap_err();
        assert!(error.contains("contradictory"), "{error}");
    }
}
