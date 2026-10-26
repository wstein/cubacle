//! The 5x5x5 as pieces: centers, wings and middle edges (the corners are
//! checked on input and left to the 3x3x3 solver afterwards).
//!
//! Ported from cs0x7f/cube555 src/CubieCube.java, MIT, Copyright (c) 2023
//! Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! Sticker numbering is that of `Facelets`: faces U R F D L B, 25 stickers
//! each, row by row. The 24 x-centers and 24 t-centers, the 24 wings and the
//! 12 middle edges are numbered as in the Java solver.

use crate::reduction::comb::{parity, swap4};

const fn u(k: usize) -> usize {
    k - 1
}
const fn r(k: usize) -> usize {
    24 + k
}
const fn f(k: usize) -> usize {
    49 + k
}
const fn d(k: usize) -> usize {
    74 + k
}
const fn l(k: usize) -> usize {
    99 + k
}
const fn b(k: usize) -> usize {
    124 + k
}

/// Sticker of each t-center slot.
const TCENTER: [usize; 24] = [
    u(8),
    u(14),
    u(18),
    u(12),
    d(8),
    d(14),
    d(18),
    d(12),
    f(8),
    f(14),
    f(18),
    f(12),
    b(8),
    b(14),
    b(18),
    b(12),
    r(8),
    r(14),
    r(18),
    r(12),
    l(8),
    l(14),
    l(18),
    l(12),
];

/// Sticker of each x-center slot.
const XCENTER: [usize; 24] = [
    u(7),
    u(9),
    u(19),
    u(17),
    d(7),
    d(9),
    d(19),
    d(17),
    f(7),
    f(9),
    f(19),
    f(17),
    b(7),
    b(9),
    b(19),
    b(17),
    r(7),
    r(9),
    r(19),
    r(17),
    l(7),
    l(9),
    l(19),
    l(17),
];

const MEDGE: [[usize; 2]; 12] = [
    [u(23), f(3)],
    [u(3), b(3)],
    [d(23), b(23)],
    [d(3), f(23)],
    [u(11), l(3)],
    [u(15), r(3)],
    [d(15), r(23)],
    [d(11), l(23)],
    [l(15), f(11)],
    [l(11), b(15)],
    [r(15), b(11)],
    [r(11), f(15)],
];

const WEDGE: [[usize; 2]; 24] = [
    [u(22), f(2)],
    [u(4), b(2)],
    [d(22), b(24)],
    [d(4), f(24)],
    [u(6), l(2)],
    [u(20), r(2)],
    [d(20), r(24)],
    [d(6), l(24)],
    [l(20), f(16)],
    [l(6), b(10)],
    [r(20), b(16)],
    [r(6), f(10)],
    [f(4), u(24)],
    [b(4), u(2)],
    [b(22), d(24)],
    [f(22), d(2)],
    [l(4), u(16)],
    [r(4), u(10)],
    [r(22), d(10)],
    [l(22), d(16)],
    [f(6), l(10)],
    [b(20), l(16)],
    [b(6), r(10)],
    [f(20), r(16)],
];

const CORNER: [[usize; 3]; 8] = [
    [u(25), r(1), f(5)],
    [u(21), f(1), l(5)],
    [u(1), l(1), b(5)],
    [u(5), b(1), r(5)],
    [d(5), f(25), r(21)],
    [d(1), l(25), f(21)],
    [d(21), b(25), l(21)],
    [d(25), r(25), b(21)],
];

/// Why a cube is not a 5x5x5 state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Invalid {
    Colors,
    TCenters,
    XCenters,
    MiddleEdges,
    Wings,
    Corners,
    EdgeFlip,
    CornerTwist,
    Parity,
}

