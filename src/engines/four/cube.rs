//! The cube as pieces: 8 corners, 24 wings, 24 centers, and the 36 moves.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/CornerCube.java, EdgeCube.java,
//! CenterCube.java and Util.java (`swap`, `set8Perm`, `parity`), MIT,
//! Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md.
//!
//! Numbering, as in the reference. Centers: positions 0-3 the U face, 4-7 D,
//! 8-11 F, 12-15 B, 16-19 R, 20-23 L, each clockwise from its top left;
//! `ct[i]` is the color (0 to 5 for `URFDLB`). Wings: positions 0-11 and 12-23
//! are the two wings of each of the 12 edges. Corners: `cp`/`co` as in
//! min2phase.

use crate::reduction::comb::set_perm;

/// Turns four slots `a b c d` of `arr`: `key` 0 forwards, 1 half, 2 backwards.
#[inline(always)]
pub(super) fn turn<T: Copy>(arr: &mut [T], [a, b, c, d]: [usize; 4], key: usize) {
    match key {
        0 => {
            let t = arr[d];
            arr[d] = arr[c];
            arr[c] = arr[b];
            arr[b] = arr[a];
            arr[a] = t;
        }
        1 => {
            arr.swap(a, c);
            arr.swap(b, d);
        }
        _ => {
            let t = arr[a];
            arr[a] = arr[b];
            arr[b] = arr[c];
            arr[c] = arr[d];
            arr[d] = t;
        }
    }
}

/// The four-cycles a face turn makes on the centers: the face itself, then
/// for the wide turns the second layer of the four neighbors.
const CENTER_CYCLES: [&[[usize; 4]]; 12] = [
    &[[0, 1, 2, 3]],
    &[[16, 17, 18, 19]],
    &[[8, 9, 10, 11]],
    &[[4, 5, 6, 7]],
    &[[20, 21, 22, 23]],
    &[[12, 13, 14, 15]],
    &[[0, 1, 2, 3], [8, 20, 12, 16], [9, 21, 13, 17]],
    &[[16, 17, 18, 19], [1, 15, 5, 9], [2, 12, 6, 10]],
    &[[8, 9, 10, 11], [2, 19, 4, 21], [3, 16, 5, 22]],
    &[[4, 5, 6, 7], [10, 18, 14, 22], [11, 19, 15, 23]],
    &[[20, 21, 22, 23], [0, 8, 4, 14], [3, 11, 7, 13]],
    &[[12, 13, 14, 15], [1, 20, 7, 18], [0, 23, 6, 17]],
];

/// The four-cycles a face turn makes on the wings.
const WING_CYCLES: [&[[usize; 4]]; 12] = [
    &[[0, 1, 2, 3], [12, 13, 14, 15]],
    &[[11, 15, 10, 19], [23, 3, 22, 7]],
    &[[0, 11, 6, 8], [12, 23, 18, 20]],
    &[[4, 5, 6, 7], [16, 17, 18, 19]],
    &[[1, 20, 5, 21], [13, 8, 17, 9]],
    &[[2, 9, 4, 10], [14, 21, 16, 22]],
    &[[0, 1, 2, 3], [12, 13, 14, 15], [9, 22, 11, 20]],
    &[[11, 15, 10, 19], [23, 3, 22, 7], [2, 16, 6, 12]],
    &[[0, 11, 6, 8], [12, 23, 18, 20], [3, 19, 5, 13]],
    &[[4, 5, 6, 7], [16, 17, 18, 19], [8, 23, 10, 21]],
    &[[1, 20, 5, 21], [13, 8, 17, 9], [14, 0, 18, 4]],
    &[[2, 9, 4, 10], [14, 21, 16, 22], [7, 15, 1, 17]],
];

/// Turns the centers by move `m` (0 to 35).
pub(super) fn move_centers<T: Copy>(ct: &mut [T; 24], m: usize) {
    for &cycle in CENTER_CYCLES[m / 3] {
        turn(ct, cycle, m % 3);
    }
}

/// Turns the wings by move `m` (0 to 35).
pub(super) fn move_wings(ep: &mut [u8; 24], m: usize) {
    for &cycle in WING_CYCLES[m / 3] {
        turn(ep, cycle, m % 3);
    }
}

/// The corners as `(cp, co)`; a corner move multiplies on the right.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Corners {
    pub cp: [u8; 8],
    pub co: [u8; 8],
}

impl Corners {
    pub(super) const SOLVED: Corners = Corners {
        cp: [0, 1, 2, 3, 4, 5, 6, 7],
        co: [0; 8],
    };

    /// The corners with permutation number `cperm` and twist number `twist`.
    pub(super) fn from_numbers(cperm: i32, twist: i32) -> Corners {
        let mut perm = [0i32; 8];
        set_perm(&mut perm, cperm, 8, false);
        let mut cp = [0u8; 8];
        for (slot, &p) in cp.iter_mut().zip(&perm) {
            *slot = p as u8;
        }
        let mut co = [0u8; 8];
        let (mut idx, mut sum) = (twist, 0);
        for i in (0..7).rev() {
            co[i] = (idx % 3) as u8;
            sum += idx % 3;
            idx /= 3;
        }
        co[7] = ((15 - sum) % 3) as u8;
        Corners { cp, co }
    }

