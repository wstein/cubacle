//! Iterative deepening A* over the moves, bounded below by the databases.
//!
//! A move on the face just turned is never tried, and of two opposite faces
//! only one order is: `U D` but not `D U`. These rules drop every sequence
//! that another, equal one would reach.

use super::model::{tables, Cube, MoveTables, MOVES};
use super::pdb::Heuristic;

/// Why a depth-limited search ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Ended {
    Found,
    /// Every sequence within the limit was tried.
    Exhausted,
    /// The budget ran out first.
    Stopped,
}

/// Faces that may follow a move on face `f` (index 6: the first move).
const fn follow_masks() -> [u8; 7] {
    let mut masks = [0b11_1111u8; 7];
    let mut f = 0;
    while f < 6 {
        masks[f] &= !(1 << f);
        if f >= 3 {
            masks[f] &= !(1 << (f - 3));
        }
        f += 1;
    }
    masks
}

const FOLLOW: [u8; 7] = follow_masks();

pub(crate) struct Search<'a> {
    heuristic: &'a Heuristic,
    tables: &'static MoveTables,
    /// Moves of the sequence being tried.
    path: [u8; 32],
    pub(crate) nodes: u64,
    max_nodes: u64,
    stop: &'a dyn Fn() -> bool,
    stopped: bool,
}

impl<'a> Search<'a> {
    pub(crate) fn new(
        heuristic: &'a Heuristic,
        max_nodes: u64,
        stop: &'a dyn Fn() -> bool,
    ) -> Self {
        Search {
            heuristic,
            tables: tables(),
            path: [0; 32],
            nodes: 0,
            max_nodes,
            stop,
            stopped: false,
        }
    }

    /// Looks for a solution of exactly `limit` moves or fewer (the caller has
    /// shown that none is shorter than `limit`).
    pub(crate) fn run(&mut self, cube: &Cube, limit: u8) -> (Ended, Vec<u8>) {
        let found = self.dfs(cube, limit, 6, 0);
        if found {
            (Ended::Found, self.path[..usize::from(limit)].to_vec())
        } else if self.stopped {
            (Ended::Stopped, Vec::new())
        } else {
            (Ended::Exhausted, Vec::new())
        }
    }

    fn dfs(&mut self, cube: &Cube, left: u8, last: usize, depth: usize) -> bool {
        let mask = FOLLOW[last];
        for face in 0..6 {
            if mask >> face & 1 == 0 {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let child = cube.turned(self.tables, m);
                self.nodes += 1;
                if self.nodes & 0x3FFF == 0 && (self.nodes >= self.max_nodes || (self.stop)()) {
                    self.stopped = true;
                    return false;
                }
                self.path[depth] = m as u8;
                if left == 1 {
                    if child.is_solved() {
                        return true;
                    }
                    continue;
                }
                if self.heuristic.exceeds(&child, left - 1) {
                    continue;
                }
                if self.dfs(&child, left - 1, face, depth + 1) {
                    return true;
                }
                if self.stopped {
                    return false;
                }
            }
        }
        false
    }
}

const _: () = assert!(MOVES == 18);
