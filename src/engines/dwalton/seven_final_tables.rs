//! Step 61 to 66 tables of the 7x7x7: the centres daisy solve.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube777.py (the `state_targets` and sticker
//! lists of `LookupTable777Step61`, `LookupTable777Step62`, `LookupTable777Step65`, `LookupTable777Step66`), MIT, Copyright (c) 2019 Daniel Walton; this file is
//! AGPL-3.0-or-later, see NOTICE.md. Reference sticker numbers, 49 per face
//! in the face order U L F R B D.

pub(super) const CELLS_61: [usize; 42] = [
    10, 11, 12, 16, 17, 18, 19, 20, 23, 24, 25, 26, 27, 30, 31, 32, 33, 34, 38, 39, 40, 255, 256,
    257, 261, 262, 263, 264, 265, 268, 269, 270, 271, 272, 275, 276, 277, 278, 279, 283, 284, 285,
];

pub(super) const TARGETS_61: [&str; 2] = [
    "UUUUUUUUUUUUUUUUUUUUUDDDDDDDDDDDDDDDDDDDDD",
    "DDDDUUUDDUUUDDUUUDDDDUUUUDDDUUDDDUUDDDUUUU",
];

pub(super) const CELLS_62: [usize; 42] = [
    59, 60, 61, 65, 66, 67, 68, 69, 72, 73, 74, 75, 76, 79, 80, 81, 82, 83, 87, 88, 89, 157, 158,
    159, 163, 164, 165, 166, 167, 170, 171, 172, 173, 174, 177, 178, 179, 180, 181, 185, 186, 187,
];

pub(super) const TARGETS_62: [&str; 2] = [
    "LLLLLLLLLLLLLLLLLLLLLRRRRRRRRRRRRRRRRRRRRR",
    "RRRRLLLRRLLLRRLLLRRRRLLLLRRRLLRRRLLRRRLLLL",
];

pub(super) const CELLS_65: [usize; 26] = [
    109, 115, 116, 117, 121, 122, 123, 124, 125, 129, 130, 131, 137, 207, 213, 214, 215, 219, 220,
    221, 222, 223, 227, 228, 229, 235,
];

pub(super) const TARGETS_65: [&str; 2] =
    ["FFFFFFFFFFFFFBBBBBBBBBBBBB", "BFFFBFFFBFFFBFBBBFBBBFBBBF"];

pub(super) const CELLS_66: [usize; 24] = [
    108, 109, 110, 114, 118, 121, 125, 128, 132, 136, 137, 138, 206, 207, 208, 212, 216, 219, 223,
    226, 230, 234, 235, 236,
];

pub(super) const TARGETS_66: [&str; 2] = ["BBBBBBBBBBBBFFFFFFFFFFFF", "FFFFFFFFFFFFBBBBBBBBBBBB"];