    /// `self * other`.
    pub(super) fn times(&self, other: &Corners) -> Corners {
        let mut out = Corners::SOLVED;
        for corn in 0..8 {
            let from = usize::from(other.cp[corn]);
            out.cp[corn] = self.cp[from];
            out.co[corn] = (self.co[from] + other.co[corn]) % 3;
        }
        out
    }

    /// The parity of the corner permutation (0 even, 1 odd).
    pub(super) fn parity(&self) -> u8 {
        parity(&self.cp)
    }
}

/// The parity (0 even, 1 odd) of a permutation of small numbers.
pub(super) fn parity(arr: &[u8]) -> u8 {
    let mut p = 0;
    for i in 0..arr.len() {
        for j in i..arr.len() {
            if arr[i] > arr[j] {
                p ^= 1;
            }
        }
    }
    p
}

/// What a corner move does, for the 18 face turns (a wide turn moves the
/// corners as its outer face does).
#[derive(Clone)]
pub(super) struct CornerMoves {
    cubes: [Corners; 18],
}

impl CornerMoves {
    pub(super) fn new() -> CornerMoves {
        let mut cubes = [Corners::SOLVED; 18];
        cubes[0] = Corners::from_numbers(15120, 0);
        cubes[3] = Corners::from_numbers(21021, 1494);
        cubes[6] = Corners::from_numbers(8064, 1236);
        cubes[9] = Corners::from_numbers(9, 0);
        cubes[12] = Corners::from_numbers(1230, 412);
        cubes[15] = Corners::from_numbers(224, 137);
        for a in (0..18).step_by(3) {
            for p in 0..2 {
                cubes[a + p + 1] = cubes[a + p].times(&cubes[a]);
            }
        }
        CornerMoves { cubes }
    }

    pub(super) fn apply(&self, corners: &mut Corners, m: usize) {
        *corners = corners.times(&self.cubes[m % 18]);
    }
}

/// The three kinds of pieces together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Cube {
    pub ep: [u8; 24],
    pub ct: [u8; 24],
    pub corners: Corners,
}

/// The center colors of the solved cube.
pub(super) const CENTER_HOME: [u8; 24] = [
    0, 0, 0, 0, 3, 3, 3, 3, 2, 2, 2, 2, 5, 5, 5, 5, 1, 1, 1, 1, 4, 4, 4, 4,
];

impl Cube {
    pub(super) fn solved() -> Cube {
        let mut ep = [0u8; 24];
        for (i, slot) in ep.iter_mut().enumerate() {
            *slot = i as u8;
        }
        Cube {
            ep,
            ct: CENTER_HOME,
            corners: Corners::SOLVED,
        }
    }

    pub(super) fn apply(&mut self, cm: &CornerMoves, m: usize) {
        move_wings(&mut self.ep, m);
        move_centers(&mut self.ct, m);
        cm.apply(&mut self.corners, m);
    }

    /// The parity of the wing permutation.
    pub(super) fn wing_parity(&self) -> u8 {
        parity(&self.ep)
    }
}

/// Whether each pair of wings has one wing in the first twelve places, an
/// even number of them from the second half: the last phase can pair the
/// wings.
pub(super) fn wings_pairable(ep: &[u8; 24]) -> bool {
    let mut seen = 0u32;
    let mut odd = false;
    for &w in &ep[..12] {
        seen |= 1 << w;
        odd ^= w >= 12;
    }
    seen & (seen >> 12) == 0 && !odd
}

/// Whether move `m` flips the parity of the wing permutation: the quarter
/// turns of the wide moves.
pub(super) const fn flips_wing_parity(m: usize) -> bool {
    m >= 18 && m % 3 != 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cm() -> CornerMoves {
        CornerMoves::new()
    }

    #[test]
    fn four_quarter_turns_are_the_identity() {
        let cm = cm();
        for face in 0..12 {
            let mut cube = Cube::solved();
            for _ in 0..4 {
                cube.apply(&cm, face * 3);
            }
            assert_eq!(cube, Cube::solved(), "face {face}");
            cube.apply(&cm, face * 3);
            cube.apply(&cm, face * 3 + 2);
            assert_eq!(cube, Cube::solved(), "face {face}");
            cube.apply(&cm, face * 3 + 1);
            cube.apply(&cm, face * 3 + 1);
            assert_eq!(cube, Cube::solved(), "face {face}");
        }
    }

    #[test]
    fn wing_parity_follows_the_wide_quarter_turns() {
        let cm = cm();
        let mut cube = Cube::solved();
        let mut parity = 0;
        let mut state = 12345u32;
        for _ in 0..500 {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let m = (state >> 16) as usize % 36;
            cube.apply(&cm, m);
            parity ^= u8::from(flips_wing_parity(m));
            assert_eq!(cube.wing_parity(), parity);
        }
    }

    #[test]
    fn every_corner_move_is_a_proper_turn() {
        let cm = cm();
        let mut cube = Cube::solved();
        for m in 0..18 {
            cube.apply(&cm, m);
        }
        let twist: u32 = cube.corners.co.iter().map(|&c| u32::from(c)).sum();
        assert_eq!(twist % 3, 0);
    }

    #[test]
    fn the_solved_cube_has_pairable_wings() {
        assert!(wings_pairable(&Cube::solved().ep));
        let mut swapped = Cube::solved();
        swapped.ep.swap(0, 12);
        assert!(!wings_pairable(&swapped.ep));
    }
}
