//! From the moves of the model cube to the moves of the real one.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/FullCube.java (`getMoveString`,
//! `move2rot`), MIT, Copyright (c) 2023 Chen Shuang; this file is
//! AGPL-3.0-or-later, see NOTICE.md.
//!
//! The model cube is turned as a whole by the first phase's extra moves and
//! by every wide turn of D, L and B (the model makes those, the real cube
//! makes the wide turn of the opposite face and a rotation). The map from
//! model moves to real moves follows that rotation through the 48 symmetries.

use super::sym::Syms;
use crate::puzzle::{Face, Move, Turn};

/// The rotation (a symmetry number) a wide D, L or B turn leaves behind, for
/// `dx1` to `bx3`.
const MOVE2ROT: [usize; 9] = [35, 1, 34, 2, 4, 6, 22, 5, 19];

/// The real moves (numbers 0 to 26) for the model moves `all`: the first
/// `length1` stay, the next two are skipped if `add1`, the rest are mapped
/// starting from the symmetry `sym`.
pub(super) fn to_real(syms: &Syms, all: &[u8], length1: usize, add1: bool, sym: usize) -> Vec<u8> {
    let mut out: Vec<u8> = all[..length1].to_vec();
    let mut sym = sym;
    for &m in &all[length1 + if add1 { 2 } else { 0 }..] {
        let mapped = usize::from(syms.mv[sym][usize::from(m)]);
        if mapped >= 27 {
            out.push((mapped - 9) as u8);
            sym = syms.mult(sym, MOVE2ROT[mapped - 27]);
        } else {
            out.push(mapped as u8);
        }
    }
    out
}

/// Move number 0 to 26 as a move of the library.
pub(super) fn to_move(code: u8) -> Move {
    let face = Face::ALL[usize::from(code / 3 % 6)];
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][usize::from(code % 3)];
    if code >= 18 {
        Move::new(face, 2, true, turn)
    } else {
        Move::new(face, 1, false, turn)
    }
}

/// The move number (0 to 17) of a turn of one outer face, if it is one.
pub(super) fn of_face_turn(m: &Move) -> Option<u8> {
    if m.depth != 1 || m.wide {
        return None;
    }
    let turn = match m.turn {
        Turn::Clockwise => 0,
        Turn::Half => 1,
        Turn::CounterClockwise => 2,
    };
    Some(m.face as u8 * 3 + turn)
}
