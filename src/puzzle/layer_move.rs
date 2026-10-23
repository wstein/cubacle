//! A turn of a contiguous block of layers: the `2-3Rw` of a compressed solution.
//!
//! [`Move`] stays as it is (a single layer, or the outer `depth` layers). A
//! [`LayerMove`] names any block `from..=to`, counted from `face`, so that
//! adjacent turns of one axis can be written as one. It is an output form:
//! the engines, [`crate::puzzle::Facelets::apply`] and the parsers only know
//! [`Move`], and [`LayerMove::to_moves`] gives the plain moves back.

use std::fmt;

use super::{Face, Move, PuzzleKind, Turn};

/// Layers `from..=to` (1 is the outer layer of `face`) turned together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerMove {
    pub face: Face,
    pub from: u8,
    pub to: u8,
    pub turn: Turn,
}

impl LayerMove {
    pub fn new(face: Face, from: u8, to: u8, turn: Turn) -> LayerMove {
        LayerMove {
            face,
            from,
            to,
            turn,
        }
    }

    /// Whether the block exists on a cube of the given size: `1 <= from <= to < n`,
    /// so at least one layer stays behind (a turn of every layer is a rotation).
    pub fn fits(&self, kind: PuzzleKind) -> bool {
        self.from >= 1 && self.from <= self.to && usize::from(self.to) < kind.size()
    }

    /// The same turn as a plain [`Move`], when one exists: a single layer, or
    /// a block that starts at the outer layer.
    pub fn to_move(&self) -> Option<Move> {
        match (self.from, self.to) {
            (1, 1) => Some(Move::new(self.face, 1, false, self.turn)),
            (1, to) => Some(Move::new(self.face, to, true, self.turn)),
            (from, to) if from == to => Some(Move::new(self.face, from, false, self.turn)),
            _ => None,
        }
    }

    /// The same turn as plain moves: one when [`LayerMove::to_move`] exists,
    /// else the outer block `to` turned and the outer block `from - 1` turned back.
    pub fn to_moves(&self) -> Vec<Move> {
        if let Some(mv) = self.to_move() {
            return vec![mv];
        }
        let inner = Move::new(self.face, self.to, true, self.turn);
        let behind = self.from - 1;
        let outer = Move::new(self.face, behind, behind > 1, invert(self.turn));
        vec![inner, outer]
    }

    /// The suffix of the turn: empty, `2` or `'`.
    pub fn turn_suffix(&self) -> &'static str {
        match self.turn {
            Turn::Clockwise => "",
            Turn::Half => "2",
            Turn::CounterClockwise => "'",
        }
    }
}

fn invert(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    }
}

impl From<Move> for LayerMove {
    fn from(mv: Move) -> LayerMove {
        let from = if mv.wide { 1 } else { mv.depth };
        LayerMove::new(mv.face, from, mv.depth, mv.turn)
    }
}

/// The WCA spelling: `R`, `2R`, `Rw`, `3Rw`, and `2-3Rw` for an inner block.
impl fmt::Display for LayerMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_move() {
            Some(mv) => write!(f, "{mv}"),
            None => write!(
                f,
                "{}-{}{}w{}",
                self.from,
                self.to,
                self.face.letter(),
                self.turn_suffix()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Facelets, MoveSequence};

    #[test]
    fn plain_moves_round_trip() {
        for text in ["R", "2R'", "Rw2", "3Rw", "4U"] {
            let mv: Move = text.parse().unwrap();
            let layer = LayerMove::from(mv);
            assert_eq!(layer.to_move(), Some(mv));
            assert_eq!(layer.to_string(), text);
        }
    }

    #[test]
    fn inner_block_is_written_as_a_range() {
        let layer = LayerMove::new(Face::R, 2, 3, Turn::CounterClockwise);
        assert_eq!(layer.to_move(), None);
        assert_eq!(layer.to_string(), "2-3Rw'");
    }

    #[test]
    fn plain_moves_of_a_range_turn_the_same_layers() {
        let layer = LayerMove::new(Face::U, 2, 4, Turn::Half);
        let moves = MoveSequence::new(layer.to_moves());
        assert_eq!(moves.len(), 2);
        let range = Facelets::solved(PuzzleKind::Cube6).apply(&moves).unwrap();
        let wanted: MoveSequence = "4Uw2 U2".parse().unwrap();
        let solved = Facelets::solved(PuzzleKind::Cube6);
        assert_eq!(range, solved.apply(&wanted).unwrap());
        assert_ne!(range, solved.apply(&"4Uw2".parse().unwrap()).unwrap());
    }

    #[test]
    fn fits_leaves_a_layer() {
        assert!(LayerMove::new(Face::R, 2, 3, Turn::Clockwise).fits(PuzzleKind::Cube4));
        assert!(!LayerMove::new(Face::R, 2, 4, Turn::Clockwise).fits(PuzzleKind::Cube4));
    }
}
