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

use super::{Face, Move, MoveSequence, PuzzleKind, Turn};

/// The version of the twist convention, printed in certificates.
pub const TWIST_CONVENTION: &str = "center-twist-1 (t=URFDLB, clockwise quarters seen from outside, up: U back, D front, sides up)";

type Vec3 = [i32; 3];

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

const NORMAL: [Vec3; 6] = [
    [0, 1, 0],
    [1, 0, 0],
    [0, 0, 1],
    [0, -1, 0],
    [-1, 0, 0],
    [0, 0, -1],
];

/// The up direction of the picture of each center in its solved orientation.
const UP0: [Vec3; 6] = [
    [0, 0, -1],
    [0, 1, 0],
    [0, 1, 0],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 0],
];

/// The axis and sign of a face: its outward normal is `sign` on `axis`.
fn axis_of(face: Face) -> (usize, i32) {
    match face {
        Face::U => (1, 1),
        Face::D => (1, -1),
        Face::R => (0, 1),
        Face::L => (0, -1),
        Face::F => (2, 1),
        Face::B => (2, -1),
    }
}

/// One right-hand quarter turn about the positive `axis`.
fn quarter(axis: usize, v: Vec3) -> Vec3 {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// `quarters` clockwise quarter turns, seen from outside `face`, of `v`.
fn clockwise(face: Face, quarters: u8, v: Vec3) -> Vec3 {
    let (axis, sign) = axis_of(face);
    // clockwise seen from outside is backwards about the outward axis of
    // U, R and F and forwards about the one of D, L and B
    let right_hand = if sign == 1 { 3 } else { 1 };
    (0..quarters * right_hand).fold(v, |v, _| quarter(axis, v))
}

/// Where each center is and which way its picture points.
#[derive(Debug, Clone, Copy)]
struct Tracker {
    normal: [Vec3; 6],
    up: [Vec3; 6],
}

impl Tracker {
    fn from_twist(t: CenterTwist) -> Tracker {
        Tracker {
            normal: NORMAL,
            up: std::array::from_fn(|g| clockwise(Face::ALL[g], t.0[g], UP0[g])),
        }
    }

    /// The face a normal points out of.
    fn face_of(normal: Vec3) -> Face {
        Face::ALL[NORMAL
            .iter()
            .position(|&n| n == normal)
            .expect("a normal stays a unit axis")]
    }

    /// Turns the layers of `mv` (3x3x3 geometry: layer height 2, 0 or -2).
    fn apply(&mut self, mv: &Move) {
        let (axis, sign) = axis_of(mv.face);
        let edge = 2 - 2 * (i32::from(mv.depth) - 1);
        let quarters = match mv.turn {
            Turn::Clockwise => 1,
            Turn::Half => 2,
            Turn::CounterClockwise => 3,
        };
        for g in 0..6 {
            let height = sign * 2 * self.normal[g][axis];
            let inside = if mv.wide {
                height >= edge
            } else {
                height == edge
            };
            if inside {
                self.normal[g] = clockwise(mv.face, quarters, self.normal[g]);
                self.up[g] = clockwise(mv.face, quarters, self.up[g]);
            }
        }
    }

    /// Turns the whole cube about the axis of `face` (`x` is R, `y` is U, `z` is F).
    fn rotate(&mut self, face: Face, quarters: u8) {
        for g in 0..6 {
            self.normal[g] = clockwise(face, quarters, self.normal[g]);
            self.up[g] = clockwise(face, quarters, self.up[g]);
        }
    }

    fn rotate_word(&mut self, word: &str) -> Result<(), TwistError> {
        for (face, quarters) in rotation_tokens(word)? {
            self.rotate(face, quarters);
        }
        Ok(())
    }

    /// Where each center sits.
    fn places(&self) -> [Face; 6] {
        std::array::from_fn(|g| Tracker::face_of(self.normal[g]))
    }

    /// The twist seen on each face: the picture that sits on face `p`, turned
    /// from the solved orientation of face `p`.
    fn read(&self) -> CenterTwist {
        let places = self.places();
        let mut t = [0u8; 6];
        for g in 0..6 {
            let p = places[g];
            t[p.index()] = (0..4u8)
                .find(|&k| clockwise(p, k, UP0[p.index()]) == self.up[g])
                .expect("a picture stays square to its face");
        }
        CenterTwist(t)
    }
}

/// The turns of a rotation word as (axis face, clockwise quarters).
fn rotation_tokens(word: &str) -> Result<Vec<(Face, u8)>, TwistError> {
    word.split_whitespace()
        .map(|token| {
            let face = match token.chars().next() {
                Some('x') => Face::R,
                Some('y') => Face::U,
                Some('z') => Face::F,
                _ => {
                    return Err(TwistError::Syntax(format!(
                        "{token:?} is no rotation (x, y or z)"
                    )))
                }
            };
            let turn = Turn::from_suffix(&token[1..]).ok_or_else(|| {
                TwistError::Syntax(format!("{token:?} is no rotation (x, y or z)"))
            })?;
            Ok((
                face,
                match turn {
                    Turn::Clockwise => 1,
                    Turn::Half => 2,
                    Turn::CounterClockwise => 3,
                },
            ))
        })
        .collect()
}

/// The layer turns of a rotation word on a 3x3x3 (as the expression reader
/// writes `x y z`: all but the last layer turn one way, the last the other).
pub fn rotation_moves(word: &str) -> Result<MoveSequence, TwistError> {
    let mut moves = Vec::new();
    for (face, quarters) in rotation_tokens(word)? {
        let (front, back) = match face {
            Face::R => (Face::R, Face::L),
            Face::U => (Face::U, Face::D),
            _ => (Face::F, Face::B),
        };
        let turn = match quarters {
            1 => Turn::Clockwise,
            2 => Turn::Half,
            _ => Turn::CounterClockwise,
        };
        let opposite = match turn {
            Turn::Clockwise => Turn::CounterClockwise,
            Turn::Half => Turn::Half,
            Turn::CounterClockwise => Turn::Clockwise,
        };
        moves.push(Move::new(front, 2, true, turn));
        moves.push(Move::new(back, 1, false, opposite));
    }
    Ok(MoveSequence::new(moves))
}

/// What a move sequence did to the centers of a 3x3x3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracked {
    /// The twist read in the frame where every center is on its own face.
    pub twist: CenterTwist,
    /// The whole-cube rotation (`x y z` text, empty for none) that carries the
    /// cube from where the moves left it to that frame. It costs no move.
    pub frame: String,
}

