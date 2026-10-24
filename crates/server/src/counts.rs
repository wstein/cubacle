//! The move counts of an answer: all seven metrics of `docs/metrics.md`.

use cubacle::metrics::{count, count_layer_moves, Metric};
use cubacle::puzzle::{LayerMove, MoveSequence, PuzzleKind};
use serde::Serialize;

/// The seven counts, always in this key order. Clients read what they need.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct Counts {
    htm: usize,
    qtm: usize,
    stm: usize,
    etm: usize,
    obtm: usize,
    btm: usize,
    atm: usize,
}

impl Counts {
    fn with(count: impl Fn(Metric) -> usize) -> Counts {
        Counts {
            htm: count(Metric::Htm),
            qtm: count(Metric::Qtm),
            stm: count(Metric::Stm),
            etm: count(Metric::Etm),
            obtm: count(Metric::Obtm),
            btm: count(Metric::Btm),
            atm: count(Metric::Atm),
        }
    }

    /// The counts of a solution.
    pub(crate) fn of(moves: &MoveSequence, kind: PuzzleKind) -> Counts {
        Counts::with(|metric| count(moves, kind, metric))
    }

    /// The counts of a solution compressed to layer ranges.
    pub(crate) fn of_ranges(moves: &[LayerMove], kind: PuzzleKind) -> Counts {
        Counts::with(|metric| count_layer_moves(moves, kind, metric))
    }
}
