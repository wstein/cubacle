//! Search spaces shared by the phases.
//!
//! The pruning tables over two coordinates move both at once; this is the
//! `PruningTable(Move1, Move2, ...)` of the Java solver.
//!
//! Reorganised from cs0x7f/cube555 src/Util.java (the move-table parts of
//! `PruningTable`), MIT, Copyright (c) 2023 Chen Shuang; this file is
//! AGPL-3.0-or-later, see NOTICE.md.

use crate::reduction::bfs::Space;

/// Pairs `(a, b)` of states, state number `a * n2 + b`, moved together by
/// move tables `a_move` (`n1` rows) and `b_move` (`n2` rows) of `moves`
/// columns each.
pub(super) struct Pair<'a> {
    pub a_move: &'a [u32],
    pub b_move: &'a [u32],
    pub moves: usize,
    pub n1: usize,
    pub n2: usize,
    pub a_solved: &'a [usize],
    pub b_solved: &'a [usize],
}

impl Space for Pair<'_> {
    fn size(&self) -> usize {
        self.n1 * self.n2
    }

    fn solved(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.a_solved.len() * self.b_solved.len());
        for &a in self.a_solved {
            for &b in self.b_solved {
                out.push(a * self.n2 + b);
            }
        }
        out
    }

    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        let (a, b) = (state / self.n2, state % self.n2);
        let (ra, rb) = (a * self.moves, b * self.moves);
        for m in 0..self.moves {
            f(
                self.a_move[ra + m] as usize * self.n2 + self.b_move[rb + m] as usize,
                0,
            );
        }
    }
}

/// States with plain moves of one table, goal `solved`.
pub(super) struct Plain<'a> {
    pub table: &'a [u32],
    pub moves: usize,
    pub solved: &'a [usize],
}

impl Space for Plain<'_> {
    fn size(&self) -> usize {
        self.table.len() / self.moves
    }

    fn solved(&self) -> Vec<usize> {
        self.solved.to_vec()
    }

    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        for &next in &self.table[state * self.moves..(state + 1) * self.moves] {
            f(next as usize, 0);
        }
    }
}

/// Classes of one coordinate by a raw coordinate that moves with them: state
/// `class * raw_count + raw`. A class move gives `class << bits | symmetry`,
/// and the raw coordinate is moved by `raw_move` and then conjugated by the
/// symmetry through `raw_conj` (8 columns).
pub(super) struct ClassTable<'a> {
    pub class_move: &'a [u32],
    pub self_sym: &'a [u32],
    pub moves: usize,
    /// Bits of the symmetry in a class move (2 for 4 symmetries, 3 for 8).
    pub bits: u32,
    pub raw_count: usize,
    pub raw_move: &'a [u32],
    pub raw_conj: &'a [u32],
    pub solved: Vec<usize>,
}

impl Space for ClassTable<'_> {
    fn size(&self) -> usize {
        self.self_sym.len() * self.raw_count
    }

    fn solved(&self) -> Vec<usize> {
        self.solved.clone()
    }

    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        let (class, raw) = (state / self.raw_count, state % self.raw_count);
        let mask = (1usize << self.bits) - 1;
        for m in 0..self.moves {
            let new = self.class_move[class * self.moves + m] as usize;
            let moved = self.raw_move[raw * self.moves + m] as usize;
            let moved = self.raw_conj[moved * 8 + (new & mask)] as usize;
            f((new >> self.bits) * self.raw_count + moved, 0);
        }
    }

    fn images<F: FnMut(usize)>(&self, target: usize, _tag: u32, f: &mut F) {
        let (class, raw) = (target / self.raw_count, target % self.raw_count);
        let mut state = self.self_sym[class];
        let mut j = 1;
        loop {
            state >>= 1;
            if state == 0 {
                return;
            }
            if state & 1 == 1 {
                f(class * self.raw_count + self.raw_conj[raw * 8 + j] as usize);
            }
            j += 1;
        }
    }
}
