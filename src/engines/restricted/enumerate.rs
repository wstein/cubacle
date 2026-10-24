//! Listing solutions instead of finding one.
//!
//! The shortest length is found first, by the search of [`super::ida`]. Then
//! a depth-bounded search lists every solution of that length, and of each
//! longer length up to a limit, one length after the other (iterative
//! deepening on the length), so the shortest come first. Solutions stream to
//! the caller as they are found.
//!
//! Which solutions count as different follows the alphabet: pairs of moves
//! that make the identity or one move are never listed (a list full of
//! `R R'` padding helps no one), and of moves that commute only one order is
//! listed, unless the caller asks for every order (`raw`). A solution never
//! passes through the solved cube before its last move.
//!
//! The search stops at once when the deadline passes or the caller says so,
//! and checks both at every node.

use std::time::Instant;

use super::letters::{apply, Letters, State};
use super::table::{Bound, Table};

/// What to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumerateRequest {
    /// The longest solution to list; none means the shortest length only.
    pub longest: Option<u8>,
    /// Stop after this many solutions.
    pub count: Option<usize>,
    /// List every order of commuting moves.
    pub raw: bool,
}

/// Why the listing ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumerateEnd {
    /// Every solution up to the limit was listed.
    Complete,
    /// The requested number of solutions was listed.
    CountReached,
    /// The deadline passed.
    TimedOut,
    /// The caller stopped the search.
    Cancelled,
}

pub(super) struct Lister<'a> {
    pub(super) letters: &'a Letters,
    pub(super) table: &'a Table,
    pub(super) deadline: Option<Instant>,
    pub(super) cancel: &'a dyn Fn() -> bool,
    pub(super) raw: bool,
    pub(super) nodes: u64,
    pub(super) found: usize,
    pub(super) count: Option<usize>,
    pub(super) ended: Option<EnumerateEnd>,
    path: Vec<usize>,
}

impl<'a> Lister<'a> {
    pub(super) fn new(
        letters: &'a Letters,
        table: &'a Table,
        deadline: Option<Instant>,
        cancel: &'a dyn Fn() -> bool,
        request: &EnumerateRequest,
    ) -> Self {
        Lister {
            letters,
            table,
            deadline,
            cancel,
            raw: request.raw,
            nodes: 0,
            found: 0,
            count: request.count,
            ended: None,
            path: Vec::new(),
        }
    }

    /// Lists the solutions of exactly `length` letters. `emit` gets the
    /// letters of each and says whether to go on.
    pub(super) fn list(
        &mut self,
        state: &State,
        length: u8,
        emit: &mut dyn FnMut(&[usize]) -> bool,
    ) {
        self.path.clear();
        self.visit(state, 0, length, None, 0, emit);
    }

    fn visit(
        &mut self,
        state: &State,
        used: u8,
        bound: u8,
        last: Option<usize>,
        held: u64,
        emit: &mut dyn FnMut(&[usize]) -> bool,
    ) {
        self.nodes += 1;
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.ended = Some(EnumerateEnd::TimedOut);
            return;
        }
        if (self.cancel)() {
            self.ended = Some(EnumerateEnd::Cancelled);
            return;
        }
        match self.table.bound(state) {
            Bound::Unreachable => return,
            Bound::Exact(0) => {
                // Solved: a solution of this length ends here, never earlier.
                if used == bound {
                    self.found += 1;
                    let more = emit(&self.path);
                    if !more || self.count.is_some_and(|n| self.found >= n) {
                        self.ended = Some(EnumerateEnd::CountReached);
                    }
                }
                return;
            }
            Bound::Exact(d) | Bound::AtLeast(d) => {
                if u16::from(used) + u16::from(d) > u16::from(bound) {
                    return;
                }
            }
        }
        if used == bound {
            return;
        }
        let mut allowed = self.letters.allowed(last, held);
        let mut child = [0u8; super::letters::N];
        while allowed != 0 {
            let m = allowed.trailing_zeros() as usize;
            allowed &= allowed - 1;
            apply(&self.letters.letters[m].src, state, &mut child);
            self.path.push(m);
            let next_held = if self.raw {
                0
            } else {
                self.letters.step(m, held)
            };
            self.visit(&child, used + 1, bound, Some(m), next_held, emit);
            self.path.pop();
            if self.ended.is_some() {
                return;
            }
        }
    }
}
