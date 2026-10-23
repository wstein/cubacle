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

/// The fingerprint a table is keyed by; the low three bits are free.
pub(super) fn key(state: &[u8]) -> u64 {
    hash(state, 0) & !7
}

/// Depth-first walk over move sequences up to `max` long, never repeating or
/// cancelling the previous move.
struct Walker<'a> {
    frame: &'a Frame<'a>,
    buffers: Vec<State>,
    path: Vec<MoveId>,
    max: usize,
    ticks: u32,
}

impl<'a> Walker<'a> {
    fn new(frame: &'a Frame<'a>, from: &[u8], max: usize) -> Walker<'a> {
        Walker {
            frame,
            buffers: vec![from.to_vec(); max + 1],
            path: Vec::new(),
            max,
            ticks: 0,
        }
    }

    /// Calls `visit` with the cube and the moves so far at every node; it
    /// returns true to stop. Also stops when the time is up.
    fn run(&mut self, visit: &mut dyn FnMut(&[u8], &[MoveId]) -> bool) -> bool {
        self.ticks += 1;
        if self.ticks % 8192 == 0 && self.frame.expired() {
            return true;
        }
        let level = self.path.len();
        if visit(&self.buffers[level], &self.path) {
            return true;
        }
        if level == self.max {
            return false;
        }
        for id in 0..self.frame.perms.len() {
            if let Some(&last) = self.path.last() {
                if id == last || id == self.frame.perms.inv(last) {
                    continue;
                }
            }
            let (done, rest) = self.buffers.split_at_mut(level + 1);
            self.frame.perms.apply_into(&mut rest[0], &done[level], id);
            self.path.push(id);
            let stop = self.run(visit);
            self.path.pop();
            if stop {
                return true;
            }
        }
        false
    }
}

/// The keys of all cubes within `depth` moves of `from`, the number of moves
/// in the low bits, sorted.
pub(super) fn near_table(frame: &Frame, from: &[u8], depth: usize) -> Vec<u64> {
    let mut table = Vec::new();
    Walker::new(frame, from, depth).run(&mut |state, path| {
        table.push(key(state) | path.len() as u64);
        false
    });
    table.sort_unstable();
    table
}

/// The move counts at which `table` holds the cube with this key.
pub(super) fn lookup(table: &[u64], key: u64) -> impl Iterator<Item = usize> + '_ {
    let lo = table.partition_point(|&e| e < key);
    table[lo..]
        .iter()
        .take_while(move |&&e| e & !7 == key)
        .map(|&e| (e & 7) as usize)
}

/// A sequence of exactly `len` moves from `from` to the cube with this key.
pub(super) fn find_path(frame: &Frame, from: &[u8], len: usize, wanted: u64) -> Vec<MoveId> {
    let mut found = Vec::new();
    Walker::new(frame, from, len).run(&mut |state, path| {
        let hit = path.len() == len && key(state) == wanted;
        if hit {
            found = path.to_vec();
        }
        hit
    });
    found
}

/// Finds the cheapest way back onto the path within `depth` moves: fewest
/// moves to do plus printed moves left. `None` when none is that close.
pub(super) fn search(frame: &Frame, depth: usize) -> Option<Fix> {
    let n = frame.printed.len();
    let (a_depth, b_depth) = (depth.div_ceil(2), depth / 2);
    let near = near_table(frame, &frame.current, a_depth);
    // (moves in all, moves to do, position, fingerprint, moves on each side)
    let mut best: Option<(usize, usize, usize, u64, usize, usize)> = None;
    for q in 0..=n {
        if frame.expired() {
            break;
        }
        Walker::new(frame, &frame.trajectory[q], b_depth).run(&mut |state, path| {
            let wanted = key(state);
            for a in lookup(&near, wanted) {
                let todo = a + path.len();
                let candidate = (todo + n - q, todo, q, wanted, a, path.len());
                if best.is_none_or(|b| candidate < b) {
                    best = Some(candidate);
                }
            }
            false
        });
    }
    let (_, _, q, wanted, a, b) = best?;
    let mut moves = find_path(frame, &frame.current, a, wanted);
    let second = find_path(frame, &frame.trajectory[q], b, wanted);
    moves.extend(frame.perms.undo_of(&second));
    let reached = frame.perms.apply_all(&frame.current, &moves);
    (reached == frame.trajectory[q]).then_some(Fix { moves, done: q })
}
