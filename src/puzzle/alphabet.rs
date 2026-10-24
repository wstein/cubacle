//! The moves a solution may use.
//!
//! An [`Alphabet`] is a set of plain moves for one cube size. Two spellings of
//! the same turn are one letter of it: `R'` on the left layer of a 5x5x5 and
//! `L` on the right one turn the same layer the same way, and `3R` equals
//! `3L'`, so the alphabet compares moves by the layers they turn and the net
//! quarter turns, not by their names. The unrestricted alphabet stands for
//! "whatever the engine uses today"; it is what `htm` names.

use super::{Face, Move, PuzzleKind, Turn};

/// What a move does, independent of its spelling: the axis (U/D, R/L, F/B),
/// the first and last layer turned counted from the positive face (U, R, F),
/// and the quarter turns about the positive direction of the axis.
type Key = (u8, u8, u8, u8);

fn key_of(kind: PuzzleKind, mv: &Move) -> Key {
    let n = kind.size() as u8;
    let negative = mv.face.index() >= 3;
    let axis = (mv.face.index() % 3) as u8;
    let quarters = match mv.turn {
        Turn::Clockwise => 1,
        Turn::Half => 2,
        Turn::CounterClockwise => 3,
    };
    let quarters = if negative {
        (4 - quarters) % 4
    } else {
        quarters
    };
    let (first, last) = match (mv.wide, negative) {
        (false, false) => (mv.depth - 1, mv.depth - 1),
        (false, true) => (n - mv.depth, n - mv.depth),
        (true, false) => (0, mv.depth - 1),
        (true, true) => (n - mv.depth, n - 1),
    };
    (axis, first, last, quarters)
}

/// A set of moves for one cube size, or no restriction at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alphabet {
    kind: PuzzleKind,
    label: String,
    unrestricted: bool,
    moves: Vec<Move>,
    keys: Vec<Key>,
}

impl Alphabet {
    /// The alphabet that restricts nothing: the engine moves as it does by
    /// default. Its `moves` are empty.
    pub fn unrestricted(kind: PuzzleKind) -> Alphabet {
        Alphabet {
            kind,
            label: "htm".to_owned(),
            unrestricted: true,
            moves: Vec::new(),
            keys: Vec::new(),
        }
    }

    /// The 18 face turns of a cube, as a list of moves (not the unrestricted
    /// marker): the alphabet a listing of solutions uses by default.
    pub fn face_turns(kind: PuzzleKind) -> Alphabet {
        let mut moves = Vec::new();
        for face in Face::ALL {
            for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
                moves.push(Move::new(face, 1, false, turn));
            }
        }
        Alphabet::new(kind, "htm", moves)
    }

    /// The alphabet of `moves` (each kept once, in the order given). Moves
    /// that do not exist on the cube are left out.
    pub fn new(kind: PuzzleKind, label: impl Into<String>, moves: Vec<Move>) -> Alphabet {
        let mut alphabet = Alphabet {
            kind,
            label: label.into(),
            unrestricted: false,
            moves: Vec::new(),
            keys: Vec::new(),
        };
        for mv in moves {
            if !mv.fits(kind) {
                continue;
            }
            let key = key_of(kind, &mv);
            if !alphabet.keys.contains(&key) {
                alphabet.keys.push(key);
                alphabet.moves.push(mv);
            }
        }
        alphabet
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// How the alphabet was named: `htm`, `half-turns`, `<R,U>`, or the move
    /// list as written.
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn is_unrestricted(&self) -> bool {
        self.unrestricted
    }

    /// The moves, in the order they were given. Empty for the unrestricted
    /// alphabet.
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        !self.unrestricted && self.moves.is_empty()
    }

    /// Whether the alphabet has this move (under any spelling of it).
    pub fn contains(&self, mv: &Move) -> bool {
        self.unrestricted || (mv.fits(self.kind) && self.keys.contains(&key_of(self.kind, mv)))
    }

    /// Whether every move has its inverse in the alphabet.
    pub fn is_closed_under_inverse(&self) -> bool {
        self.unrestricted
            || self
                .moves
                .iter()
                .all(|mv| self.contains(&Move::new(mv.face, mv.depth, mv.wide, invert(mv.turn))))
    }
}

