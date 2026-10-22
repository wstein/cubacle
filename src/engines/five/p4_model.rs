//! Phase 4 pieces: the up/down and right/left centres, and the middle, low
//! and high wings of the four slots in play.
//!
//! Ported from cs0x7f/cube555 src/Phase4Center.java and src/Phase4Edge.java,
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md.

use super::mv::*;
use crate::reduction::comb::{
    copy_from_comb, copy_to_comb, get_comb, get_perm, set_comb, set_perm, swap2, swap4,
};

/// The sixteen moves of the phase.
pub(super) const MOVES: [u8; 16] = [
    U1, U2, U3, R2, F2, D1, D2, D3, L2, B2, UW2, RW2, FW2, DW2, LW2, BW2,
];

/// Up/down and right/left centres: t and x kind of each, eight slots.
pub(super) struct Centre {
    pub udt: [i32; 8],
    pub udx: [i32; 8],
    pub rlt: [i32; 8],
    pub rlx: [i32; 8],
}

impl Centre {
    pub(super) fn new() -> Centre {
        Centre {
            udt: [0; 8],
            udx: [0; 8],
            rlt: [0; 8],
            rlx: [0; 8],
        }
    }

    pub(super) fn set_ud(&mut self, idx: usize) {
        set_comb(&mut self.udx, (idx / 70) as i32, 4, 8);
        set_comb(&mut self.udt, (idx % 70) as i32, 4, 8);
    }

    pub(super) fn get_ud(&self) -> usize {
        get_comb(&self.udx, 4, 8) as usize * 70 + get_comb(&self.udt, 4, 8) as usize
    }

    pub(super) fn set_rl(&mut self, idx: usize) {
        set_comb(&mut self.rlx, (idx / 70) as i32, 4, 8);
        set_comb(&mut self.rlt, (idx % 70) as i32, 4, 8);
    }

    /// Reads the right/left state, first turning the arrays to the form with
    /// the last x slot unmarked (as the original does).
    pub(super) fn get_rl(&mut self) -> usize {
        if self.rlx[7] != -1 {
            for v in &mut self.rlx {
                *v = -1 - *v;
            }
            for i in 0..4 {
                self.rlt[i << 1 | 1] = -1 - self.rlt[i << 1 | 1];
            }
        }
        get_comb(&self.rlx, 4, 8) as usize * 70 + get_comb(&self.rlt, 4, 8) as usize
    }

    pub(super) fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let pow = i32::from(code % 3);
        let Centre { udt, udx, rlt, rlx } = self;
        match code {
            UW2 | U1 | U2 | U3 => {
                if code == UW2 {
                    swap2(rlt, 3, 7);
                    swap2(rlx, 3, 7);
                    swap2(rlx, 0, 4);
                }
                swap4(udt, [0, 1, 2, 3], pow, false);
                swap4(udx, [0, 1, 2, 3], pow, false);
            }
            RW2 | R2 => {
                if code == RW2 {
                    swap2(udt, 1, 5);
                    swap2(udx, 1, 5);
                    swap2(udx, 2, 6);
                }
                swap4(rlt, [0, 1, 2, 3], pow, false);
                swap4(rlx, [0, 1, 2, 3], pow, false);
            }
            FW2 => {
                swap2(udt, 2, 4);
                swap2(udx, 2, 4);
                swap2(udx, 3, 5);
                swap2(rlt, 2, 4);
                swap2(rlx, 2, 4);
                swap2(rlx, 3, 5);
            }
            DW2 | D1 | D2 | D3 => {
                if code == DW2 {
                    swap2(rlt, 1, 5);
                    swap2(rlx, 1, 5);
                    swap2(rlx, 2, 6);
                }
                swap4(udt, [4, 5, 6, 7], pow, false);
                swap4(udx, [4, 5, 6, 7], pow, false);
            }
            LW2 | L2 => {
                if code == LW2 {
                    swap2(udt, 3, 7);
                    swap2(udx, 3, 7);
                    swap2(udx, 0, 4);
                }
                swap4(rlt, [4, 5, 6, 7], pow, false);
                swap4(rlx, [4, 5, 6, 7], pow, false);
            }
            BW2 => {
                swap2(udt, 0, 6);
                swap2(udx, 0, 6);
                swap2(udx, 1, 7);
                swap2(rlt, 0, 6);
                swap2(rlx, 0, 6);
                swap2(rlx, 1, 7);
            }
            _ => {}
        }
    }

    pub(super) fn do_conj(&mut self, conj: u8) {
        let Centre { udt, udx, rlt, rlx } = self;
        match conj {
            0 => {
                for a in [&mut *udt, &mut *udx] {
                    for i in 0..4 {
                        a.swap(i, i + 4);
                    }
                }
                for a in [&mut *rlt, &mut *rlx] {
                    swap4(a, [0, 1, 2, 3], 1, false);
                    swap4(a, [4, 5, 6, 7], 1, false);
                }
                for i in 0..8 {
                    udt[i] = -1 - udt[i];
                    udx[i] = -1 - udx[i];
                }
            }
            1 => {
                for a in [&mut *rlt, &mut *rlx] {
                    for i in 0..4 {
                        a.swap(i, i + 4);
                    }
                }
                for a in [&mut *udt, &mut *udx] {
                    swap4(a, [0, 1, 2, 3], 1, false);
                    swap4(a, [4, 5, 6, 7], 1, false);
                }
                for i in 0..8 {
                    rlt[i] = -1 - rlt[i];
                    rlx[i] = -1 - rlx[i];
                }
            }
            _ => {
                swap2(udt, 1, 3);
                swap2(udt, 5, 7);
                for (a, b) in [(0, 1), (2, 3), (4, 5), (6, 7)] {
                    swap2(udx, a, b);
                }
                for (a, b) in [(0, 6), (1, 5), (2, 4), (3, 7)] {
                    swap2(rlt, a, b);
                }
                for (a, b) in [(0, 7), (1, 6), (2, 5), (3, 4)] {
                    swap2(rlx, a, b);
                }
                for i in 0..8 {
                    rlt[i] = -1 - rlt[i];
                    rlx[i] = -1 - rlx[i];
                }
            }
        }
    }
}

