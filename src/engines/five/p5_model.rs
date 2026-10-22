//! Phase 5 pieces: the centres and the low and high wings of the last phase.
//!
//! Ported from cs0x7f/cube555 src/Phase5Center.java and src/Phase5Edge.java,
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md.

use super::mv::*;
use crate::reduction::comb::{get_comb, get_perm, set_comb, set_perm, swap2, swap4};

/// The fourteen moves of the phase.
pub(super) const MOVES: [u8; 14] = [U1, U2, U3, R2, F2, D1, D2, D3, L2, B2, RW2, FW2, LW2, BW2];

/// Centres: t and x kind on U and D, and the four-and-four of R F L B.
pub(super) struct Centre {
    pub t: [i32; 8],
    pub x: [i32; 8],
    pub rflb: [i32; 8],
}

impl Centre {
    pub(super) fn new() -> Centre {
        let mut c = Centre {
            t: [0; 8],
            x: [0; 8],
            rflb: [0; 8],
        };
        c.set_rflb(0);
        c.set_x(0);
        c.set_t(0);
        c
    }

    pub(super) fn set_rflb(&mut self, idx: usize) {
        let (mut fb, mut rl) = ([0; 4], [0; 4]);
        set_comb(&mut fb, (idx % 6) as i32, 2, 4);
        set_comb(&mut rl, (idx / 6) as i32, 2, 4);
        self.rflb[..4].copy_from_slice(&fb);
        self.rflb[4..].copy_from_slice(&rl);
    }

    pub(super) fn get_rflb(&self) -> usize {
        get_comb(&self.rflb[4..], 2, 4) as usize * 6 + get_comb(&self.rflb[..4], 2, 4) as usize
    }

    pub(super) fn set_x(&mut self, idx: usize) {
        set_comb(&mut self.x, idx as i32, 4, 8);
    }
    pub(super) fn get_x(&self) -> usize {
        get_comb(&self.x, 4, 8) as usize
    }
    pub(super) fn set_t(&mut self, idx: usize) {
        set_comb(&mut self.t, idx as i32, 4, 8);
    }
    pub(super) fn get_t(&self) -> usize {
        get_comb(&self.t, 4, 8) as usize
    }

    pub(super) fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let pow = i32::from(code % 3);
        let Centre { t, x, rflb } = self;
        match code {
            U1 | U2 | U3 => {
                swap4(t, [0, 1, 2, 3], pow, false);
                swap4(x, [0, 1, 2, 3], pow, false);
            }
            RW2 => {
                swap2(t, 1, 5);
                swap2(x, 1, 5);
                swap2(x, 2, 6);
                swap2(rflb, 0, 3);
                swap2(rflb, 4, 5);
            }
            R2 => swap2(rflb, 4, 5),
            FW2 => {
                swap2(t, 2, 4);
                swap2(x, 2, 4);
                swap2(x, 3, 5);
                swap2(rflb, 5, 6);
                swap2(rflb, 0, 1);
            }
            F2 => swap2(rflb, 0, 1),
            D1 | D2 | D3 => {
                swap4(t, [4, 5, 6, 7], pow, false);
                swap4(x, [4, 5, 6, 7], pow, false);
            }
            LW2 => {
                swap2(t, 3, 7);
                swap2(x, 3, 7);
                swap2(x, 0, 4);
                swap2(rflb, 1, 2);
                swap2(rflb, 6, 7);
            }
            L2 => swap2(rflb, 6, 7),
            BW2 => {
                swap2(t, 0, 6);
                swap2(x, 0, 6);
                swap2(x, 1, 7);
                swap2(rflb, 4, 7);
                swap2(rflb, 2, 3);
            }
            B2 => swap2(rflb, 2, 3),
            _ => {}
        }
    }

    pub(super) fn do_conj(&mut self, conj: u8) {
        let Centre { t, x, rflb } = self;
        match conj {
            0 => {
                swap4(t, [0, 1, 2, 3], 0, false);
                swap4(x, [0, 1, 2, 3], 0, false);
                swap4(t, [4, 5, 6, 7], 2, false);
                swap4(x, [4, 5, 6, 7], 2, false);
                for v in &mut rflb[..4] {
                    *v = -1 - *v;
                }
                swap4(rflb, [1, 7, 3, 5], 0, false);
                swap4(rflb, [0, 6, 2, 4], 0, false);
            }
            1 => {
                for i in 0..4 {
                    t.swap(i, i + 4);
                    x.swap(i, i + 4);
                }
                for (a, b) in [(0, 3), (4, 5), (1, 2), (6, 7)] {
                    swap2(rflb, a, b);
                }
                for i in 0..8 {
                    t[i] = -1 - t[i];
                    x[i] = -1 - x[i];
                }
                for v in &mut rflb[..4] {
                    *v = -1 - *v;
                }
            }
            _ => {
                swap2(t, 1, 3);
                swap2(t, 5, 7);
                for (a, b) in [(0, 1), (2, 3), (4, 5), (6, 7)] {
                    swap2(x, a, b);
                }
                for (a, b) in [(0, 1), (2, 3), (4, 7), (5, 6)] {
                    swap2(rflb, a, b);
                }
                for v in &mut rflb[4..] {
                    *v = -1 - *v;
                }
            }
        }
    }
}

/// Middle, low and high wings of the layer, each a permutation of eight.
pub(super) struct Edge {
    /// `[middle, low, high]`.
    pub e: [[i32; 8]; 3],
    pub standard: bool,
}

impl Edge {
    pub(super) fn new() -> Edge {
        let mut row = [0; 8];
        for (i, v) in row.iter_mut().enumerate() {
            *v = i as i32;
        }
        Edge {
            e: [row; 3],
            standard: true,
        }
    }

    pub(super) fn standardise(&mut self) {
        if self.standard {
            return;
        }
        let mut inverse = [0i32; 8];
        for i in 0..8 {
            inverse[self.e[0][i] as usize] = i as i32;
        }
        for i in 0..8 {
            self.e[0][i] = i as i32;
            self.e[1][i] = inverse[self.e[1][i] as usize];
            self.e[2][i] = inverse[self.e[2][i] as usize];
        }
        self.standard = true;
    }

    pub(super) fn set_l(&mut self, idx: usize) {
        self.standardise();
        set_perm(&mut self.e[1], idx as i32, 8, false);
    }
    pub(super) fn get_l(&mut self) -> usize {
        self.standardise();
        get_perm(&self.e[1], 8, false) as usize
    }
    pub(super) fn get_h(&mut self) -> usize {
        self.standardise();
        get_perm(&self.e[2], 8, false) as usize
    }

    pub(super) fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let pow = i32::from(code % 3);
        self.standard = false;
        let [mm, l, h] = &mut self.e;
        match code {
            U1 | U2 | U3 => {
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
            D1 | D2 | D3 => {
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
                    swap4(a, [0, 4, 1, 5], 0, false);
                    swap4(a, [2, 7, 3, 6], 2, false);
                }
            }
            1 => {
                for a in [&mut *mm, &mut *l, &mut *h] {
                    for (x, y) in [(0, 2), (1, 3), (4, 7), (5, 6)] {
                        swap2(a, x, y);
                    }
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