fn invert(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    }
}

/// The three faces whose layers are counted from the positive side of an axis.
const POSITIVE: [Face; 3] = [Face::U, Face::R, Face::F];
const NEGATIVE: [Face; 3] = [Face::D, Face::L, Face::B];

/// Every single layer of the cube once, as moves of the given turns: the
/// layers `1..n-1` from the positive face of each axis and the outer layer of
/// the negative face. With `outer_only` just the six outer layers.
pub fn layer_set(kind: PuzzleKind, turns: &[Turn], outer_only: bool) -> Vec<Move> {
    let n = kind.size() as u8;
    let mut out = Vec::new();
    for axis in 0..3 {
        for &turn in turns {
            let last = if outer_only { 1 } else { n - 1 };
            for depth in 1..=last {
                out.push(Move::new(POSITIVE[axis], depth, false, turn));
            }
            out.push(Move::new(NEGATIVE[axis], 1, false, turn));
        }
    }
    out
}

/// The inner layers (not the outer ones) of each axis, as moves of the given
/// turns, counted from the positive face.
pub fn inner_layer_set(kind: PuzzleKind, turns: &[Turn]) -> Vec<Move> {
    let n = kind.size() as u8;
    let mut out = Vec::new();
    for face in POSITIVE {
        for depth in 2..n {
            for &turn in turns {
                out.push(Move::new(face, depth, false, turn));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(text: &str) -> Move {
        text.parse().unwrap()
    }

    #[test]
    fn spellings_of_one_turn_are_one_letter() {
        let kind = PuzzleKind::Cube5;
        let alphabet = Alphabet::new(kind, "x", vec![mv("3R"), mv("3L'"), mv("R"), mv("5R")]);
        // 3R and 3L' are the middle layer; R is the right layer; 5R does not exist.
        assert_eq!(alphabet.len(), 2);
        assert!(alphabet.contains(&mv("3L'")));
        assert!(!alphabet.contains(&mv("L'")));
        assert!(alphabet.contains(&mv("3R")));
    }

    #[test]
    fn the_opposite_layer_is_another_letter_on_the_outside() {
        let kind = PuzzleKind::Cube3;
        let alphabet = Alphabet::new(kind, "x", vec![mv("R"), mv("L")]);
        assert_eq!(alphabet.len(), 2);
        assert!(alphabet.contains(&mv("R")));
        assert!(!alphabet.contains(&mv("R'")));
    }

    #[test]
    fn a_wide_turn_is_not_its_single_layers() {
        let kind = PuzzleKind::Cube4;
        let alphabet = Alphabet::new(kind, "x", vec![mv("Rw")]);
        assert!(alphabet.contains(&mv("Rw")));
        assert!(!alphabet.contains(&mv("R")));
        assert!(!alphabet.contains(&mv("2R")));
        // Lw turns the two layers at the other side.
        assert!(!alphabet.contains(&mv("Lw")));
    }

    #[test]
    fn closure_under_inverse() {
        let kind = PuzzleKind::Cube3;
        assert!(Alphabet::new(kind, "x", vec![mv("R2"), mv("U2")]).is_closed_under_inverse());
        assert!(!Alphabet::new(kind, "x", vec![mv("R")]).is_closed_under_inverse());
        assert!(Alphabet::new(kind, "x", vec![mv("R"), mv("R'")]).is_closed_under_inverse());
        assert!(Alphabet::unrestricted(kind).is_closed_under_inverse());
    }

    #[test]
    fn layer_sets_have_the_expected_sizes() {
        let all = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];
        assert_eq!(layer_set(PuzzleKind::Cube3, &all, true).len(), 18);
        assert_eq!(layer_set(PuzzleKind::Cube3, &all, false).len(), 27);
        assert_eq!(layer_set(PuzzleKind::Cube5, &[Turn::Half], false).len(), 15);
        assert_eq!(inner_layer_set(PuzzleKind::Cube3, &all).len(), 9);
    }
}
