//! The 48 symmetries of the cube acting on `Cubie`, their products, and
//! their action on moves.
//!
//! Ported from cs0x7f/cube555 src/CubieCube.java (`init`, `doConj`,
//! `getSymMove`), MIT, Copyright (c) 2023 Chen Shuang; this file is
//! AGPL-3.0-or-later, see NOTICE.md.

use super::cubie::Cubie;
use super::mv::{DW2, DW3, LW3, RW1, UW1, UW2};
use crate::reduction::comb::{swap2, swap4};

/// Slot of the centre piece that carries each colour.
const COLOR_TO_CENTER: [usize; 6] = [0, 16, 8, 4, 20, 12];

pub(crate) struct Syms {
    cube: Vec<Cubie>,
    /// `mult[i][j]`: the symmetry `i * j`.
    pub mult: [[u8; 48]; 48],
    /// `mult_inv[k][j]`: the `i` with `i * j = k`.
    pub mult_inv: [[u8; 48]; 48],
    /// `mv[s][m]`: move `m` seen through symmetry `s`.
    pub mv: [[u8; 36]; 48],
}

impl Syms {
    pub(crate) fn new() -> Syms {
        let mut c = Cubie::solved();
        for i in 0..24 {
            c.tcenter[i] = i as i32;
            c.xcenter[i] = i as i32;
        }
        let mut cube = Vec::with_capacity(48);
        for i in 0..48 {
            cube.push(c.clone());
            // x
            rotate_x(&mut c);
            if i & 3 == 3 {
                // y2
                c.do_move(UW2);
                c.do_move(DW2);
                swap4(&mut c.tcenter, [9, 21, 13, 17], 1, false);
                swap4(&mut c.tcenter, [11, 23, 15, 19], 1, false);
                swap4(&mut c.medge, [8, 9, 10, 11], 1, true);
            }
            if i & 7 == 7 {
                mirror(&mut c);
            }
            if i & 0xf == 0xf {
                // URF -> RFU, that is x y
                rotate_x(&mut c);
                c.do_move(UW1);
                c.do_move(DW3);
                swap4(&mut c.tcenter, [9, 21, 13, 17], 0, false);
                swap4(&mut c.tcenter, [11, 23, 15, 19], 0, false);
                swap4(&mut c.medge, [8, 9, 10, 11], 0, true);
            }
        }
        let mut mult = [[0u8; 48]; 48];
        let mut mult_inv = [[0u8; 48]; 48];
        for i in 0..48 {
            for j in 0..48 {
                let product = Cubie::mult(&cube[i], &cube[j]);
                let k = (0..48)
                    .find(|&k| cube[k].wedge == product.wedge)
                    .expect("symmetries form a group");
                mult[i][j] = k as u8;
                mult_inv[k][j] = i as u8;
            }
        }
        let mut syms = Syms {
            cube,
            mult,
            mult_inv,
            mv: [[0; 36]; 48],
        };
        let wings: Vec<[i32; 24]> = (0..36u8)
            .map(|m| {
                let mut d = Cubie::solved();
                d.do_move(m);
                d.wedge
            })
            .collect();
        for m in 0..36u8 {
            for s in 0..48 {
                let mut c = Cubie::solved();
                c.do_move(m);
                syms.conj(&mut c, usize::from(syms.mult_inv[0][s]));
                let m2 = wings
                    .iter()
                    .position(|w| *w == c.wedge)
                    .expect("a symmetry maps moves to moves");
                syms.mv[s][usize::from(m)] = m2 as u8;
            }
        }
        syms
    }

