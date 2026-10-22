//! Turning layers of a facelet state.
//!
//! Stickers are placed on a grid with the cube centred on the origin, `x`
//! pointing right, `y` up and `z` towards the viewer of the F face. A layer
//! turn is then a quarter rotation of every sticker in that layer, which works
//! for every cube size without per-size tables.

use std::fmt;

use super::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

type Vec3 = [i32; 3];

/// Why a move sequence could not be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyError {
    /// The first move that does not exist on the cube.
    pub mv: Move,
    pub kind: PuzzleKind,
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.kind.size();
        write!(
            f,
            "move {} does not exist on a {size}x{size}x{size}",
            self.mv
        )
    }
}

impl std::error::Error for ApplyError {}

impl Facelets {
    /// The state reached by performing `moves` on this one.
    ///
    /// Fails if a move turns a layer the cube does not have.
    pub fn apply(&self, moves: &MoveSequence) -> Result<Facelets, ApplyError> {
        let kind = self.kind();
        if let Some(&mv) = moves.moves().iter().find(|mv| !mv.fits(kind)) {
            return Err(ApplyError { mv, kind });
        }
        let mut stickers = self.stickers().to_vec();
        for mv in moves.moves() {
            stickers = turned(kind, &stickers, mv);
        }
        Ok(Facelets::from_stickers(stickers).expect("turning keeps every colour count"))
    }
}

/// The position and outward normal of a sticker, by index.
pub(crate) fn locate(kind: PuzzleKind, index: usize) -> (Vec3, Vec3) {
    let n = kind.size() as i32;
    let m = n - 1;
    let face = index / kind.face_len();
    let row = (index % kind.face_len()) / kind.size();
    let col = index % kind.size();
    let a = 2 * col as i32 - m;
    let b = 2 * row as i32 - m;
    match Face::ALL[face] {
        Face::U => ([a, m, b], [0, 1, 0]),
        Face::R => ([m, -b, -a], [1, 0, 0]),
        Face::F => ([a, -b, m], [0, 0, 1]),
        Face::D => ([a, -m, -b], [0, -1, 0]),
        Face::L => ([-m, -b, a], [-1, 0, 0]),
        Face::B => ([-a, -b, -m], [0, 0, -1]),
    }
}

/// The inverse of [`locate`].
fn index_of(kind: PuzzleKind, position: Vec3, normal: Vec3) -> usize {
    let m = kind.size() as i32 - 1;
    let [x, y, z] = position;
    let (face, a, b) = match normal {
        [0, 1, 0] => (Face::U, x, z),
        [1, 0, 0] => (Face::R, -z, -y),
        [0, 0, 1] => (Face::F, x, -y),
        [0, -1, 0] => (Face::D, x, -z),
        [-1, 0, 0] => (Face::L, z, -y),
        _ => (Face::B, -x, -y),
    };
    let col = ((a + m) / 2) as usize;
    let row = ((b + m) / 2) as usize;
    face.index() * kind.face_len() + row * kind.size() + col
}

/// A quarter turn about the positive `axis`, counter-clockwise when seen from
/// the positive side (the right-hand rule).
fn quarter(axis: usize, v: Vec3) -> Vec3 {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// Where every sticker goes when `mv` is performed: sticker `i` ends up at
/// index `result[i]`, and stickers outside the turned layers stay put.
pub(crate) fn destinations(kind: PuzzleKind, mv: &Move) -> Vec<usize> {
    let (axis, sign) = match mv.face {
        Face::U => (1, 1),
        Face::D => (1, -1),
        Face::R => (0, 1),
        Face::L => (0, -1),
        Face::F => (2, 1),
        Face::B => (2, -1),
    };
    // Clockwise seen from outside is a negative right-hand turn about the
    // face's own outward axis: one quarter backwards for U, R and F, one
    // forwards for D, L and B.
    let quarters = match (mv.turn, sign) {
        (Turn::Half, _) => 2,
        (Turn::Clockwise, 1) | (Turn::CounterClockwise, -1) => 3,
        _ => 1,
    };
    let edge = kind.size() as i32 - 1 - 2 * (i32::from(mv.depth) - 1);
    let mut result: Vec<usize> = (0..kind.facelet_count()).collect();
    for (index, slot) in result.iter_mut().enumerate() {
        let (position, normal) = locate(kind, index);
        let height = sign * position[axis];
        let inside = if mv.wide {
            height >= edge
        } else {
            height == edge
        };
        if !inside {
            continue;
        }
        let (mut position, mut normal) = (position, normal);
        for _ in 0..quarters {
            position = quarter(axis, position);
            normal = quarter(axis, normal);
        }
        *slot = index_of(kind, position, normal);
    }
    result
}

fn turned(kind: PuzzleKind, stickers: &[Face], mv: &Move) -> Vec<Face> {
    let mut result = stickers.to_vec();
    for (index, &target) in destinations(kind, mv).iter().enumerate() {
        result[target] = stickers[index];
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(text: &str) -> MoveSequence {
        text.parse().unwrap()
    }

    #[test]
    fn known_3x3_states() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        assert_eq!(
            solved.apply(&seq("R")).unwrap().to_string(),
            "UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB"
        );
        assert_eq!(
            solved.apply(&seq("U")).unwrap().to_string(),
            "UUUUUUUUUBBBRRRRRRRRRFFFFFFDDDDDDDDDFFFLLLLLLLLLBBBBBB"
        );
    }

    #[test]
    fn a_turn_and_its_inverse_cancel_on_every_size() {
        for kind in PuzzleKind::ALL {
            let solved = Facelets::solved(kind);
            for face in Face::ALL {
                let letter = face.letter();
                let there = solved.apply(&seq(&letter.to_string())).unwrap();
                assert!(!there.is_solved(), "{kind:?} {letter}");
                let back = there.apply(&seq(&format!("{letter}'"))).unwrap();
                assert!(back.is_solved(), "{kind:?} {letter}");
                let half = solved.apply(&seq(&format!("{letter}2"))).unwrap();
                assert_eq!(half, there.apply(&seq(&letter.to_string())).unwrap());
            }
        }
    }

    #[test]
    fn four_quarter_turns_and_commutators_return_home() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        assert!(solved.apply(&seq("R R R R")).unwrap().is_solved());
        // (R U R' U') has order 6.
        let six = seq("R U R' U' R U R' U' R U R' U' R U R' U' R U R' U' R U R' U'");
        assert!(solved.apply(&six).unwrap().is_solved());
    }

    #[test]
    fn inner_layers_and_wide_moves_follow_the_size() {
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let wide = solved.apply(&seq("Rw")).unwrap();
        let both = solved.apply(&seq("R 2R")).unwrap();
        assert_eq!(wide, both);
        assert!(wide.apply(&seq("Rw'")).unwrap().is_solved());
    }

    #[test]
    fn moves_the_cube_lacks_are_rejected() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        let error = solved.apply(&seq("3R")).unwrap_err();
        assert_eq!(error.mv.to_string(), "3R");
        assert!(error.to_string().contains("3x3x3"));
        assert!(Facelets::solved(PuzzleKind::Cube2)
            .apply(&seq("2R"))
            .is_err());
    }
}
