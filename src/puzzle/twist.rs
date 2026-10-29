//! The twist of the six face centers of a 3x3x3 (a picture cube).
//!
//! Convention, version 1 (`TWIST_CONVENTION`): the twist vector is
//! `t = (U, R, F, D, L, B)`, each entry in Z4. Entry `f` counts the clockwise
//! quarter turns, seen from outside the face, that the picture on center `f`
//! is turned from its solved orientation. In the solved orientation the up
//! direction of the picture is back for U, front for D and up for the four
//! side faces (the top of the cross net). The move `f` adds 1 to `t[f]`, `f2`
//! adds 2, `f'` adds 3; no other center is touched by an outer face move.
//! The sum of `t` has the parity of the corner permutation (half of the 4096
//! vectors belong to a cube position).
//!
//! A whole-cube rotation costs no move. It carries the centers to other
//! faces, so it moves the entries of `t` and adds a fixed turn to each; the
//! tracker below computes both from the geometry of the picture's up vector,
//! the same way for rotations and for the middle slices.

use std::fmt;
use std::str::FromStr;

use super::twist_track::Tracker;
use super::Face;

/// The version of the twist convention, printed in certificates.
pub const TWIST_CONVENTION: &str = "center-twist-1 (t=URFDLB, clockwise quarters seen from outside, up: U back, D front, sides up)";

/// Why a twist text or a replay was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwistError {
    /// The text is no list of six twists.
    Syntax(String),
    /// The move or cube is outside what the tracker supports.
    Unsupported(String),
}

impl fmt::Display for TwistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwistError::Syntax(text) | TwistError::Unsupported(text) => f.write_str(text),
        }
    }
}

impl std::error::Error for TwistError {}

/// The twist of the six centers, each 0 to 3, in the order `URFDLB`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CenterTwist([u8; 6]);

impl CenterTwist {
    /// All six pictures upright.
    pub const UPRIGHT: CenterTwist = CenterTwist([0; 6]);

    /// The vector `t`, each entry taken mod 4.
    pub fn new(t: [u8; 6]) -> CenterTwist {
        CenterTwist(t.map(|x| x & 3))
    }

    pub fn get(&self, face: Face) -> u8 {
        self.0[face.index()]
    }

    pub fn as_array(&self) -> [u8; 6] {
        self.0
    }

    /// Whether the sum of the six twists is even. It must equal the parity of
    /// the corner permutation of the cube the twist belongs to.
    pub fn sum_is_even(&self) -> bool {
        self.0.iter().map(|&x| u32::from(x)).sum::<u32>() % 2 == 0
    }

    /// Turns the picture of `face` by `quarters` clockwise quarters.
    pub fn turned(mut self, face: Face, quarters: u8) -> CenterTwist {
        let at = &mut self.0[face.index()];
        *at = (*at + quarters) & 3;
        self
    }

    /// The twist seen on each face after a whole-cube rotation, given as text
    /// of `x y z` turns (`x`, `x'`, `x2`, separated by spaces): each picture
    /// goes with its center to another face.
    pub fn rotated(&self, word: &str) -> Result<CenterTwist, TwistError> {
        let mut tracker = Tracker::from_twist(*self);
        tracker.rotate_word(word)?;
        Ok(tracker.read())
    }
}

impl fmt::Display for CenterTwist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [u, r, ff, d, l, b] = self.0;
        write!(f, "{u},{r},{ff},{d},{l},{b}")
    }
}

impl FromStr for CenterTwist {
    type Err = TwistError;

    /// Six numbers 0 to 3 separated by commas (spaces are ignored): `0,1,2,0,3,0`.
    fn from_str(text: &str) -> Result<CenterTwist, TwistError> {
        let bad = || {
            TwistError::Syntax(format!(
                "a center twist is six numbers 0 to 3 for U,R,F,D,L,B separated by commas, like 0,1,2,0,3,0 (not {text:?})"
            ))
        };
        let parts: Vec<&str> = text.split(',').map(str::trim).collect();
        if parts.len() != 6 {
            return Err(bad());
        }
        let mut t = [0u8; 6];
        for (slot, part) in t.iter_mut().zip(&parts) {
            *slot = match *part {
                "0" => 0,
                "1" => 1,
                "2" => 2,
                "3" => 3,
                _ => return Err(bad()),
            };
        }
        Ok(CenterTwist(t))
    }
}

