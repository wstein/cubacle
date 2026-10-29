//! Iterative-deepening A* to a target set, with the multi-goal tables as bound.

use std::time::Instant;

use super::goal::Goal;
use super::model::{Pos, Turns};
use super::tables::{CornerTable, EdgeTable};

/// How a search ended.
pub(super) enum Ended {
    /// A shortest answer, as indices into the 18 turns.
    Found(Vec<usize>),
    /// No answer of at most this many moves exists (searched out).
    Empty(usize),
    /// The time ran out; no answer shorter than this exists.
    Stopped(usize),
}

/// The tables as one bound; either may be absent.
pub(super) struct Bound {
    pub(super) corners: Option<CornerTable>,
    pub(super) edges: Option<EdgeTable>,
}

#[derive(Clone, Copy)]
struct Coords {
    corner: (u16, u16),
    edge: u32,
}

impl Bound {
    fn coords(&self, pos: &Pos) -> Coords {
        Coords {
            corner: self
                .corners
                .as_ref()
                .map_or((0, 0), |_| CornerTable::index(pos)),
            edge: self.edges.as_ref().map_or(0, |t| t.index(pos)),
        }
    }

    #[inline]
    fn step(&self, at: Coords, m: usize) -> Coords {
        Coords {
            corner: self
                .corners
                .as_ref()
                .map_or((0, 0), |t| t.step(at.corner, m)),
            edge: self.edges.as_ref().map_or(0, |t| t.step(at.edge, m)),
        }
    }

    #[inline]
    fn value(&self, at: Coords) -> u8 {
        let c = self.corners.as_ref().map_or(0, |t| t.bound(at.corner));
        let e = self.edges.as_ref().map_or(0, |t| t.bound(at.edge));
        c.max(e)
    }
}

/// May face `g` follow face `f`? Not the same face, and of two opposite
/// faces the first of U R F goes first.
#[inline]
fn allowed(f: usize, g: usize) -> bool {
    !(g == f || (f >= 3 && g == f - 3))
}

pub(super) struct Search<'a> {
    pub(super) turns: &'a Turns,
    pub(super) goal: &'a Goal,
    pub(super) bound: &'a Bound,
    pub(super) deadline: Instant,
    pub(super) nodes: u64,
    stopped: bool,
    path: Vec<usize>,
}

impl<'a> Search<'a> {
    pub(super) fn new(
        turns: &'a Turns,
        goal: &'a Goal,
        bound: &'a Bound,
        deadline: Instant,
    ) -> Search<'a> {
        Search {
            turns,
            goal,
            bound,
            deadline,
            nodes: 0,
            stopped: false,
            path: Vec::new(),
        }
    }

    /// The first depth worth searching: the bound at the start.
    pub(super) fn start_bound(&self, start: &Pos) -> usize {
        usize::from(self.bound.value(self.bound.coords(start)))
    }

    /// Searches depth by depth from `from` up to `max` moves.
    pub(super) fn run(&mut self, start: &Pos, from: usize, max: usize) -> Ended {
        let coords = self.bound.coords(start);
        for depth in from..=max {
            self.path.clear();
            if self.dfs(start, coords, depth, usize::MAX) {
                return Ended::Found(self.path.clone());
            }
            if self.stopped {
                return Ended::Stopped(depth);
            }
        }
        Ended::Empty(max)
    }

    fn dfs(&mut self, pos: &Pos, at: Coords, left: usize, last: usize) -> bool {
        self.nodes += 1;
        if self.nodes & 0xFFFF == 0 && Instant::now() >= self.deadline {
            self.stopped = true;
            return false;
        }
        if left == 0 {
            return self.goal.is_met(pos);
        }
        for g in 0..6 {
            if !allowed(last, g) {
                continue;
            }
            for q in 0..3 {
                let m = g * 3 + q;
                let next_at = self.bound.step(at, m);
                if usize::from(self.bound.value(next_at)) >= left {
                    continue;
                }
                let next = pos.step(self.turns, m);
                self.path.push(m);
                if self.dfs(&next, next_at, left - 1, g) {
                    return true;
                }
                self.path.pop();
                if self.stopped {
                    return false;
                }
            }
        }
        false
    }
}
