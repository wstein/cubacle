//! One turn as the metrics see it: a block of layers about an axis.

use crate::puzzle::{Face, LayerMove, Move, PuzzleKind, Turn};

/// A turn of the layers `lo..=hi` (1 is the layer of the positive face U, R
/// or F; `n` the layer of D, L or B) about `axis` (0 = U/D, 1 = R/L, 2 = F/B).
/// A block of all `n` layers is a whole-cube rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Unit {
    pub(crate) axis: u8,
    pub(crate) lo: u8,
    pub(crate) hi: u8,
    pub(crate) half: bool,
}

pub(crate) fn size(kind: PuzzleKind) -> u8 {
    u8::try_from(kind.size()).expect("sizes are small")
}

impl Unit {
    pub(crate) fn rotation(kind: PuzzleKind, axis: u8, half: bool) -> Unit {
        Unit {
            axis,
            lo: 1,
            hi: size(kind),
            half,
        }
    }

    /// The layers `from..=to` counted from `face`.
    pub(crate) fn block(kind: PuzzleKind, face: Face, from: u8, to: u8, turn: Turn) -> Unit {
        let n = size(kind);
        let (lo, hi) = if face.index() >= 3 {
            (n + 1 - to, n + 1 - from)
        } else {
            (from, to)
        };
        Unit {
            axis: u8::try_from(face.index() % 3).expect("three axes"),
            lo,
            hi,
            half: turn == Turn::Half,
        }
    }

    pub(crate) fn of_move(kind: PuzzleKind, mv: &Move) -> Unit {
        let from = if mv.wide { 1 } else { mv.depth };
        Unit::block(kind, mv.face, from, mv.depth, mv.turn)
    }

    pub(crate) fn of_layer_move(kind: PuzzleKind, mv: &LayerMove) -> Unit {
        Unit::block(kind, mv.face, mv.from, mv.to, mv.turn)
    }

    pub(crate) fn is_rotation(&self, kind: PuzzleKind) -> bool {
        self.lo == 1 && self.hi == size(kind)
    }

    pub(crate) fn layers(&self) -> usize {
        usize::from(self.hi - self.lo) + 1
    }

    /// Whether the block includes an outer layer of its axis.
    pub(crate) fn touches_outside(&self, kind: PuzzleKind) -> bool {
        self.lo == 1 || self.hi == size(kind)
    }
}