/// The twists each center accepts: bit `k` of entry `f` set means the center
/// `f` may show twist `k`. `FREE` accepts everything.
pub type TwistMasks = [u8; 6];

/// Every twist accepted on every center.
pub const FREE: TwistMasks = [0b1111; 6];

/// Reads a twist condition: six items separated by commas, each a twist 0 to
/// 3, several joined with `|` (`0|2`), or `X` or `?` for any: `0,X,2,X,X,X`.
pub fn parse_twist_masks(text: &str) -> Result<TwistMasks, TwistError> {
    let bad = |why: &str| {
        TwistError::Syntax(format!(
            "a twist condition is six items for U,R,F,D,L,B separated by commas, each 0 to 3 (several joined by |) or X for any, like 0,X,2|3,X,X,X ({why}; got {text:?})"
        ))
    };
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if parts.len() != 6 {
        return Err(bad("not six items"));
    }
    let mut masks = [0u8; 6];
    for (slot, part) in masks.iter_mut().zip(&parts) {
        if matches!(*part, "X" | "x" | "?") {
            *slot = 0b1111;
            continue;
        }
        for item in part.split('|') {
            match item.trim() {
                "0" => *slot |= 1,
                "1" => *slot |= 2,
                "2" => *slot |= 4,
                "3" => *slot |= 8,
                _ => return Err(bad("an item is no twist")),
            }
        }
    }
    Ok(masks)
}

/// The text form of a twist condition, the inverse of [`parse_twist_masks`].
pub fn twist_masks_text(masks: &TwistMasks) -> String {
    masks
        .iter()
        .map(|&mask| {
            if mask & 0b1111 == 0b1111 {
                "X".to_owned()
            } else {
                (0..4)
                    .filter(|k| mask >> k & 1 == 1)
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::twist_track::track_moves;
    use crate::puzzle::{MoveSequence, Turn};

    fn seq(text: &str) -> MoveSequence {
        text.parse().unwrap()
    }

    #[test]
    fn a_face_move_adds_to_its_own_center_only() {
        for (i, face) in Face::ALL.into_iter().enumerate() {
            for (suffix, add) in [("", 1), ("2", 2), ("'", 3)] {
                let moves = seq(&format!("{}{suffix}", face.letter()));
                let tracked = track_moves(CenterTwist::UPRIGHT, &moves).unwrap();
                let mut want = [0u8; 6];
                want[i] = add;
                assert_eq!(tracked.twist.as_array(), want, "{moves}");
                assert!(tracked.frame.is_empty());
            }
        }
    }

    #[test]
    fn the_sum_parity_is_the_quarter_turn_parity() {
        let moves = seq("R U2 F' D L B R2 U' F2 D' L2 B' R U");
        let tracked = track_moves(CenterTwist::UPRIGHT, &moves).unwrap();
        let quarters: usize = moves
            .moves()
            .iter()
            .map(|m| if m.turn == Turn::Half { 2 } else { 1 })
            .sum();
        assert_eq!(tracked.twist.sum_is_even(), quarters.is_multiple_of(2));
    }

    #[test]
    fn the_text_form_round_trips_and_refuses_junk() {
        let t: CenterTwist = "0,1,2,0,3,0".parse().unwrap();
        assert_eq!(t.as_array(), [0, 1, 2, 0, 3, 0]);
        assert_eq!(t.to_string(), "0,1,2,0,3,0");
        assert_eq!(" 0, 1,2,0,3 ,0".parse::<CenterTwist>().unwrap(), t);
        for bad in [
            "",
            "0,1,2,0,3",
            "0,1,2,0,3,4",
            "a,b,c,d,e,f",
            "0,1,2,0,3,0,0",
        ] {
            assert!(bad.parse::<CenterTwist>().is_err(), "{bad}");
        }
    }

    #[test]
    fn a_twist_condition_has_alternatives_and_open_centers() {
        let masks = parse_twist_masks("0,X,2|3,?,1,X").unwrap();
        assert_eq!(masks, [1, 15, 12, 15, 2, 15]);
        assert_eq!(twist_masks_text(&masks), "0,X,2|3,X,1,X");
        assert!(parse_twist_masks("0,1").is_err());
        assert!(parse_twist_masks("0,1,2,3,4,0").is_err());
    }
}
