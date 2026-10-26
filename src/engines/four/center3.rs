//! Phase 3, centers: every face gets its own color, the parity of wings and
//! corners tracked.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Center3.java, MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! The state is `2 * ((ud * 35 + fb) * 12 + rl) + parity`: 29,400 states.
//! `ud` and `fb` (35 each) place 4 of 7 centers of a pair of faces, `rl` is
//! one of the 12 placements of the R and L centers that fit.

use super::center2::{comb_get, comb_set};
use super::cube::turn;
use crate::reduction::comb::CNK;

pub(super) const N_STATES: usize = 35 * 35 * 12 * 2;
/// Moves of the tables, and of the pruning search.
pub(super) const N_MOVES: usize = 20;
const N_SEARCH_MOVES: usize = 17;

/// The 12 placements of 4 of 8 centers of R and L that occur, as numbers of
/// the combinations of the 8 slots.
const RL2STD: [i32; 12] = [0, 9, 14, 23, 27, 28, 41, 42, 46, 55, 60, 69];

fn std2rl(idx: i32) -> Option<usize> {
    RL2STD.iter().position(|&v| v == idx)
}

/// The centers of the three pairs of faces and the parity.
#[derive(Clone, Copy)]
pub(super) struct State {
    ud: [u8; 8],
    rl: [u8; 8],
    fb: [u8; 8],
    parity: u8,
}

impl State {
    fn blank() -> State {
        State {
            ud: [0; 8],
            rl: [0; 8],
            fb: [0; 8],
            parity: 0,
        }
    }

    /// The state of the centers `centers`, with `extra` the parity of wings
    /// and corners together.
    pub(super) fn of(centers: &[u8; 24], extra: u8) -> State {
        let kind = |i: usize| centers[i] % 3;
        let parity =
            u8::from(!(((kind(0) > kind(8)) ^ (kind(8) > kind(16))) ^ (kind(0) > kind(16))));
        let mut s = State::blank();
        for i in 0..8 {
            s.ud[i] = (centers[i] / 3) ^ 1;
            s.fb[i] = (centers[i + 8] / 3) ^ 1;
            s.rl[i] = (centers[i + 16] / 3) ^ 1 ^ parity;
        }
        s.parity = parity ^ extra;
        s
    }

    /// The state number, or `None` if the centers do not fit the phase.
    pub(super) fn number(&self) -> Option<usize> {
        let ud = comb_get(|i| self.ud[i] != self.ud[7], 7, 4)?;
        let fb = comb_get(|i| self.fb[i] != self.fb[7], 7, 4)?;
        let check = self.fb[7] ^ self.ud[7];
        let rl = comb_get(|i| self.rl[i] != check, 8, 4)?;
        let rl = std2rl(rl)?;
        Some(usize::from(self.parity) + 2 * ((ud as usize * 35 + fb as usize) * 12 + rl))
    }

    fn set_number(&mut self, idx: usize) {
        self.parity = (idx & 1) as u8;
        let idx = idx >> 1;
        comb_set(&mut self.rl, RL2STD[idx % 12], 4);
        let idx = idx / 12;
        self.fb[7] = 0;
        comb_set(&mut self.fb[..7], (idx % 35) as i32, 4);
        self.ud[7] = 0;
        comb_set(&mut self.ud[..7], (idx / 35) as i32, 4);
    }

    fn apply(&mut self, i: usize) {
        self.parity ^= u8::from(i >= 14);
        let (ud, rl, fb) = (&mut self.ud, &mut self.rl, &mut self.fb);
        match i {
            0..=2 => turn(ud, [0, 1, 2, 3], i % 3),
            3 => turn(rl, [0, 1, 2, 3], 1),
            4..=6 => turn(fb, [0, 1, 2, 3], (i - 1) % 3),
            7..=9 => turn(ud, [4, 5, 6, 7], (i - 1) % 3),
            10 => turn(rl, [4, 5, 6, 7], 1),
            11..=13 => turn(fb, [4, 5, 6, 7], (i + 1) % 3),
            14 => {
                turn(ud, [0, 1, 2, 3], 1);
                turn(rl, [0, 5, 4, 1], 1);
                turn(fb, [0, 5, 4, 1], 1);
            }
            15 => {
                turn(rl, [0, 1, 2, 3], 1);
                turn(fb, [1, 4, 7, 2], 1);
                turn(ud, [1, 6, 5, 2], 1);
            }
            16 => {
                turn(fb, [0, 1, 2, 3], 1);
                turn(ud, [3, 2, 5, 4], 1);
                turn(rl, [0, 3, 6, 5], 1);
            }
            17 => {
                turn(ud, [4, 5, 6, 7], 1);
                turn(rl, [3, 2, 7, 6], 1);
                turn(fb, [3, 2, 7, 6], 1);
            }
            18 => {
                turn(rl, [4, 5, 6, 7], 1);
                turn(fb, [0, 3, 6, 5], 1);
                turn(ud, [0, 3, 4, 7], 1);
            }
            _ => {
                turn(fb, [4, 5, 6, 7], 1);
                turn(ud, [0, 7, 6, 1], 1);
                turn(rl, [1, 4, 7, 2], 1);
            }
        }
    }
}

/// The tables of phase 3's centers.
pub(super) struct Center3 {
    /// `ctmove[state * 20 + m]`.
    pub ctmove: Vec<u16>,
    /// Moves to the goal (255: unknown).
    pub prun: Vec<u8>,
}

impl Center3 {
    pub(super) fn build() -> Center3 {
        let mut c = State::blank();
        let mut ctmove = vec![0u16; N_STATES * N_MOVES];
        for i in 0..N_STATES {
            for m in 0..N_MOVES {
                c.set_number(i);
                c.apply(m);
                ctmove[i * N_MOVES + m] = c.number().expect("a reachable placement") as u16;
            }
        }
        let mut prun = vec![255u8; N_STATES];
        prun[0] = 0;
        let mut depth = 0u8;
        let mut frontier = vec![0usize];
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for &i in &frontier {
                for m in 0..N_SEARCH_MOVES {
                    let idx = usize::from(ctmove[i * N_MOVES + m]);
                    if prun[idx] == 255 {
                        prun[idx] = depth + 1;
                        next.push(idx);
                    }
                }
            }
            depth += 1;
            frontier = next;
        }
        debug_assert!(CNK[7][4] == 35);
        Center3 { ctmove, prun }
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::fnv;
    use super::*;

    #[test]
    fn tables_match_the_reference_checksums() {
        let c = Center3::build();
        assert_eq!(
            fnv(c.ctmove.iter().map(|&v| i64::from(v))),
            754766324099193587
        );
        assert_eq!(
            fnv(c.prun.iter().map(|&v| i64::from(v as i8))),
            6503529057452532853
        );
        let mut histogram = [0usize; 11];
        for &d in &c.prun {
            histogram[usize::from(d)] += 1;
        }
        // 29,400 states, of which the pruning search reaches these (the
        // reference counts the same layers).
        assert_eq!(
            histogram,
            [1, 3, 34, 238, 1118, 3719, 6991, 8360, 6600, 2208, 128]
        );
    }
}
