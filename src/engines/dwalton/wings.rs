//! Pairing the wings of the 6x6x6 through the 4x4x4 and 5x5x5 engines.
//!
//! New code; the stages are those of the reference's `reduce_555` tail and
//! `group_edges` (see `fake`).

use std::sync::Arc;

use super::fake::{five_of, four_of, lift, three_of};
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn,
};

/// What went wrong when a stage handed to an engine.
#[derive(Debug)]
pub(super) struct Refused {
    pub stage: &'static str,
    pub failure: Failure,
    pub message: Option<String>,
}

fn reduce_with(
    engine: &Arc<dyn Solver + Send + Sync>,
    fake: &Facelets,
    stage: &'static str,
) -> Result<Vec<Move>, Refused> {
    let result = engine.solve(fake, &SolveOptions::default().with_reduce_only(true));
    match result.outcome {
        Ok(seq) => Ok(seq.moves().to_vec()),
        Err(failure) => Err(Refused {
            stage,
            failure,
            message: result.message,
        }),
    }
}

fn apply(cube: &Facelets, moves: &[Move]) -> Facelets {
    cube.apply(&MoveSequence::new(moves.to_vec()))
        .expect("the lifted moves fit the cube")
}

fn centres_home(cube: &Facelets) -> bool {
    let s = cube.stickers();
    Face::ALL
        .iter()
        .enumerate()
        .all(|(f, &face)| [5, 6, 9, 10].iter().all(|&i| s[f * 16 + i] == face))
}

/// The shortest whole-cube rotation (wide turns) that puts the centres of a
/// 4x4x4 back home.
fn frame_back(cube: &Facelets) -> Option<Vec<Move>> {
    let axes = [(Face::R, Face::L), (Face::U, Face::D), (Face::F, Face::B)];
    let turns = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];
    let flip = |t: Turn| match t {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    };
    let one = |a: usize, t: Turn| {
        vec![
            Move::new(axes[a].0, 2, true, t),
            Move::new(axes[a].1, 2, true, flip(t)),
        ]
    };
    let mut candidates: Vec<Vec<Move>> = vec![Vec::new()];
    for a in 0..3 {
        for t in turns {
            candidates.push(one(a, t));
        }
    }
    for a in 0..3 {
        for b in 0..3 {
            if a != b {
                for ta in turns {
                    for tb in turns {
                        let mut both = one(a, ta);
                        both.extend(one(b, tb));
                        candidates.push(both);
                    }
                }
            }
        }
    }
    candidates.into_iter().find(|moves| {
        cube.apply(&MoveSequence::new(moves.clone()))
            .is_ok_and(|c| centres_home(&c))
    })
}

/// Pairs the inner wing orbit of an even cube; returns the moves.
pub(super) fn inner_wings(
    four: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
) -> Result<Vec<Move>, Refused> {
    let n = cube.kind().size();
    let fake = four_of(cube);
    let mut moves = reduce_with(four, &fake, "inner wings (4x4x4)")?;
    // The 4x4x4 engine may end with its centres turned as a whole; the big
    // cube's centres must stay where they are, so turn the frame back.
    let reduced = apply(&fake, &moves);
    let back = frame_back(&reduced).ok_or(Refused {
        stage: "inner wings (4x4x4)",
        failure: Failure::Internal,
        message: Some("the reduced fake 4x4x4 is not a rotation of the scheme".into()),
    })?;
    moves.extend(back);
    let depth = (n / 2) as u8;
    let lifted: Vec<Move> = moves.iter().map(|m| lift(m, depth)).collect();
    // The 4x4x4 turns whole-cube-wide layers as 4Xw; none are expected.
    debug_assert!(lifted.iter().all(|m| m.fits(cube.kind())));
    Ok(lifted)
}

/// Pairs the wings of `orbit` through a 5x5x5; returns the moves.
pub(super) fn orbit_wings(
    five: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    orbit: usize,
) -> Result<Vec<Move>, Refused> {
    let moves = reduce_with(five, &five_of(cube, orbit), "outer wings (5x5x5)")?;
    Ok(moves.iter().map(|m| lift(m, (orbit + 2) as u8)).collect())
}

/// Solves the reduced cube as a 3x3x3; returns the moves.
pub(super) fn finish(
    three: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
) -> Result<Vec<Move>, Refused> {
    debug_assert!(matches!(cube.kind(), PuzzleKind::Cube6 | PuzzleKind::Cube7));
    let result = three.solve(&three_of(cube), &SolveOptions::default());
    match result.outcome {
        Ok(seq) => Ok(seq.moves().to_vec()),
        Err(failure) => Err(Refused {
            stage: "3x3x3",
            failure,
            message: result.message,
        }),
    }
}

#[allow(unused)]
pub(super) fn replay(cube: &Facelets, moves: &[Move]) -> Facelets {
    apply(cube, moves)
}
