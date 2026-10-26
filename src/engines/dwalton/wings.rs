//! Pairing the wings of the 6x6x6 through the 4x4x4 and 5x5x5 engines.
//!
//! New code; the stages are those of the reference's `reduce_555` tail and
//! `group_edges` (see `fake`).

use std::sync::Arc;
use std::time::Duration;

use super::fake::{five_of, four_of, lift, three_of};
use crate::puzzle::{Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveOptions, Solver};

/// Probes of the 3x3x3 stage for each probe asked: measured (see
/// `tools/bench/probes_big.rs`), the 3x3x3 stage gains most of the shorter
/// answers, and a thousand per probe is where the gain flattens.
const TAIL_PROBES: u64 = 1000;

/// What a stage may spend: a time budget and a probe count (the number of
/// reductions a wing stage keeps, the probe minimum of the 3x3x3 stage).
pub(super) type Stint = (Option<Duration>, Option<u64>);

/// What went wrong when a stage handed to an engine.
#[derive(Debug)]
pub(super) struct Refused {
    pub stage: &'static str,
    pub failure: Failure,
    pub message: Option<String>,
}

/// The options of a stage that may take `budget` at most.
fn limited(options: SolveOptions, budget: Option<Duration>) -> SolveOptions {
    match budget {
        Some(budget) => options.with_time_budget(budget),
        None => options,
    }
}

/// `options` asking for `probes` probes, if any.
fn probing(options: SolveOptions, probes: Option<u64>) -> SolveOptions {
    match probes {
        Some(n) => options.with_min_probes(n),
        None => options,
    }
}

/// The moves of a wing stage and the ends of the engine's own stages inside
/// them, as counts of those moves (the last is the length).
pub(super) struct Staged {
    pub moves: Vec<Move>,
    pub ends: Vec<usize>,
}

fn reduce_with(
    engine: &Arc<dyn Solver + Send + Sync>,
    fake: &Facelets,
    stage: &'static str,
    (budget, probes): Stint,
) -> Result<(Vec<Move>, Vec<usize>), Refused> {
    let options = probing(
        limited(SolveOptions::default().with_reduce_only(true), budget),
        probes,
    );
    let result = engine.solve(fake, &options);
    let ends: Vec<usize> = result.stages.iter().map(|s| s.end).collect();
    match result.outcome {
        Ok(seq) => Ok((seq.moves().to_vec(), ends)),
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

/// The moves `moves` lifted by `depth`, with the stage `ends` of the originals
/// counted in the lifted moves.
fn lifted(moves: &[Move], ends: &[usize], depth: u8) -> Staged {
    let mut out = Vec::new();
    let mut at = vec![0];
    for m in moves {
        out.extend(lift(m, depth));
        at.push(out.len());
    }
    Staged {
        ends: ends.iter().map(|&e| at[e.min(moves.len())]).collect(),
        moves: out,
    }
}

/// Pairs the inner wing orbit of an even cube; returns the moves.
pub(super) fn inner_wings(
    four: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    stint: Stint,
) -> Result<Staged, Refused> {
    let n = cube.kind().size();
    let (moves, ends) = reduce_with(four, &four_of(cube), "inner wings (4x4x4)", stint)?;
    let depth = (n / 2) as u8;
    let staged = lifted(&moves, &ends, depth);
    // The 4x4x4 turns whole-cube-wide layers as 4Xw; none are expected.
    debug_assert!(staged.moves.iter().all(|m| m.fits(cube.kind())));
    Ok(staged)
}

/// Pairs the wings of `orbit` through a 5x5x5; returns the moves.
pub(super) fn orbit_wings(
    five: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    orbit: usize,
    stint: Stint,
) -> Result<Staged, Refused> {
    let (moves, ends) = reduce_with(five, &five_of(cube, orbit), "outer wings (5x5x5)", stint)?;
    Ok(lifted(&moves, &ends, (orbit + 2) as u8))
}

/// Solves the reduced cube as a 3x3x3; returns the moves.
pub(super) fn finish(
    three: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    (budget, probes): Stint,
) -> Result<Staged, Refused> {
    debug_assert!(matches!(cube.kind(), PuzzleKind::Cube6 | PuzzleKind::Cube7));
    let options = probing(
        limited(SolveOptions::default(), budget),
        probes.map(|n| n * TAIL_PROBES),
    );
    let result = three.solve(&three_of(cube), &options);
    match result.outcome {
        Ok(seq) => Ok(Staged {
            ends: vec![seq.len()],
            moves: seq.moves().to_vec(),
        }),
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
