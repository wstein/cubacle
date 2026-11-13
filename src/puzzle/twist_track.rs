//! The geometry behind the center twist: where each picture sits and which
//! way it points, followed through layer turns and whole-cube rotations.
//!
//! See [`super::twist`] for the convention.

use super::twists::{CenterTwists, TwistError};
use super::{Face, Move, MoveSequence, PuzzleKind, Turn};

type Vec3 = [i32; 3];

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
pub(super) struct Tracker {
    normal: [Vec3; 6],
    up: [Vec3; 6],
}

impl Tracker {
    pub(super) fn from_twists(t: &CenterTwists) -> Result<Tracker, TwistError> {
        if t.kind() != PuzzleKind::Cube3 {
            return Err(TwistError::Unsupported(format!(
                "center twists are tracked on the 3x3x3 only, not the {0}x{0}x{0}",
                t.kind().size()
            )));
        }
        Ok(Tracker::new(std::array::from_fn(|g| t.twist(Face::ALL[g]))))
    }

    fn new(t: [u8; 6]) -> Tracker {
        Tracker {
            normal: NORMAL,
            up: std::array::from_fn(|g| clockwise(Face::ALL[g], t[g], UP0[g])),
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

    pub(super) fn rotate_word(&mut self, word: &str) -> Result<(), TwistError> {
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
    pub(super) fn read(&self) -> CenterTwists {
        let places = self.places();
        let mut t = [0u8; 6];
        for (&p, &up) in places.iter().zip(&self.up) {
            t[p.index()] = (0..4u8)
                .find(|&k| clockwise(p, k, UP0[p.index()]) == up)
                .expect("a picture stays square to its face");
        }
        CenterTwists::new(PuzzleKind::Cube3, t).expect("quarters are below four")
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
    pub twists: CenterTwists,
    /// The whole-cube rotation (`x y z` text, empty for none) that carries the
    /// cube from where the moves left it to that frame. It costs no move.
    pub frame: String,
}

/// Replays `moves` on a 3x3x3 whose centers start twisted as `start`. Outer
/// layer turns, wide turns, middle slices and rotations are all followed. When
/// the centers end on other faces than their own (a rotation, or slices), the
/// cube is turned as a whole into the frame where they are home, and the
/// result says which rotation that is.
pub(super) fn track(start: &CenterTwists, moves: &MoveSequence) -> Result<Tracked, TwistError> {
    let mut tracker = Tracker::from_twists(start)?;
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
            twists: tracker.read(),
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
        twists: tracker.read(),
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
    let identity = Tracker::new([0; 6]);
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
