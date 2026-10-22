//! Pairing the wings of the 6x6x6 through the 4x4x4 and 5x5x5 engines.
//!
//! New code; the stages are those of the reference's `reduce_555` tail and
//! `group_edges` (see `fake`).

use std::sync::Arc;

use super::fake::{five_of, four_of, lift, three_of};
use crate::puzzle::{Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveOptions, Solver};

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

/// Pairs the inner wing orbit of an even cube; returns the moves.
pub(super) fn inner_wings(
    four: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
) -> Result<Vec<Move>, Refused> {
    let n = cube.kind().size();
    let moves = reduce_with(four, &four_of(cube), "inner wings (4x4x4)")?;
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
