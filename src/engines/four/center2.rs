//! Phase 2: the centers of the four faces other than R and L sorted by kind,
//! the R and L centers by color, and the parity of the wings tracked.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Center2.java, MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! The state is `ct * 70 + rl`: `ct` (6,435) says which 8 of the 15 centers of
//! the faces U, D, F, B differ from the last one, `rl` (70) which 4 of the 7
//! centers of R and L differ from the last one, times two for the parity.

use super::cube::turn;
use super::moves::{MOVE2STD, SET2};
use crate::reduction::comb::CNK;

pub(super) const N_CT: usize = 6435;
pub(super) const N_RL: usize = 70;
/// Moves of the tables, and of the pruning search.
pub(super) const N_MOVES: usize = 28;
const N_SEARCH_MOVES: usize = 23;

/// Number of the placement of `r` marked slots among the `n` slots whose
/// mark `marked(i)` says so, or `None` if the count of marks is not `r`.
pub(super) fn comb_get(marked: impl Fn(usize) -> bool, n: usize, r: usize) -> Option<i32> {
    let mut left = r;
    let mut idx = 0;
    for i in (0..n).rev() {
        if marked(i) {
            if left == 0 {
                return None;
            }
            idx += CNK[i][left];
            left -= 1;
        }
    }
    (left == 0).then_some(idx)
}

/// Marks `r` of the slots of `out` with 1 (the others 0): the placement `idx`.
pub(super) fn comb_set(out: &mut [u8], mut idx: i32, mut r: usize) {
    for i in (0..out.len()).rev() {
        if r > 0 && idx >= CNK[i][r] {
            idx -= CNK[i][r];
            r -= 1;
            out[i] = 1;
        } else {
            out[i] = 0;
        }
    }
}

/// The centers of phase 2 and the wing parity.
#[derive(Clone, Copy)]
pub(super) struct State {
    rl: [u8; 8],
    ct: [u8; 16],
    parity: u8,
}

impl State {
    fn blank() -> State {
        State {
            rl: [0; 8],
            ct: [0; 16],
            parity: 0,
        }
    }

    /// The state of the centers `centers` of a cube with wing parity `parity`.
    #[allow(clippy::needless_range_loop)] // index loops as in the reference
    pub(super) fn of(centers: &[u8; 24], parity: u8) -> State {
        let mut s = State::blank();
        for i in 0..16 {
            s.ct[i] = centers[i] % 3;
        }
        s.rl.copy_from_slice(&centers[16..]);
        s.parity = parity;
        s
    }

    /// `(ct, rl)`, or `None` if the centers are not in the right classes.
    pub(super) fn numbers(&self) -> Option<(usize, usize)> {
        Some((self.ct_number()?, self.rl_number()?))
    }

    fn set_rl(&mut self, idx: usize) {
        self.parity = (idx & 1) as u8;
        self.rl[7] = 0;
        comb_set(&mut self.rl[..7], (idx >> 1) as i32, 4);
    }

    fn set_ct(&mut self, idx: usize) {
        self.ct[15] = 0;
        comb_set(&mut self.ct[..15], idx as i32, 8);
    }

    fn rl_number(&self) -> Option<usize> {
        let rl = comb_get(|i| self.rl[i] != self.rl[7], 7, 4)?;
        Some(rl as usize * 2 + usize::from(self.parity))
    }

    fn ct_number(&self) -> Option<usize> {
        let ct = comb_get(|i| self.ct[i] != self.ct[15], 15, 8)?;
        Some(ct as usize)
    }

