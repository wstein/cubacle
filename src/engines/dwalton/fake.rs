//! The 4x4x4 and 5x5x5 hidden in a bigger cube.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCubeNNNEvenEdges.py (`pair_inside_edges_via_444`,
//! `pair_edge_orbit_via_555`) and RubiksCubeNNNOddEdges.py, MIT, Copyright (c)
//! 2019 Daniel Walton; this file is AGPL-3.0-or-later, see NOTICE.md. The
//! fake cubes are handed to the 4x4x4 and 5x5x5 engines, and their moves are
//! mapped back to the layers of the big cube.

use crate::puzzle::{Face, Facelets, Move, PuzzleKind};

/// Picks stickers (1-based cell numbers of the big cube, per face) for the
/// cells of a fake cube; cells without a source show the colour of their face.
fn build(
    big: &Facelets,
    fake_kind: PuzzleKind,
    source: impl Fn(usize) -> Option<usize>,
) -> Facelets {
    let n = big.kind().size();
    let m = fake_kind.size();
    let mut out = Vec::with_capacity(6 * m * m);
    for face in 0..6 {
        for cell in 1..=m * m {
            out.push(match source(cell) {
                Some(src) => big.stickers()[face * n * n + src - 1],
                None => Face::ALL[face],
            });
        }
    }
    Facelets::from_stickers(out).expect("every colour appears as often as it should")
}

/// The 4x4x4 of an even cube: its corners and the inner wing orbit.
pub(super) fn four_of(big: &Facelets) -> Facelets {
    let n = big.kind().size();
    let half = n / 2;
    let map = |cell: usize| match cell {
        1 => Some(1),
        4 => Some(n),
        13 => Some(n * n - n + 1),
        16 => Some(n * n),
        2 => Some(half),
        3 => Some(half + 1),
        5 => Some(n * (half - 1) + 1),
        9 => Some(n * half + 1),
        8 => Some(n * half),
        12 => Some(n * (half + 1)),
        14 => Some(n * n - half),
        15 => Some(n * n - half + 1),
        _ => None,
    };
    build(big, PuzzleKind::Cube4, map)
}

/// The 5x5x5 for pairing the wings of `orbit` (0 is the outermost) of an
/// even or odd cube: its corners, the centres (only for orbit 0 of an odd
/// cube, else solved) and the wings of the orbit beside the middle edge.
pub(super) fn five_of(big: &Facelets, orbit: usize) -> Facelets {
    let n = big.kind().size();
    let odd = n % 2 == 1;
    let (half, max_orbit) = if odd {
        (n.div_ceil(2), (n - 3) / 2)
    } else {
        (n / 2, (n - 2) / 2 - 1)
    };
    let gap = max_orbit - orbit;
    // Row 1 of the wings: left, middle, right.
    let (r1c2, r1c1, r1c3) = if odd {
        (half, half - gap, half + gap)
    } else {
        (half, half - gap, half + gap + 1)
    };
    let r2c1 = (orbit + 1) * n + 1;
    let r3c1 = n * (half - 1) + 1;
    let r4c1 = n * n - (orbit + 2) * n + 1;
    let far = (n - 1) * n;
    // Centres.
    let (c, show_centres) = if odd {
        let hc = (n - 2) / 2;
        let r1 = n + 2;
        let r2 = (n * n).div_ceil(2);
        let r3 = n * n - n - 1;
        (
            [
                [r1, r1 + hc, r1 + 2 * hc],
                [r2 - hc, r2, r2 + hc],
                [r3 - 2 * hc, r3 - hc, r3],
            ],
            orbit == 0,
        )
    } else {
        let r1 = half + n;
        let r2 = n * n / 2 + 2;
        let r3 = n * n - half - n;
        (
            [
                [r1 - gap, r1, r1 + gap + 1],
                [r2, r2 + half - 1, r2 + n - 3],
                [r3 - gap, r3, r3 + gap + 1],
            ],
            true,
        )
    };
    let map = |cell: usize| match cell {
        1 => Some(1),
        5 => Some(n),
        21 => Some(n * n - n + 1),
        25 => Some(n * n),
        7 | 8 | 9 | 12 | 13 | 14 | 17 | 18 | 19 if !show_centres => None,
        7 => Some(c[0][0]),
        8 => Some(c[0][1]),
        9 => Some(c[0][2]),
        12 => Some(c[1][0]),
        13 => Some(c[1][1]),
        14 => Some(c[1][2]),
        17 => Some(c[2][0]),
        18 => Some(c[2][1]),
        19 => Some(c[2][2]),
        2 => Some(r1c1),
        3 => Some(r1c2),
        4 => Some(r1c3),
        6 => Some(r2c1),
        10 => Some(r2c1 + n - 1),
        11 => Some(r3c1),
        15 => Some(r3c1 + n - 1),
        16 => Some(r4c1),
        20 => Some(r4c1 + n - 1),
        22 => Some(r1c1 + far),
        23 => Some(r1c2 + far),
        24 => Some(r1c3 + far),
        _ => None,
    };
    build(big, PuzzleKind::Cube5, map)
}

/// A move of the fake cube as a move of the big cube: its second layer is
/// layer `depth` of the big cube (the 4x4x4 wide turn covers `depth` layers).
pub(super) fn lift(mv: &Move, depth: u8) -> Move {
    let mut out = *mv;
    if mv.depth == 2 {
        out.depth = depth;
    }
    out
}

/// The 3x3x3 of a reduced cube.
pub(super) fn three_of(big: &Facelets) -> Facelets {
    let n = big.kind().size();
    let at = [0, 1, n - 1];
    let mut out = Vec::with_capacity(54);
    for face in 0..6 {
        for row in 0..3 {
            for col in 0..3 {
                let (r, c) = (at[row], at[col]);
                out.push(if row == 1 && col == 1 {
                    Face::ALL[face]
                } else {
                    big.stickers()[face * n * n + r * n + c]
                });
            }
        }
    }
    Facelets::from_stickers(out).expect("nine stickers of each colour")
}