impl Invalid {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Invalid::Colors => "a color does not appear 25 times, or a center repeats a color",
            Invalid::TCenters => "the t-centers do not hold four stickers of each color",
            Invalid::XCenters => "the x-centers do not hold four stickers of each color",
            Invalid::MiddleEdges => "the middle edges are not the twelve edges of a cube",
            Invalid::Wings => "the wings are not the twenty-four wings of a cube",
            Invalid::Corners => "the corners are not the eight corners of a cube",
            Invalid::EdgeFlip => "a middle edge is flipped",
            Invalid::CornerTwist => "a corner is twisted",
            Invalid::Parity => "corner and middle-edge parity differ",
        }
    }
}

/// Centers, wings and middle edges. A middle edge is `piece << 1 | flip`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cubie {
    pub tcenter: [i32; 24],
    pub xcenter: [i32; 24],
    pub medge: [i32; 12],
    pub wedge: [i32; 24],
}

/// Color index (0 to 5, as the face of the sticker's own center) of the
/// center slot `i` on a solved cube.
pub(crate) const fn solved_color(sticker: usize) -> i32 {
    (sticker / 25) as i32
}

impl Cubie {
    pub(crate) fn solved() -> Cubie {
        let mut cube = Cubie {
            tcenter: [0; 24],
            xcenter: [0; 24],
            medge: [0; 12],
            wedge: [0; 24],
        };
        for i in 0..24 {
            cube.tcenter[i] = solved_color(TCENTER[i]);
            cube.xcenter[i] = solved_color(XCENTER[i]);
            cube.wedge[i] = i as i32;
        }
        for i in 0..12 {
            cube.medge[i] = (i as i32) << 1;
        }
        cube
    }

    /// Reads the pieces from sticker colors (indices of the face whose
    /// center shows that color). The corners are only validated.
    #[allow(clippy::needless_range_loop)] // index pairs mirror the reference
    pub(crate) fn from_colors(face: &[u8; 150]) -> Result<Cubie, Invalid> {
        let mut cube = Cubie::solved();
        let mut color_count = [0usize; 6];
        for &c in face {
            color_count[usize::from(c)] += 1;
        }
        if color_count.iter().any(|&n| n != 25) {
            return Err(Invalid::Colors);
        }
        let at = |i: usize| i32::from(face[i]);
        let (mut tcount, mut xcount) = (0u32, 0u32);
        for i in 0..24 {
            cube.tcenter[i] = at(TCENTER[i]);
            cube.xcenter[i] = at(XCENTER[i]);
            tcount += 1 << (cube.tcenter[i] << 2);
            xcount += 1 << (cube.xcenter[i] << 2);
        }
        let (mut medge_seen, mut flip_sum) = (0u32, 0);
        for i in 0..12 {
            for j in 0..12 {
                let (a, bb) = (at(MEDGE[i][0]), at(MEDGE[i][1]));
                let (ja, jb) = (solved_color(MEDGE[j][0]), solved_color(MEDGE[j][1]));
                if a == ja && bb == jb || a == jb && bb == ja {
                    let flip = i32::from(a != ja);
                    cube.medge[i] = (j as i32) << 1 | flip;
                    medge_seen |= 1 << j;
                    flip_sum ^= flip;
                    break;
                }
            }
        }
        let mut wing_seen = 0u32;
        for i in 0..24 {
            for j in 0..24 {
                if at(WEDGE[i][0]) == solved_color(WEDGE[j][0])
                    && at(WEDGE[i][1]) == solved_color(WEDGE[j][1])
                {
                    cube.wedge[i] = j as i32;
                    wing_seen |= 1 << j;
                    break;
                }
            }
        }
        let (mut cp, mut corner_seen, mut twist) = ([0i32; 8], 0u32, 0);
        for i in 0..8 {
            let mut ori = 0;
            while ori < 3 && at(CORNER[i][ori]) != 0 && at(CORNER[i][ori]) != 3 {
                ori += 1;
            }
            if ori == 3 {
                return Err(Invalid::Corners);
            }
            let c0 = at(CORNER[i][ori]);
            let c1 = at(CORNER[i][(ori + 1) % 3]);
            let c2 = at(CORNER[i][(ori + 2) % 3]);
            for j in 0..8 {
                // All three colors, in turning order: the side pair alone also
                // fits the mirror image of the piece (U/D color exchanged).
                if c0 == solved_color(CORNER[j][0])
                    && c1 == solved_color(CORNER[j][1])
                    && c2 == solved_color(CORNER[j][2])
                {
                    cp[i] = j as i32;
                    twist += ori;
                    corner_seen |= 1 << j;
                    break;
                }
            }
        }
        if tcount != 0x44_4444 {
            return Err(Invalid::TCenters);
        } else if xcount != 0x44_4444 {
            return Err(Invalid::XCenters);
        } else if medge_seen != 0xfff {
            return Err(Invalid::MiddleEdges);
        } else if wing_seen != 0xff_ffff {
            return Err(Invalid::Wings);
        } else if corner_seen != 0xff {
            return Err(Invalid::Corners);
        } else if flip_sum != 0 {
            return Err(Invalid::EdgeFlip);
        } else if twist % 3 != 0 {
            return Err(Invalid::CornerTwist);
        }
        let ep: Vec<i32> = cube.medge.iter().map(|e| e >> 1).collect();
        if parity(&ep) != parity(&cp) {
            return Err(Invalid::Parity);
        }
        Ok(cube)
    }

