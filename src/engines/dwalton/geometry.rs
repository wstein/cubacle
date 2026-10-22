//! Sticker numbering of the reference and the move catalogue.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver rubikscubennnsolver/RubiksCube666.py
//! (the position tables), MIT, Copyright (c) 2019 Daniel Walton; this file is
//! AGPL-3.0-or-later, see NOTICE.md. The reference numbers stickers from 1 in
//! the face order U L F R B D; the cubacle order is U R F D L B.

use crate::puzzle::{destinations, Move, PuzzleKind};

/// The cubacle face index of reference side 0 to 5 (U L F R B D).
const SIDE: [usize; 6] = [0, 4, 2, 1, 5, 3];

/// The cubacle sticker index of reference sticker `i` (from 1) on a cube of
/// `n * n` stickers per face.
pub(super) fn from_ref(n: usize, i: usize) -> usize {
    let (side, cell) = ((i - 1) / (n * n), (i - 1) % (n * n));
    SIDE[side] * n * n + cell
}

pub(super) fn from_ref_all(n: usize, list: &[usize]) -> Vec<usize> {
    list.iter().map(|&i| from_ref(n, i)).collect()
}

/// Colour bit of a face letter in a mask over the cubacle face indices.
pub(super) const fn bit(face: usize) -> u8 {
    1 << face
}
pub(super) const U: usize = 0;
pub(super) const R: usize = 1;
pub(super) const F: usize = 2;
pub(super) const D: usize = 3;
pub(super) const L: usize = 4;
pub(super) const B: usize = 5;

// fmt: 6x6x6 sets, reference numbering.
pub(super) const INNER_X_666: [usize; 24] = [
    15, 16, 21, 22, 51, 52, 57, 58, 87, 88, 93, 94, 123, 124, 129, 130, 159, 160, 165, 166, 195,
    196, 201, 202,
];
pub(super) const OUTER_X_666: [usize; 24] = [
    8, 11, 26, 29, 44, 47, 62, 65, 80, 83, 98, 101, 116, 119, 134, 137, 152, 155, 170, 173, 188,
    191, 206, 209,
];
pub(super) const LEFT_OBLIQUE_666: [usize; 24] = [
    9, 17, 28, 20, 45, 53, 64, 56, 81, 89, 100, 92, 117, 125, 136, 128, 153, 161, 172, 164, 189,
    197, 208, 200,
];
pub(super) const RIGHT_OBLIQUE_666: [usize; 24] = [
    10, 23, 27, 14, 46, 59, 63, 50, 82, 95, 99, 86, 118, 131, 135, 122, 154, 167, 171, 158, 190,
    203, 207, 194,
];

/// One move of the catalogue.
pub(super) struct MoveDef {
    pub name: String,
    pub mv: Move,
    /// Destination of every sticker of the cube.
    pub dest: Vec<usize>,
}

/// The reference's move set of a cube of size `n`: `X`, `Xw`, `3Xw` and, for
/// 7x7x7, nothing deeper, each in three turns.
pub(super) fn catalogue(kind: PuzzleKind) -> Vec<MoveDef> {
    let mut out = Vec::new();
    for face in ["U", "L", "F", "R", "B", "D"] {
        for kindname in ["", "w", "3w"] {
            for turn in ["", "'", "2"] {
                let name = match kindname {
                    "" => format!("{face}{turn}"),
                    "w" => format!("{face}w{turn}"),
                    _ => format!("3{face}w{turn}"),
                };
                let mv: Move = name.parse().expect("a valid move");
                out.push(MoveDef {
                    dest: destinations(kind, &mv),
                    name,
                    mv,
                });
            }
        }
    }
    out
}
