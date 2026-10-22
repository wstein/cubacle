//! The 48 symmetries of the cube as seen by the centres, and the tables that
//! relate them to the 36 moves.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Center1.java (`rot`, `rotate`,
//! `initSym`, `symmult`, `symmove`, `syminv`, `finish`), MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! Symmetry `j` is what `step(j)` adds to symmetry `j - 1`; the numbering is
//! the reference's, so that the tables agree with its solutions.

use super::cube::{move_centres, turn};
use super::moves::{BX3, DX2, DX3, FX1, LX3, RX1, UX1, UX2};

/// One generator of the symmetries, on any labelling of the 24 centres.
pub(super) fn rot<T: Copy>(ct: &mut [T; 24], r: usize) {
    match r {
        0 => {
            move_centres(ct, UX2);
            move_centres(ct, DX2);
        }
        1 => {
            move_centres(ct, RX1);
            move_centres(ct, LX3);
        }
        2 => {
            turn(ct, [0, 3, 1, 2], 1);
            turn(ct, [8, 11, 9, 10], 1);
            turn(ct, [4, 7, 5, 6], 1);
            turn(ct, [12, 15, 13, 14], 1);
            turn(ct, [16, 19, 21, 22], 1);
            turn(ct, [17, 18, 20, 23], 1);
        }
        _ => {
            move_centres(ct, UX1);
            move_centres(ct, DX3);
            move_centres(ct, FX1);
            move_centres(ct, BX3);
        }
    }
}

/// Goes from symmetry `j` to the next one.
pub(super) fn step<T: Copy>(ct: &mut [T; 24], j: usize) {
    rot(ct, 0);
    if j % 2 == 1 {
        rot(ct, 1);
    }
    if j % 8 == 7 {
        rot(ct, 2);
    }
    if j % 16 == 15 {
        rot(ct, 3);
    }
}

/// Applies symmetry `r` (the first `r` steps from the identity).
fn rotate<T: Copy>(ct: &mut [T; 24], r: usize) {
    for j in 0..r {
        step(ct, j);
    }
}

/// Products, inverses and the action on moves of the 48 symmetries.
pub(super) struct Syms {
    /// `mult[i][j]`: symmetry `i` followed by `j`, as the reference defines it.
    pub mult: [[u8; 48]; 48],
    pub inv: [u8; 48],
    /// `mv[sym][m]`: the move `m` seen through `sym`.
    pub mv: [[u8; 36]; 48],
    /// `finish[sym]`: the flagged-centre number the solved state has seen
    /// through the symmetry (0, 12869 or 735470).
    pub finish: [u32; 48],
}

fn identity() -> [u8; 24] {
    let mut c = [0u8; 24];
    for (i, slot) in c.iter_mut().enumerate() {
        *slot = i as u8;
    }
    c
}

impl Syms {
    #[allow(clippy::needless_range_loop)] // index loops as in the reference
    pub(super) fn new() -> Syms {
        let mut mult = [[0u8; 48]; 48];
        let mut inv = [0u8; 48];
        let e = identity();
        let mut c = e;
        for i in 0..48 {
            for j in 0..48 {
                let mut d = e;
                for k in 0..48 {
                    if c == d {
                        mult[i][j] = k as u8;
                        if k == 0 {
                            inv[i] = j as u8;
                        }
                    }
                    step(&mut d, k);
                }
                step(&mut c, j);
            }
            step(&mut c, i);
        }

        let mut mv = [[0u8; 36]; 48];
        for i in 0..48 {
            let mut c = e;
            rotate(&mut c, usize::from(inv[i]));
            for j in 0..36 {
                let mut d = c;
                move_centres(&mut d, j);
                rotate(&mut d, i);
                for k in 0..36 {
                    let mut f = e;
                    move_centres(&mut f, k);
                    if f == d {
                        mv[i][j] = k as u8;
                        break;
                    }
                }
            }
        }

        let mut finish = [0u32; 48];
        let mut c = super::center1::Flags::from_number(0);
        for i in 0..48 {
            finish[usize::from(inv[i])] = c.number();
            c.step(i);
        }
        Syms {
            mult,
            inv,
            mv,
            finish,
        }
    }

    pub(super) fn mult(&self, a: usize, b: usize) -> usize {
        usize::from(self.mult[a][b])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_symmetries_form_a_group() {
        let s = Syms::new();
        for a in 0..48 {
            assert_eq!(s.mult(a, usize::from(s.inv[a])), 0);
            assert_eq!(s.mult(0, a), a);
            assert_eq!(s.mult(a, 0), a);
            for b in 0..48 {
                // closed: every product is again a symmetry, and rows are permutations
                assert!(s.mult(a, b) < 48);
            }
            let mut row: Vec<u8> = s.mult[a].to_vec();
            row.sort_unstable();
            assert_eq!(row, (0..48).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn the_identity_leaves_moves_alone_and_the_solved_state_finishes_at_zero() {
        let s = Syms::new();
        for m in 0..36 {
            assert_eq!(usize::from(s.mv[0][m]), m);
        }
        // The reference prints these (Center1.finish).
        let mut expected = [0u32; 48];
        for (i, v) in expected.iter_mut().enumerate() {
            *v = [
                0, 0, 12869, 12869, 0, 0, 12869, 12869, 0, 0, 12869, 12869, 0, 0, 12869, 12869,
                12869, 12869, 735470, 735470, 12869, 12869, 735470, 735470, 12869, 12869, 735470,
                735470, 12869, 12869, 735470, 735470, 735470, 735470, 0, 0, 735470, 735470, 0, 0,
                735470, 735470, 0, 0, 735470, 735470, 0, 0,
            ][i];
        }
        assert_eq!(s.finish, expected);
    }
}