    /// Performs a move (0 to 35, see `mv`) on the pieces; corners excluded.
    pub(crate) fn do_move(&mut self, mv: u8) {
        move_centers(&mut self.xcenter, &mut self.tcenter, mv);
        move_edges(&mut self.medge, &mut self.wedge, mv);
    }

    /// The product `a * b` as the Java solver composes cubes.
    pub(crate) fn mult(a: &Cubie, b: &Cubie) -> Cubie {
        let mut p = a.clone();
        for i in 0..12 {
            p.medge[i] = a.medge[(b.medge[i] >> 1) as usize] ^ (b.medge[i] & 1);
        }
        for i in 0..24 {
            p.tcenter[i] = a.tcenter[b.tcenter[i] as usize];
            p.xcenter[i] = a.xcenter[b.xcenter[i] as usize];
            p.wedge[i] = a.wedge[b.wedge[i] as usize];
        }
        p
    }
}

/// The effect of a move on the x-centers and t-centers.
pub(super) fn move_centers(x: &mut [i32], t: &mut [i32], mv: u8) {
    let pow = i32::from(mv % 3);
    let wide = mv >= 18;
    match (mv % 18) / 3 {
        0 => {
            if wide {
                swap4(x, [8, 20, 12, 16], pow, false);
                swap4(x, [9, 21, 13, 17], pow, false);
                swap4(t, [8, 20, 12, 16], pow, false);
            }
            swap4(x, [0, 1, 2, 3], pow, false);
            swap4(t, [0, 1, 2, 3], pow, false);
        }
        1 => {
            if wide {
                swap4(x, [1, 15, 5, 9], pow, false);
                swap4(x, [2, 12, 6, 10], pow, false);
                swap4(t, [1, 15, 5, 9], pow, false);
            }
            swap4(x, [16, 17, 18, 19], pow, false);
            swap4(t, [16, 17, 18, 19], pow, false);
        }
        2 => {
            if wide {
                swap4(x, [2, 19, 4, 21], pow, false);
                swap4(x, [3, 16, 5, 22], pow, false);
                swap4(t, [2, 19, 4, 21], pow, false);
            }
            swap4(x, [8, 9, 10, 11], pow, false);
            swap4(t, [8, 9, 10, 11], pow, false);
        }
        3 => {
            if wide {
                swap4(x, [10, 18, 14, 22], pow, false);
                swap4(x, [11, 19, 15, 23], pow, false);
                swap4(t, [10, 18, 14, 22], pow, false);
            }
            swap4(x, [4, 5, 6, 7], pow, false);
            swap4(t, [4, 5, 6, 7], pow, false);
        }
        4 => {
            if wide {
                swap4(x, [0, 8, 4, 14], pow, false);
                swap4(x, [3, 11, 7, 13], pow, false);
                swap4(t, [3, 11, 7, 13], pow, false);
            }
            swap4(x, [20, 21, 22, 23], pow, false);
            swap4(t, [20, 21, 22, 23], pow, false);
        }
        _ => {
            if wide {
                swap4(x, [1, 20, 7, 18], pow, false);
                swap4(x, [0, 23, 6, 17], pow, false);
                swap4(t, [0, 23, 6, 17], pow, false);
            }
            swap4(x, [12, 13, 14, 15], pow, false);
            swap4(t, [12, 13, 14, 15], pow, false);
        }
    }
}

