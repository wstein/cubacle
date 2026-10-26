//! The exact route: a breadth-first search over the whole edges a target names.

use std::time::Instant;

use super::super::cubie::Cubie;
use super::finish;
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveOptions, SolveResult, Turn,
};

/// How each of the 18 turns moves an edge: piece at slot `p` goes to
/// `to[p]` and its flip changes by `flip[p]`.
struct EdgeMove {
    to: [u8; 12],
    flip: [u8; 12],
}

fn edge_moves() -> Vec<(Face, Turn, EdgeMove)> {
    let mut out = Vec::new();
    for face in Face::ALL {
        for (turn, text) in [
            (Turn::Clockwise, ""),
            (Turn::Half, "2"),
            (Turn::CounterClockwise, "'"),
        ] {
            let moves: MoveSequence = format!("{}{text}", face.letter())
                .parse()
                .expect("a face turn");
            let turned = Facelets::solved(PuzzleKind::Cube3)
                .apply(&moves)
                .expect("a face turn fits");
            let mut cube = Cubie::new();
            assert_eq!(cube.set_from_facelet(&turned.to_string()), 0);
            let mut mv = EdgeMove {
                to: [0; 12],
                flip: [0; 12],
            };
            for slot in 0..12 {
                // slot now holds piece ea>>1 of the old position
                let from = usize::from(cube.ea[slot] >> 1);
                mv.to[from] = slot as u8;
                mv.flip[from] = cube.ea[slot] & 1;
            }
            out.push((face, turn, mv));
        }
    }
    out
}

/// Positions and flips of `k` tracked edges as one table index.
fn encode(slots: &[u8], flips: &[u8]) -> usize {
    let mut index = 0;
    for &slot in slots.iter().rev() {
        index = index * 12 + usize::from(slot);
    }
    for &flip in flips.iter().rev() {
        index = index * 2 + usize::from(flip);
    }
    index
}

/// The table index after one turn; no allocation, for the inner loop of the search.
fn step(index: usize, k: usize, mv: &EdgeMove) -> usize {
    let mut slots = [0u8; super::MAX_EDGES];
    let mut flips = [0u8; super::MAX_EDGES];
    let mut rest = index;
    for flip in flips.iter_mut().take(k) {
        *flip = (rest % 2) as u8;
        rest /= 2;
    }
    for slot in slots.iter_mut().take(k) {
        *slot = (rest % 12) as u8;
        rest /= 12;
    }
    for i in 0..k {
        let from = usize::from(slots[i]);
        slots[i] = mv.to[from];
        flips[i] ^= mv.flip[from];
    }
    encode(&slots[..k], &flips[..k])
}

pub(super) fn edges_only(
    started: Instant,
    cube: &Cubie,
    wanted: &[(usize, usize, u8)],
    options: &SolveOptions,
) -> SolveResult {
    let k = wanted.len();
    if k == 0 {
        return finish(
            started,
            Ok(MoveSequence::new(Vec::new())),
            "shortest: 0 moves, the target constrains nothing a move can change".into(),
        );
    }
    let moves = edge_moves();
    let size = 12usize.pow(k as u32) * (1 << k);
    let goal_slots: Vec<u8> = wanted.iter().map(|w| w.0 as u8).collect();
    let goal_flips: Vec<u8> = wanted.iter().map(|w| w.2).collect();
    let goal = encode(&goal_slots, &goal_flips);
    let mut dist = vec![u8::MAX; size];
    dist[goal] = 0;
    let mut frontier = vec![goal as u32];
    let mut depth = 0u8;
    while !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for &state in &frontier {
            for (_, _, mv) in &moves {
                let to = step(state as usize, k, mv);
                if dist[to] == u8::MAX {
                    dist[to] = depth;
                    next.push(to as u32);
                }
            }
        }
        frontier = next;
    }
    // Where the start has the tracked pieces.
    let mut slots = Vec::with_capacity(k);
    let mut flips = Vec::with_capacity(k);
    for &(_, piece, _) in wanted {
        let slot = (0..12)
            .find(|&s| usize::from(cube.ea[s] >> 1) == piece)
            .expect("a verified cube has every edge");
        slots.push(slot as u8);
        flips.push(cube.ea[slot] & 1);
    }
    let mut state = encode(&slots, &flips);
    let length = usize::from(dist[state]);
    if options
        .max_moves
        .is_some_and(|max| length > usize::from(max))
    {
        return finish(
            started,
            Err(Failure::Exhausted),
            format!("the shortest solution has {length} moves, over the limit"),
        );
    }
    let mut solution = Vec::with_capacity(length);
    while dist[state] > 0 {
        let want = dist[state] - 1;
        let (face, turn, mv) = moves
            .iter()
            .find(|(_, _, mv)| dist[step(state, k, mv)] == want)
            .expect("every state off the goal has a neighbor one closer");
        solution.push(Move::new(*face, 1, false, *turn));
        state = step(state, k, mv);
    }
    finish(
        started,
        Ok(solution.into_iter().collect()),
        format!("shortest: proved by an exact search over the {k} edges the target names"),
    )
}
