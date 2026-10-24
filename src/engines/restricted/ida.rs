//! Iterative deepening over the alphabet, with the exact table as the bound.
//!
//! A node's bound is its distance when the table knows the state, else one
//! more than the depth the table reaches (see [`Table`]). When a node's state
//! is in the table, its distance is exact, so the search ends there: the rest
//! of the solution is read out of the table. That makes the table half of a
//! meet in the middle, and the answer is shortest in the alphabet.
//!
//! Which letter may follow which is taken from the alphabet's own relations
//! ([`Letters`]): pairs that make the identity or one letter are skipped, and
//! of commuting letters only one order is searched.

use std::time::Instant;

use super::letters::{apply, Letters, State};
use super::table::{Bound, Table};

/// How a search ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Ended {
    /// The letters of the shortest solution, in order.
    Found(Vec<usize>),
    /// No solution within the depth given.
    NoneWithin(u8),
    /// The table is complete and the state is not in it: the alphabet cannot
    /// solve this cube.
    NotInGroup,
    /// The deadline passed.
    TimedOut,
}

pub(super) struct Ida<'a> {
    letters: &'a Letters,
    table: &'a Table,
    deadline: Option<Instant>,
    pub(super) nodes: u64,
    path: Vec<usize>,
    timed_out: bool,
}

enum Flow {
    Found,
    Continue,
}

impl<'a> Ida<'a> {
    pub(super) fn new(letters: &'a Letters, table: &'a Table, deadline: Option<Instant>) -> Self {
        Ida {
            letters,
            table,
            deadline,
            nodes: 0,
            path: Vec::new(),
            timed_out: false,
        }
    }

    /// The shortest solution of at most `max_depth` letters.
    pub(super) fn solve(&mut self, state: &State, max_depth: u8) -> Ended {
        let start = match self.table.bound(state) {
            Bound::Unreachable => return Ended::NotInGroup,
            Bound::Exact(d) => d,
            Bound::AtLeast(h) => h,
        };
        for bound in start..=max_depth {
            self.path.clear();
            match self.visit(state, 0, bound, None, 0) {
                Flow::Found => return Ended::Found(std::mem::take(&mut self.path)),
                Flow::Continue if self.timed_out => return Ended::TimedOut,
                Flow::Continue => {}
            }
        }
        Ended::NoneWithin(max_depth)
    }

    /// The letters from `state` (whose distance is `distance`) down to the
    /// solved cube, read out of the table.
    fn descend(&mut self, state: &State, mut distance: u8) {
        let mut here = *state;
        let mut child = [0u8; super::letters::N];
        while distance > 0 {
            let next = self.letters.letters.iter().position(|letter| {
                apply(&letter.src, &here, &mut child);
                self.table.get(&child) == Some(distance - 1)
            });
            let index = next.expect("a state of the table has a neighbour one step nearer");
            self.path.push(index);
            apply(&self.letters.letters[index].src, &here.clone(), &mut here);
            distance -= 1;
        }
    }

    fn visit(
        &mut self,
        state: &State,
        used: u8,
        bound: u8,
        last: Option<usize>,
        held: u64,
    ) -> Flow {
        self.nodes += 1;
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.timed_out = true;
            return Flow::Continue;
        }
        match self.table.bound(state) {
            Bound::Exact(d) => {
                if u16::from(used) + u16::from(d) > u16::from(bound) {
                    return Flow::Continue;
                }
                self.descend(state, d);
                return Flow::Found;
            }
            Bound::Unreachable => return Flow::Continue,
            Bound::AtLeast(h) => {
                if u16::from(used) + u16::from(h) > u16::from(bound) {
                    return Flow::Continue;
                }
            }
        }
        if used == bound {
            return Flow::Continue;
        }
        let mut allowed = self.letters.allowed(last, held);
        let mut child = [0u8; super::letters::N];
        while allowed != 0 {
            let m = allowed.trailing_zeros() as usize;
            allowed &= allowed - 1;
            apply(&self.letters.letters[m].src, state, &mut child);
            self.path.push(m);
            let next_held = self.letters.step(m, if last.is_some() { held } else { 0 });
            if matches!(
                self.visit(&child, used + 1, bound, Some(m), next_held),
                Flow::Found
            ) {
                return Flow::Found;
            }
            self.path.pop();
            if self.timed_out {
                return Flow::Continue;
            }
        }
        Flow::Continue
    }
}
