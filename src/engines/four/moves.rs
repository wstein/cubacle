//! Move numbers and the rules that keep move sequences canonical.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Moves.java, MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! A move is `face * 3 + turn`; the faces are `U R F D L B` and then the
//! wide turns `u r f d l b` (two layers), the turns 0 (clockwise), 1 (half)
//! and 2 (counter-clockwise). The three phases use subsets of the 36 moves;
//! [`MoveSet`] lists one subset by move number, tells which moves may not
//! follow which, and where to jump to skip the powers of a refused face.

/// "No move yet": the row of the tables for the start of a sequence.
pub(super) const EOM: usize = 36;

pub(super) const UX1: usize = 18;
pub(super) const UX2: usize = 19;
pub(super) const RX1: usize = 21;
pub(super) const FX1: usize = 24;
pub(super) const DX2: usize = 28;
pub(super) const DX3: usize = 29;
pub(super) const LX3: usize = 32;
pub(super) const BX3: usize = 35;

/// The moves of phase 2, as move numbers (and "none" at the end).
pub(super) const MOVE2STD: [usize; 29] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 21, 22, 23, 25, 28, 30, 31,
    32, 34, EOM,
];

/// The moves of phase 3, as move numbers (and "none" at the end).
pub(super) const MOVE3STD: [usize; 21] = [
    0, 1, 2, 4, 6, 7, 8, 9, 10, 11, 13, 15, 16, 17, 19, 22, 25, 28, 31, 34, EOM,
];

/// Whether move `j` may not follow move `i`: same face, or the same axis
/// in the wrong order (so that `U D` is tried and `D U` is not).
const fn refused(i: usize, j: usize) -> bool {
    i != EOM && (i / 3 == j / 3 || (i / 3 % 3 == j / 3 % 3 && i > j))
}

/// One move subset: `N` moves (and the row for "no move" makes `R = N + 1`).
pub(super) struct MoveSet<const N: usize, const R: usize> {
    /// The number of each move of the subset in the 36 moves.
    pub std: [usize; R],
    /// `blocked[last][m]`: move `m` may not follow move `last`.
    pub blocked: [[bool; N]; R],
    /// `skip[m]`: the last move of the run of `m`'s face, to jump over when
    /// `m` is refused or cannot lead anywhere.
    pub skip: [usize; N],
}

const fn move_set<const N: usize, const R: usize>(std: [usize; R]) -> MoveSet<N, R> {
    let mut blocked = [[false; N]; R];
    let mut skip = [0usize; N];
    let mut i = 0;
    while i < R {
        let mut j = 0;
        while j < N {
            blocked[i][j] = refused(std[i], std[j]);
            j += 1;
        }
        i += 1;
    }
    let mut i = 0;
    while i < N {
        skip[i] = N;
        let mut j = i;
        while j < N {
            if !blocked[i][j] {
                skip[i] = j - 1;
                break;
            }
            j += 1;
        }
        i += 1;
    }
    MoveSet { std, blocked, skip }
}

/// The 28 moves of the tables of phase 2 (the search uses the first 23).
pub(super) const SET2: MoveSet<28, 29> = move_set(MOVE2STD);
/// The 20 moves of the tables of phase 3 (the search uses the first 17).
pub(super) const SET3: MoveSet<20, 21> = move_set(MOVE3STD);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_jumps_match_the_reference() {
        // Printed by the reference (skipAxis2, skipAxis3).
        assert_eq!(
            SET2.skip,
            [
                2, 2, 2, 5, 5, 5, 8, 8, 8, 11, 11, 11, 14, 14, 14, 17, 17, 17, 18, 21, 21, 21, 22,
                23, 26, 26, 26, 28
            ]
        );
        assert_eq!(
            SET3.skip,
            [2, 2, 2, 3, 6, 6, 6, 9, 9, 9, 10, 13, 13, 13, 14, 15, 16, 17, 18, 20]
        );
    }

    #[test]
    fn a_face_does_not_follow_itself_or_its_opposite_in_the_wrong_order() {
        assert!(SET2.blocked[0][1]); // U then U2
        assert!(SET2.blocked[9][0]); // D then U
        assert!(!SET2.blocked[0][9]); // U then D
        assert!(!SET2.blocked[28][5]); // nothing before: free
    }
}