/// Replays `moves` on a 3x3x3 whose centers start twisted as `start`. Outer
/// layer turns, wide turns, middle slices and rotations are all followed. When
/// the centers end on other faces than their own (a rotation, or slices), the
/// cube is turned as a whole into the frame where they are home, and the
/// result says which rotation that is.
pub fn track_moves(start: CenterTwist, moves: &MoveSequence) -> Result<Tracked, TwistError> {
    let mut tracker = Tracker::from_twist(start);
    for mv in moves.moves() {
        if !mv.fits(PuzzleKind::Cube3) {
            return Err(TwistError::Unsupported(format!(
                "move {mv} does not exist on a 3x3x3"
            )));
        }
        tracker.apply(mv);
    }
    let places = tracker.places();
    if places.iter().enumerate().all(|(g, p)| p.index() == g) {
        return Ok(Tracked {
            twist: tracker.read(),
            frame: String::new(),
        });
    }
    let rotation = center_rotations()
        .into_iter()
        .find(|r| (0..6).all(|g| r.dest[places[g].index()] == g))
        .ok_or_else(|| {
            TwistError::Unsupported("the centers are not a rotation of the cube".into())
        })?;
    tracker.rotate_word(&rotation.word)?;
    Ok(Tracked {
        twist: tracker.read(),
        frame: rotation.word,
    })
}

/// One of the 24 whole-cube rotations, in the order of `rotation_maps`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CenterRotation {
    /// The word of `x y z` turns that makes it, empty for the identity.
    pub(crate) word: String,
    /// Center `g` ends on face `dest[g]`.
    pub(crate) dest: [usize; 6],
    /// The picture of center `g` is turned by `add[g]` more quarters there.
    pub(crate) add: [u8; 6],
}

