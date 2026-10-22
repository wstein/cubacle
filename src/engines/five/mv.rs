//! Move numbers of the five-phase solver.
//!
//! Ported from cs0x7f/cube555 src/Util.java (the `Ux1` ... `bx3` constants),
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md. A move is `face * 3 + turn` for the outer layer and 18 more
//! for the two outer layers together (`Uw`), faces in the order U R F D L B
//! and turns 1 (clockwise), 2 (half), 3 (counter-clockwise), written `U1`,
//! `U2`, `U3` and `UW1`, `UW2`, `UW3`.

// The table lists every move constant of the Java solver, in its order, so a
// reader can check it against the original; each phase uses only some of them.
#![allow(dead_code)]

pub(super) const U1: u8 = 0;
pub(super) const U2: u8 = 1;
pub(super) const U3: u8 = 2;
pub(super) const R1: u8 = 3;
pub(super) const R2: u8 = 4;
pub(super) const R3: u8 = 5;
pub(super) const F1: u8 = 6;
pub(super) const F2: u8 = 7;
pub(super) const F3: u8 = 8;
pub(super) const D1: u8 = 9;
pub(super) const D2: u8 = 10;
pub(super) const D3: u8 = 11;
pub(super) const L1: u8 = 12;
pub(super) const L2: u8 = 13;
pub(super) const L3: u8 = 14;
pub(super) const B1: u8 = 15;
pub(super) const B2: u8 = 16;
pub(super) const B3: u8 = 17;
pub(super) const UW1: u8 = 18;
pub(super) const UW2: u8 = 19;
pub(super) const UW3: u8 = 20;
pub(super) const RW1: u8 = 21;
pub(super) const RW2: u8 = 22;
pub(super) const RW3: u8 = 23;
pub(super) const FW1: u8 = 24;
pub(super) const FW2: u8 = 25;
pub(super) const FW3: u8 = 26;
pub(super) const DW1: u8 = 27;
pub(super) const DW2: u8 = 28;
pub(super) const DW3: u8 = 29;
pub(super) const LW1: u8 = 30;
pub(super) const LW2: u8 = 31;
pub(super) const LW3: u8 = 32;
pub(super) const BW1: u8 = 33;
pub(super) const BW2: u8 = 34;
pub(super) const BW3: u8 = 35;

/// Number of moves.
pub(super) const COUNT: usize = 36;

/// For each last move (and one extra entry for none), the moves that need
/// not be tried next: the same axis, and a later move on the same pair of
/// opposite faces. Ported from `Util.genSkipMoves`, including its `int`
/// shift: bit `m` is `1 << (m & 31)` sign-extended, so with more than 32
/// moves (the first phase) moves 32 to 35 land on bits 0 to 3, and move 31
/// also sets bits 32 and up. Kept as is, because the reduction lengths of the
/// Java solver depend on it.
pub(super) fn skip_masks(valid: &[u8]) -> Vec<u64> {
    let mut masks = vec![0u64; valid.len() + 1];
    for (last, mask) in masks.iter_mut().enumerate().take(valid.len()) {
        let la = valid[last] / 3;
        for (mv, &code) in valid.iter().enumerate() {
            let axis = code / 3;
            if axis == la || axis % 3 == la % 3 && axis >= la {
                *mask |= i64::from(1i32.wrapping_shl(mv as u32)) as u64;
            }
        }
    }
    masks
}

/// Bit `i + 1` for every move `i` that is a clockwise quarter turn
/// (`Util.genNextAxis`).
pub(super) fn next_axis(valid: &[u8]) -> u64 {
    valid
        .iter()
        .enumerate()
        .filter(|(_, &code)| code % 3 == 0)
        .fold(0, |bits, (i, _)| bits | 1u64 << (i + 1))
}
