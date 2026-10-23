//! The shortest few moves from the current cube back onto the printed path.
//!
//! Meet in the middle: all cubes within `a` moves of the current one, against
//! all cubes within `b` moves of each cube of the trajectory.

use super::search::Frame;
use super::state::{hash, MoveId, State};

/// Moves that bring the current cube to the cube after `done` printed moves.
pub(super) struct Fix {
    pub(super) moves: Vec<MoveId>,
    pub(super) done: usize,
}

/// Depth-first walk over move sequences up to `max` long, never repeating or
/// cancelling the previous move. `visit` gets the cube and the moves so far
/// and returns true to stop.
fn walk(
    frame: &Frame,
    buffers: &mut [State],
    path: &mut Vec<MoveId>,
    max: usize,
    visit: &mut dyn FnMut(&[u8], &[MoveId]) -> bool,
) -> bool {
    let level = path.len();
    if visit(&buffers[level], path) {
        return true;
    }
    if level == max {
        return false;
    }
    for id in 0..frame.perms.len() {
        if let Some(&last) = path.last() {
            if id == last || id == frame.perms.inv(last) {
                continue;
            }
        }
        let (done, rest) = buffers.split_at_mut(level + 1);
        frame.perms.apply_into(&mut rest[0], &done[level], id);
        path.push(id);
        let stop = walk(frame, buffers, path, max, visit);
        path.pop();
        if stop {
            return true;
        }
    }
    false
}

fn buffers_from(state: &[u8], depth: usize) -> Vec<State> {
    vec![state.to_vec(); depth + 1]
}

/// Finds the cheapest way back onto the path within `depth` moves: fewest
/// moves to do now plus printed moves left. `None` when none is that close.
pub(super) fn search(frame: &Frame, depth: usize) -> Option<Fix> {
    let n = frame.printed.len();
    let (a_depth, b_depth) = (depth.div_ceil(2), depth / 2);
    // Low three bits hold the number of moves.
    let mut near: Vec<u64> = Vec::new();
    walk(
        frame,
        &mut buffers_from(&frame.current, a_depth),
        &mut Vec::new(),
        a_depth,
        &mut |state, path| {
            near.push((hash(state, 0) & !7) | path.len() as u64);
            false
        },
    );
    near.sort_unstable();
    // (moves left in all, moves to do, position, fingerprint, moves on each side)
    let mut best: Option<(usize, usize, usize, u64, usize, usize)> = None;
    for q in 0..=n {
        if frame.expired() {
            break;
        }
        walk(
            frame,
            &mut buffers_from(&frame.trajectory[q], b_depth),
            &mut Vec::new(),
            b_depth,
            &mut |state, path| {
                let key = hash(state, 0) & !7;
                let lo = near.partition_point(|&e| e < key);
                for &e in near[lo..].iter().take_while(|&&e| e & !7 == key) {
                    let a = (e & 7) as usize;
                    let todo = a + path.len();
                    let candidate = (todo + n - q, todo, q, key, a, path.len());
                    if best.is_none_or(|b| candidate < b) {
                        best = Some(candidate);
                    }
                }
                false
            },
        );
    }
    let (_, _, q, key, a, b) = best?;
    // Rebuild the two halves.
    let mut first = Vec::new();
    walk(
        frame,
        &mut buffers_from(&frame.current, a),
        &mut Vec::new(),
        a,
        &mut |state, path| {
            let found = path.len() == a && hash(state, 0) & !7 == key;
            if found {
                first = path.to_vec();
            }
            found
        },
    );
    let mut second = Vec::new();
    walk(
        frame,
        &mut buffers_from(&frame.trajectory[q], b),
        &mut Vec::new(),
        b,
        &mut |state, path| {
            let found = path.len() == b && hash(state, 0) & !7 == key;
            if found {
                second = path.to_vec();
            }
            found
        },
    );
    let mut moves = first;
    moves.extend(frame.perms.undo_of(&second));
    let reached = frame.perms.apply_all(&frame.current, &moves);
    (reached == frame.trajectory[q]).then_some(Fix { moves, done: q })
}