    /// Conjugates `c` by symmetry `idx`.
    pub(crate) fn conj(&self, c: &mut Cubie, idx: usize) {
        let a = c.clone();
        let sinv = &self.cube[usize::from(self.mult_inv[0][idx])];
        let s = &self.cube[idx];
        let solved = Cubie::solved();
        for i in 0..12 {
            let e = a.medge[(s.medge[i] >> 1) as usize];
            c.medge[i] = sinv.medge[(e >> 1) as usize] ^ (e & 1) ^ (s.medge[i] & 1);
        }
        for i in 0..24 {
            let t = a.tcenter[s.tcenter[i] as usize] as usize;
            c.tcenter[i] = solved.tcenter[sinv.tcenter[COLOR_TO_CENTER[t]] as usize];
            let x = a.xcenter[s.xcenter[i] as usize] as usize;
            c.xcenter[i] = solved.xcenter[sinv.xcenter[COLOR_TO_CENTER[x]] as usize];
            c.wedge[i] = sinv.wedge[a.wedge[s.wedge[i] as usize] as usize];
        }
    }

    /// `table[s * moves.len() + m]`: index in `moves` of move `m` under
    /// symmetry `s`, for the first `nsym` symmetries.
    pub(crate) fn sym_moves(&self, moves: &[u8], nsym: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(nsym * moves.len());
        for s in 0..nsym {
            for &m in moves {
                let image = self.mv[s][usize::from(m)];
                out.push(
                    moves
                        .iter()
                        .position(|&x| x == image)
                        .map_or(u8::MAX, |p| p as u8),
                );
            }
        }
        out
    }
}

fn rotate_x(c: &mut Cubie) {
    c.do_move(RW1);
    c.do_move(LW3);
    swap4(&mut c.tcenter, [0, 14, 4, 8], 0, false);
    swap4(&mut c.tcenter, [2, 12, 6, 10], 0, false);
    swap4(&mut c.medge, [0, 1, 2, 3], 0, true);
}

fn mirror(c: &mut Cubie) {
    for (a, b) in [
        (1, 3),
        (5, 7),
        (9, 11),
        (13, 15),
        (16, 20),
        (17, 23),
        (18, 22),
        (19, 21),
    ] {
        swap2(&mut c.tcenter, a, b);
    }
    for (a, b) in [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (8, 9),
        (10, 11),
        (12, 13),
        (14, 15),
        (16, 21),
        (17, 20),
        (18, 23),
        (19, 22),
    ] {
        swap2(&mut c.xcenter, a, b);
    }
    for (a, b) in [
        (0, 12),
        (1, 13),
        (2, 14),
        (3, 15),
        (4, 17),
        (5, 16),
        (6, 19),
        (7, 18),
        (8, 23),
        (9, 22),
        (10, 21),
        (11, 20),
    ] {
        swap2(&mut c.wedge, a, b);
    }
    for (a, b) in [(4, 5), (6, 7), (8, 11), (9, 10)] {
        swap2(&mut c.medge, a, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_symmetries_form_a_group_acting_on_moves() {
        let syms = Syms::new();
        // identity, closure and the inverse table
        for i in 0..48 {
            assert_eq!(syms.mult[0][i], i as u8);
            assert_eq!(syms.mult[i][0], i as u8);
            for j in 0..48 {
                assert_eq!(syms.mult[usize::from(syms.mult_inv[i][j])][j], i as u8);
            }
        }
        // every symmetry permutes the 36 moves
        for s in 0..48 {
            let mut seen = [false; 36];
            for m in 0..36 {
                seen[usize::from(syms.mv[s][m])] = true;
            }
            assert!(seen.iter().all(|&x| x), "symmetry {s}");
        }
        // moves keep their turn count
        for s in 0..48 {
            for m in 0..36 {
                assert_eq!(syms.mv[s][m] % 3 == 1, m % 3 == 1);
            }
        }
    }

    #[test]
    fn conjugation_by_the_identity_changes_nothing_and_moves_commute_with_it() {
        let syms = Syms::new();
        let mut c = Cubie::solved();
        for m in [3, 21, 7, 30] {
            c.do_move(m);
        }
        let mut same = c.clone();
        syms.conj(&mut same, 0);
        assert_eq!(same, c);
    }
}