    fn apply(&mut self, m: usize) {
        self.parity ^= u8::from(matches!(m, 21 | 23 | 30 | 32));
        let key = m % 3;
        let (ct, rl) = (&mut self.ct, &mut self.rl);
        match m / 3 {
            0 => turn(ct, [0, 1, 2, 3], key),
            1 => turn(rl, [0, 1, 2, 3], key),
            2 => turn(ct, [8, 9, 10, 11], key),
            3 => turn(ct, [4, 5, 6, 7], key),
            4 => turn(rl, [4, 5, 6, 7], key),
            5 => turn(ct, [12, 13, 14, 15], key),
            6 => {
                turn(ct, [0, 1, 2, 3], key);
                turn(rl, [0, 5, 4, 1], key);
                turn(ct, [8, 9, 12, 13], key);
            }
            7 => {
                turn(rl, [0, 1, 2, 3], key);
                turn(ct, [1, 15, 5, 9], key);
                turn(ct, [2, 12, 6, 10], key);
            }
            8 => {
                turn(ct, [8, 9, 10, 11], key);
                turn(rl, [0, 3, 6, 5], key);
                turn(ct, [3, 2, 5, 4], key);
            }
            9 => {
                turn(ct, [4, 5, 6, 7], key);
                turn(rl, [3, 2, 7, 6], key);
                turn(ct, [11, 10, 15, 14], key);
            }
            10 => {
                turn(rl, [4, 5, 6, 7], key);
                turn(ct, [0, 8, 4, 14], key);
                turn(ct, [3, 11, 7, 13], key);
            }
            _ => {
                turn(ct, [12, 13, 14, 15], key);
                turn(rl, [1, 4, 7, 2], key);
                turn(ct, [1, 0, 7, 6], key);
            }
        }
    }
}

/// The tables of phase 2.
pub(super) struct Center2 {
    /// `rlmv[rl * 28 + m]`.
    pub rlmv: Vec<u8>,
    /// `ctmv[ct * 28 + m]`.
    pub ctmv: Vec<u16>,
    /// `ctprun[ct * 70 + rl]`: moves to the goal (255: unknown).
    pub ctprun: Vec<u8>,
}

impl Center2 {
    pub(super) fn build() -> Center2 {
        let mut c = State::blank();
        let mut rlmv = vec![0u8; N_RL * N_MOVES];
        for i in 0..N_RL {
            for m in 0..N_MOVES {
                c.set_rl(i);
                c.apply(MOVE2STD[m]);
                rlmv[i * N_MOVES + m] = c.rl_number().expect("a placement") as u8;
            }
        }
        let mut ctmv = vec![0u16; N_CT * N_MOVES];
        for i in 0..N_CT {
            for m in 0..N_MOVES {
                c.set_ct(i);
                c.apply(MOVE2STD[m]);
                ctmv[i * N_MOVES + m] = c.ct_number().expect("a placement") as u16;
            }
        }
        let mut ctprun = vec![255u8; N_CT * N_RL];
        for goal in [0, 18, 28, 46, 54, 56] {
            ctprun[goal] = 0;
        }
        let mut depth = 0u8;
        let mut frontier: Vec<usize> = vec![0, 18, 28, 46, 54, 56];
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for &i in &frontier {
                let (ct, rl) = (i / N_RL, i % N_RL);
                for m in 0..N_SEARCH_MOVES {
                    let idx = usize::from(ctmv[ct * N_MOVES + m]) * N_RL
                        + usize::from(rlmv[rl * N_MOVES + m]);
                    if ctprun[idx] == 255 {
                        ctprun[idx] = depth + 1;
                        next.push(idx);
                    }
                }
            }
            depth += 1;
            frontier = next;
        }
        debug_assert!(SET2.std.len() == N_MOVES + 1);
        Center2 { rlmv, ctmv, ctprun }
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::fnv;
    use super::*;

    #[test]
    fn tables_match_the_reference_checksums() {
        let c = Center2::build();
        // The reference hashes int[][] rlmv and char[][] ctmv.
        assert_eq!(
            fnv(c.rlmv.iter().map(|&v| i64::from(v))),
            -5693903271162198417
        );
        assert_eq!(
            fnv(c.ctmv.iter().map(|&v| i64::from(v))),
            2564983055892664183
        );
        assert_eq!(
            fnv(c.ctprun.iter().map(|&v| i64::from(v as i8))),
            8892340655916192270
        );
        let mut histogram = [0usize; 10];
        for &d in &c.ctprun {
            histogram[usize::from(d)] += 1;
        }
        assert_eq!(
            histogram,
            [6, 11, 97, 840, 5651, 30868, 114112, 217442, 80295, 1128]
        );
    }

    #[test]
    fn placements_round_trip() {
        let mut slots = [0u8; 15];
        for idx in (0..N_CT as i32).step_by(13) {
            comb_set(&mut slots, idx, 8);
            assert_eq!(comb_get(|i| slots[i] == 1, 15, 8), Some(idx));
        }
        assert_eq!(comb_get(|_| true, 15, 8), None);
        assert_eq!(comb_get(|_| false, 15, 8), None);
    }
}