/// The 24 rotations with their effect on the centers, in the same order as
/// [`super::rotation_maps`] (breadth first over `x`, `y`, `z`).
pub(crate) fn center_rotations() -> Vec<CenterRotation> {
    let generators = [("x", Face::R), ("y", Face::U), ("z", Face::F)];
    let effect = |tracker: &Tracker| -> ([usize; 6], [u8; 6]) {
        let places = tracker.places();
        let dest = places.map(Face::index);
        let add = std::array::from_fn(|g| {
            let p = places[g];
            (0..4u8)
                .find(|&k| clockwise(p, k, UP0[p.index()]) == tracker.up[g])
                .expect("a picture stays square to its face")
        });
        (dest, add)
    };
    let identity = Tracker::from_twist(CenterTwist::UPRIGHT);
    let (dest, add) = effect(&identity);
    let mut seen = vec![
        (CenterRotation {
            word: String::new(),
            dest,
            add,
        }),
    ];
    let mut states = vec![identity];
    let mut at = 0;
    while at < seen.len() {
        for (letter, face) in generators {
            let mut next = states[at];
            next.rotate(face, 1);
            let (dest, add) = effect(&next);
            if !seen.iter().any(|r| r.dest == dest) {
                let word = if seen[at].word.is_empty() {
                    letter.to_owned()
                } else {
                    format!("{} {letter}", seen[at].word)
                };
                seen.push(CenterRotation { word, dest, add });
                states.push(next);
            }
        }
        at += 1;
    }
    seen
}

/// The words of the 24 whole-cube rotations, the identity (empty) first.
pub fn rotation_words() -> Vec<String> {
    center_rotations().into_iter().map(|r| r.word).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{rotation_maps, Facelets};

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
        assert_eq!(tracked.twist.sum_is_even(), quarters % 2 == 0);
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

    #[test]
    fn a_rotation_relabels_the_centers_and_costs_no_move() {
        // y carries the side centers one face round: not home, so the cube is
        // turned back into the frame where they are, and the pictures are upright there
        let tracked = track_moves(CenterTwist::UPRIGHT, &seq("y")).unwrap();
        assert!(!tracked.frame.is_empty());
        assert_eq!(tracked.twist, CenterTwist::UPRIGHT);
        let there_and_back = track_moves(CenterTwist::UPRIGHT, &seq("x x'")).unwrap();
        assert_eq!(there_and_back.twist, CenterTwist::UPRIGHT);
        assert!(there_and_back.frame.is_empty());
        // four quarters of any axis are the identity
        let four = track_moves(CenterTwist::UPRIGHT, &seq("z z z z")).unwrap();
        assert_eq!(four.twist, CenterTwist::UPRIGHT);
    }

    #[test]
    fn a_rotation_word_applied_in_text_and_in_moves_agrees() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        let maps = rotation_maps(PuzzleKind::Cube3);
        let rotations = center_rotations();
        assert_eq!(rotations.len(), 24);
        for (map, rotation) in maps.iter().zip(&rotations) {
            // the sticker map agrees with the geometry on the centers
            for g in 0..6 {
                assert_eq!(map[g * 9 + 4] / 9, rotation.dest[g], "{}", rotation.word);
            }
            // and the layer turns of the word make the same sticker map
            let turned = solved
                .apply(&rotation_moves(&rotation.word).unwrap())
                .unwrap();
            let mut by_map = solved.stickers().to_vec();
            for (i, &to) in map.iter().enumerate() {
                by_map[to] = solved.stickers()[i];
            }
            assert_eq!(turned.stickers(), by_map.as_slice(), "{}", rotation.word);
        }
    }

    #[test]
    fn the_effect_of_a_rotation_is_the_same_in_the_tracker_and_the_table() {
        let start: CenterTwist = "1,2,0,3,1,1".parse().unwrap();
        for rotation in center_rotations() {
            let seen = start.rotated(&rotation.word).unwrap().as_array();
            for g in 0..6 {
                assert_eq!(
                    seen[rotation.dest[g]],
                    (start.as_array()[g] + rotation.add[g]) & 3,
                    "{}",
                    rotation.word
                );
            }
        }
    }

    #[test]
    fn a_middle_slice_is_followed_and_ends_in_a_home_frame() {
        let slice = track_moves(CenterTwist::UPRIGHT, &seq("2R")).unwrap();
        assert!(rotation_words().contains(&slice.frame));
        assert!(!slice.frame.is_empty());
        let cancels = track_moves(CenterTwist::UPRIGHT, &seq("L' 2R 2R'")).unwrap();
        assert_eq!(cancels.twist.as_array(), [0, 0, 0, 0, 3, 0]);
        assert!(cancels.frame.is_empty());
    }
}