/// Middle, low and high wings, as slot arrays: each marks four of eight
/// slots with the number of the middle edge slot they are in.
pub(super) struct Edge {
    /// `[middle, low, high]`.
    pub e: [[i32; 8]; 3],
    pub standard: bool,
}

const M: usize = 0;
const L: usize = 1;
const H: usize = 2;

impl Edge {
    pub(super) fn new() -> Edge {
        let mut e = [[0; 8]; 3];
        for row in &mut e {
            for i in 0..4 {
                row[i] = -1;
                row[i + 4] = i as i32;
            }
        }
        Edge { e, standard: true }
    }

    /// Renames the pieces so that the middle edges are 0 to 3 in order.
    pub(super) fn standardise(&mut self) {
        if self.standard {
            return;
        }
        let mut quad = [[0i32; 4]; 3];
        for (q, row) in quad.iter_mut().zip(&self.e) {
            copy_from_comb(row, q);
        }
        let mut inverse = [0usize; 4];
        for i in 0..4 {
            inverse[quad[M][i] as usize] = i;
        }
        for i in 0..4 {
            quad[M][i] = i as i32;
            quad[L][i] = inverse[quad[L][i] as usize] as i32;
            quad[H][i] = inverse[quad[H][i] as usize] as i32;
        }
        for (q, row) in quad.iter().zip(&mut self.e) {
            copy_to_comb(q, row);
        }
        self.standard = true;
    }

    fn set(&mut self, which: usize, idx: usize) {
        self.standardise();
        set_comb(&mut self.e[which], (69 - idx / 24) as i32, 4, 8);
        let mut tmp = [0, 1, 2, 3];
        set_perm(&mut tmp, (idx % 24) as i32, 4, false);
        copy_to_comb(&tmp, &mut self.e[which]);
    }

    fn get(&mut self, which: usize) -> usize {
        self.standardise();
        let comb = 69 - get_comb(&self.e[which], 4, 8) as usize;
        let mut tmp = [0i32; 4];
        copy_from_comb(&self.e[which], &mut tmp);
        comb * 24 + get_perm(&tmp, 4, false) as usize
    }

    pub(super) fn set_m(&mut self, idx: usize) {
        self.set(M, idx * 24);
    }
    pub(super) fn get_m(&mut self) -> usize {
        self.get(M) / 24
    }
    pub(super) fn set_l(&mut self, idx: usize) {
        self.set(L, idx);
    }
    pub(super) fn get_l(&mut self) -> usize {
        self.get(L)
    }
    pub(super) fn set_h(&mut self, idx: usize) {
        self.set(H, idx);
    }
    pub(super) fn get_h(&mut self) -> usize {
        self.get(H)
    }

    pub(super) fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let pow = i32::from(code % 3);
        self.standard = false;
        let [mm, l, h] = &mut self.e;
        match code {
            UW2 | U1 | U2 | U3 => {
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap4(a, [0, 4, 1, 5], pow, false);
                }
            }
            RW2 | R2 => {
                if code == RW2 {
                    swap2(l, 1, 3);
                    swap2(h, 0, 2);
                }
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap2(a, 5, 6);
                }
            }
            FW2 | F2 => {
                if code == FW2 {
                    swap2(l, 5, 7);
                    swap2(h, 4, 6);
                }
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap2(a, 0, 3);
                }
            }
            DW2 | D1 | D2 | D3 => {
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap4(a, [2, 7, 3, 6], pow, false);
                }
            }
            LW2 | L2 => {
                if code == LW2 {
                    swap2(l, 0, 2);
                    swap2(h, 1, 3);
                }
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap2(a, 4, 7);
                }
            }
            BW2 | B2 => {
                if code == BW2 {
                    swap2(l, 4, 6);
                    swap2(h, 5, 7);
                }
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap2(a, 1, 2);
                }
            }
            _ => {}
        }
    }

    pub(super) fn do_conj(&mut self, conj: u8) {
        self.standard = false;
        let [mm, l, h] = &mut self.e;
        match conj {
            0 => {
                for a in [&mut *mm, &mut *l, &mut *h] {
                    for (x, y) in [(0, 2), (1, 3), (4, 7), (5, 6)] {
                        swap2(a, x, y);
                    }
                }
            }
            1 => {
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap4(a, [0, 4, 1, 5], 1, false);
                    swap4(a, [2, 7, 3, 6], 1, false);
                }
            }
            _ => {
                for i in 0..8 {
                    std::mem::swap(&mut l[i], &mut h[i]);
                }
                for a in [&mut *mm, &mut *l, &mut *h] {
                    swap2(a, 4, 5);
                    swap2(a, 6, 7);
                }
            }
        }
    }
}
