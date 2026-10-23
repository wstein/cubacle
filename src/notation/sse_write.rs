//! Writing SSE (Superset ENG), the canonical spelling of each turn.
//!
//! A turn is a block of layers `from..=to` counted from a face (1 is the
//! face's own layer), as in [`crate::puzzle::LayerMove`]. Spelling follows the
//! "keep it short" rules of the six pages, tried in this order:
//!
//! 1. every layer: the cube rotation `CR`;
//! 2. a block that starts at layer 1: a tier, `R`, `TR`, `T3R`;
//! 3. a block that ends at the last layer: the tier of the opposite face,
//!    turning the other way round (`N4-7R` on a 7x7 is `T4L'`);
//! 4. a block that starts at layer 2 and has two layers or more: `WR` when it
//!    is all but the outer layers, else a void twist `VR`, `V3R` (the same
//!    layers as `M2R` or `M3R` there, but `VR` is the shorter);
//! 5. the layers in the middle, the extra layer of an uneven split towards
//!    the face (see [`super::sse`]): `MR`, `M2R`, ...; on a 4x4, where `N`
//!    does not exist, a single layer that is in the middle seen from the
//!    opposite face is written from there;
//! 6. one layer: `NR`, `N4R`;
//! 7. any other block: a range, `N2-4R`.
//!
//! Slices (`S`) are never written: the writer turns one block at a time, and
//! `R L'` stays two moves. The turn suffix is nothing, `2` or `'`.

use crate::puzzle::{Face, LayerMove, Move, PuzzleKind, Turn};

use super::range::back;

fn opposite(face: Face) -> Face {
    Face::ALL[(face.index() + 3) % 6]
}

fn suffix(turn: Turn) -> &'static str {
    match turn {
        Turn::Clockwise => "",
        Turn::Half => "2",
        Turn::CounterClockwise => "'",
    }
}

/// The block of layers a plain move turns, counted from its own face.
fn block_of(mv: &Move) -> (usize, usize) {
    let depth = usize::from(mv.depth);
    if mv.wide {
        (1, depth)
    } else {
        (depth, depth)
    }
}

/// One plain move in SSE on a cube of the given kind.
pub(crate) fn write_move(kind: PuzzleKind, mv: &Move) -> String {
    let (from, to) = block_of(mv);
    spell(kind.size(), mv.face, from, to, mv.turn)
}

/// One block of layers (a compressed range) in SSE on a cube of the given kind.
pub(crate) fn write_layer_move(kind: PuzzleKind, mv: &LayerMove) -> String {
    spell(
        kind.size(),
        mv.face,
        usize::from(mv.from),
        usize::from(mv.to),
        mv.turn,
    )
}

/// The first layer of `count` layers in the middle, counted from a face.
fn middle_start(n: usize, count: usize) -> usize {
    (n - count) / 2 + 1
}

fn spell(n: usize, face: Face, from: usize, to: usize, turn: Turn) -> String {
    let letter = face.letter();
    let tail = suffix(turn);
    let count = to - from + 1;
    if from == 1 && to >= n {
        return format!("C{letter}{tail}");
    }
    if from == 1 {
        return match to {
            1 => format!("{letter}{tail}"),
            2 => format!("T{letter}{tail}"),
            _ => format!("T{to}{letter}{tail}"),
        };
    }
    if to >= n {
        return spell(n, opposite(face), 1, n + 1 - from, back(turn));
    }
    if from == 2 && count >= 2 {
        // Layers 2..=count+1: W when that is all but the outer layers, else V.
        if count + 2 == n && n >= 4 {
            return format!("W{letter}{tail}");
        }
        return match count {
            2 => format!("V{letter}{tail}"),
            _ => format!("V{count}{letter}{tail}"),
        };
    }
    if n >= 3 && count + 2 <= n {
        if from == middle_start(n, count) {
            return middle(count, letter, tail);
        }
        // Without N, a single middle layer is written from its nearer face.
        if n < 5 && n + 1 - to == middle_start(n, count) {
            return middle(count, opposite(face).letter(), suffix(back(turn)));
        }
    }
    match (count, from) {
        (1, 2) => format!("N{letter}{tail}"),
        (1, _) => format!("N{from}{letter}{tail}"),
        _ => format!("N{from}-{to}{letter}{tail}"),
    }
}

fn middle(count: usize, letter: char, tail: &str) -> String {
    if count == 1 {
        format!("M{letter}{tail}")
    } else {
        format!("M{count}{letter}{tail}")
    }
}
