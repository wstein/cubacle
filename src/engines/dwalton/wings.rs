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

/// The moves of a reduction and the ends of the engine's own stages inside.
type Reduction = (Vec<Move>, Vec<usize>);

/// Up to `n` reductions of a wing stage through `engine`, the plain one first,
/// each with the ends of the engine's own stages.
fn reduce_many_with(
    engine: &Arc<dyn Solver + Send + Sync>,
    fake: &Facelets,
    stage: &'static str,
    n: usize,
    budget: Option<Duration>,
) -> Result<Vec<Reduction>, Refused> {
    let options = limited(SolveOptions::default().with_reduce_only(true), budget);
    let results = engine.solve_many(fake, &options, n);
    let mut listed = Vec::new();
    for result in results {
        let ends: Vec<usize> = result.stages.iter().map(|s| s.end).collect();
        match result.outcome {
            Ok(seq) => listed.push((seq.moves().to_vec(), ends)),
            Err(failure) if listed.is_empty() => {
                return Err(Refused {
                    stage,
                    failure,
                    message: result.message,
                })
            }
            Err(_) => {}
        }
    }
    Ok(listed)
}

fn apply(cube: &Facelets, moves: &[Move]) -> Facelets {
    cube.apply(&MoveSequence::new(moves.to_vec()))
        .expect("the lifted moves fit the cube")
}

/// The moves `moves` lifted by `depth`, with the stage `ends` of the originals
/// counted in the lifted moves.
pub(super) fn lifted(moves: &[Move], ends: &[usize], depth: u8) -> Staged {
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

/// Up to `n` ways to pair the inner wings of an even cube, the plain one first.
pub(super) fn inner_wings_many(
    four: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    n: usize,
    budget: Option<Duration>,
) -> Result<Vec<Staged>, Refused> {
    let depth = (cube.kind().size() / 2) as u8;
    let listed = reduce_many_with(four, &four_of(cube), "inner wings (4x4x4)", n, budget)?;
    Ok(listed.iter().map(|(m, e)| lifted(m, e, depth)).collect())
}

/// Up to `n` ways to pair the wings of `orbit` through a 5x5x5.
pub(super) fn orbit_wings_many(
    five: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    orbit: usize,
    n: usize,
    budget: Option<Duration>,
) -> Result<Vec<Staged>, Refused> {
    let fake = five_of(cube, orbit);
    let listed = reduce_many_with(five, &fake, "outer wings (5x5x5)", n, budget)?;
    Ok(listed
        .iter()
        .map(|(m, e)| lifted(m, e, (orbit + 2) as u8))
        .collect())
}

/// Up to `n` solutions of the reduced cube as a 3x3x3, the plain one first.
pub(super) fn finish_many(
    three: &Arc<dyn Solver + Send + Sync>,
    cube: &Facelets,
    n: usize,
    budget: Option<Duration>,
) -> Result<Vec<Staged>, Refused> {
    let options = limited(SolveOptions::default(), budget);
    let mut listed = Vec::new();
    for result in three.solve_many(&three_of(cube), &options, n) {
        match result.outcome {
            Ok(seq) => listed.push(Staged {
                ends: vec![seq.len()],
                moves: seq.moves().to_vec(),
            }),
            Err(failure) if listed.is_empty() => {
                return Err(Refused {
                    stage: "3x3x3",
                    failure,
                    message: result.message,
                })
            }
            Err(_) => {}
        }
    }
    Ok(listed)
}
