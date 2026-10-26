//! Partial targets on the 3x3x3.
//!
//! Two routes, told apart by what the target constrains:
//!
//! - **Edges only** (the cross, any few edges): the target names up to five
//!   whole edges, each with both stickers fixed. Only those pieces matter, so
//!   the state is where they are and how they are flipped (12P5 * 2^5 at the
//!   most) and a breadth-first search from the goal gives exact distances.
//!   The answer is proven shortest.
//! - **Few completions**: the target is enumerated into the complete cubes it
//!   allows. If there are at most [`MAX_COMPLETIONS`], the two-phase search
//!   solves the way to each and the shortest answer is kept. That answer is
//!   the best found, not proven shortest, as for a plain `solve`.
//!
//! Anything else says so and names what is supported.

use std::time::Instant;

use super::cubie::{Cubie, EDGE_FACELET};
use super::solver::Min2PhaseSolver;
use crate::puzzle::{
    Face, Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult,
    Target,
};
use completions::by_completions;
use edges::edges_only;

/// The most whole edges the exact search tracks: 12P5 * 2^5 = 7,962,624 table entries.
const MAX_EDGES: usize = 5;
/// The most complete cubes a target may allow for the second route.
const MAX_COMPLETIONS: usize = 24;
/// Search nodes after which the enumeration of completions gives up.
const MAX_NODES: usize = 200_000;
const ANY: u8 = 0b11_1111;

fn finish(
    started: Instant,
    outcome: Result<MoveSequence, Failure>,
    message: String,
) -> SolveResult {
    SolveResult {
        outcome,
        message: Some(message),
        metrics: SolveMetrics {
            elapsed: started.elapsed(),
            probes: None,
        },
        stages: Vec::new(),
    }
}

/// The color (face index) of a facelet position.
fn color(facelet: u8) -> usize {
    usize::from(facelet) / 9
}

/// The single color a sticker must show, if the target fixes it.
fn fixed(set: u8) -> Option<usize> {
    (set.count_ones() == 1).then(|| set.trailing_zeros() as usize)
}

pub(super) fn solve_to(
    engine: &Min2PhaseSolver,
    start: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> SolveResult {
    let started = Instant::now();
    let fail = |failure, message: &str| finish(started, Err(failure), message.to_owned());
    if start.kind() != PuzzleKind::Cube3 || target.kind() != PuzzleKind::Cube3 {
        return fail(Failure::InvalidInput, "expected a 3x3x3 start and target");
    }
    let mut cube = Cubie::new();
    if cube.set_from_facelet(&start.to_string()) != 0 || cube.verify() != 0 {
        return fail(Failure::InvalidInput, "the start cube state is impossible");
    }
    let sets = target.sets();
    // The centers never move: a target that wants another color there cannot be met.
    for face in Face::ALL {
        if sets[face.index() * 9 + 4] >> face.index() & 1 == 0 {
            return fail(
                Failure::Exhausted,
                "unreachable: the target wants another color on a center, and the centers of a 3x3x3 never move",
            );
        }
    }
    match whole_edges(sets) {
        Ok(Some(wanted)) if wanted.len() <= MAX_EDGES => {
            return edges_only(started, &cube, &wanted, options)
        }
        Ok(_) => {}
        Err(message) => return fail(Failure::Exhausted, &message),
    }
    by_completions(engine, started, start, target, options)
}

/// Slot, piece and flip of every whole edge a target fixes.
type Wanted = Vec<(usize, usize, u8)>;

/// The edges the target fixes, as (slot, piece, flip), when it constrains
/// nothing but whole edges and centers; `Ok(None)` when it constrains other
/// stickers (corners, or half an edge); `Err` when a fixed edge is no edge.
fn whole_edges(sets: &[u8]) -> Result<Option<Wanted>, String> {
    let on_edge = |facelet: u8| EDGE_FACELET.iter().any(|pair| pair.contains(&facelet));
    for (position, &set) in sets.iter().enumerate() {
        let center = position % 9 == 4;
        if set != ANY && !center && !on_edge(position as u8) {
            return Ok(None);
        }
    }
    let mut wanted = Vec::new();
    for (slot, pair) in EDGE_FACELET.iter().enumerate() {
        let (a, b) = (sets[usize::from(pair[0])], sets[usize::from(pair[1])]);
        if a == ANY && b == ANY {
            continue;
        }
        let (Some(a), Some(b)) = (fixed(a), fixed(b)) else {
            return Ok(None);
        };
        let piece = EDGE_FACELET.iter().position(|home| {
            (color(home[0]), color(home[1])) == (a, b) || (color(home[1]), color(home[0])) == (a, b)
        });
        let Some(piece) = piece else {
            return Err(format!(
                "contradictory: no edge of the cube has the colors {} and {}",
                Face::ALL[a].letter(),
                Face::ALL[b].letter()
            ));
        };
        let flip =
            u8::from((color(EDGE_FACELET[piece][0]), color(EDGE_FACELET[piece][1])) != (a, b));
        if wanted.iter().any(|&(_, other, _)| other == piece) {
            return Err(format!(
                "contradictory: the target wants the edge {}{} in two places",
                Face::ALL[a].letter(),
                Face::ALL[b].letter()
            ));
        }
        wanted.push((slot, piece, flip));
    }
    Ok(Some(wanted))
}

mod completions;
mod edges;
#[cfg(test)]
mod tests;