/// The effect of a move on the middle edges and the wings.
pub(super) fn move_edges(m: &mut [i32], w: &mut [i32], mv: u8) {
    let pow = i32::from(mv % 3);
    let wide = mv >= 18;
    match (mv % 18) / 3 {
        0 => {
            if wide {
                swap4(w, [9, 22, 11, 20], pow, false);
            }
            swap4(m, [0, 4, 1, 5], pow, false);
            swap4(w, [0, 4, 1, 5], pow, false);
            swap4(w, [12, 16, 13, 17], pow, false);
        }
        1 => {
            if wide {
                swap4(w, [1, 14, 3, 12], pow, false);
            }
            swap4(m, [5, 10, 6, 11], pow, true);
            swap4(w, [5, 22, 6, 23], pow, false);
            swap4(w, [17, 10, 18, 11], pow, false);
        }
        2 => {
            if wide {
                swap4(w, [5, 18, 7, 16], pow, false);
            }
            swap4(m, [0, 11, 3, 8], pow, false);
            swap4(w, [0, 11, 3, 8], pow, false);
            swap4(w, [12, 23, 15, 20], pow, false);
        }
        3 => {
            if wide {
                swap4(w, [8, 23, 10, 21], pow, false);
            }
            swap4(m, [2, 7, 3, 6], pow, false);
            swap4(w, [2, 7, 3, 6], pow, false);
            swap4(w, [14, 19, 15, 18], pow, false);
        }
        4 => {
            if wide {
                swap4(w, [0, 15, 2, 13], pow, false);
            }
            swap4(m, [4, 8, 7, 9], pow, true);
            swap4(w, [4, 20, 7, 21], pow, false);
            swap4(w, [16, 8, 19, 9], pow, false);
        }
        _ => {
            if wide {
                swap4(w, [4, 19, 6, 17], pow, false);
            }
            swap4(m, [1, 9, 2, 10], pow, false);
            swap4(w, [1, 9, 2, 10], pow, false);
            swap4(w, [13, 21, 14, 22], pow, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_solved_cube_reads_back_as_solved() {
        let mut face = [0u8; 150];
        for (i, c) in face.iter_mut().enumerate() {
            *c = (i / 25) as u8;
        }
        assert_eq!(Cubie::from_colors(&face), Ok(Cubie::solved()));
    }

    #[test]
    fn mirrored_corners_are_not_a_cube() {
        // Every corner shows its two side colors exchanged: the corner set is
        // the mirror image of the solved one, which no sequence of turns makes.
        let mut face = [0u8; 150];
        for (i, c) in face.iter_mut().enumerate() {
            *c = (i / 25) as u8;
        }
        for corner in CORNER {
            face.swap(corner[1], corner[2]);
        }
        assert_eq!(Cubie::from_colors(&face), Err(Invalid::Corners));
    }

    #[test]
    fn every_move_has_order_four_or_two_and_inverse_turns_cancel() {
        for mv in 0..36u8 {
            let mut cube = Cubie::solved();
            cube.do_move(mv);
            assert_ne!(cube, Cubie::solved(), "move {mv}");
            let base = mv - mv % 3;
            let inverse = base + (2 - mv % 3);
            let mut back = cube.clone();
            back.do_move(if mv % 3 == 1 { mv } else { inverse });
            assert_eq!(back, Cubie::solved(), "move {mv}");
        }
    }
}
