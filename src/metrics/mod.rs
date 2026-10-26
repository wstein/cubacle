//! Move metrics: how many moves a sequence is, counted in seven ways.
//!
//! The solvers' own length (`moves N`) counts block turns of outer layers or
//! single layers. This module counts a sequence in the standard metrics
//! (`docs/metrics.md` gives the exact rules and the choices made where
//! sources differ). It only counts; it never changes a sequence or searches.

mod count;
mod metric;
#[cfg(test)]
mod tests;
mod text;
mod unit;

pub use metric::Metric;

use crate::puzzle::{LayerMove, MoveSequence, PuzzleKind};
use unit::Unit;

/// The length of `moves` on a cube of `kind` in `metric`. Each move is one
/// turn of a block of layers; a [`MoveSequence`] holds no rotation, so
/// [`Metric::Etm`] equals [`Metric::Btm`] here.
pub fn count(moves: &MoveSequence, kind: PuzzleKind, metric: Metric) -> usize {
    let units: Vec<Unit> = moves
        .moves()
        .iter()
        .map(|mv| Unit::of_move(kind, mv))
        .collect();
    count::count_units(kind, &units, metric)
}

/// The same for a compressed sequence of layer ranges (`--optimize ranges`).
pub fn count_layer_moves(moves: &[LayerMove], kind: PuzzleKind, metric: Metric) -> usize {
    let units: Vec<Unit> = moves
        .iter()
        .map(|mv| Unit::of_layer_move(kind, mv))
        .collect();
    count::count_units(kind, &units, metric)
}

/// The length of a sequence written as text, one turn per word: `M`, `x`,
/// `Rw` and `2-3Rw` are single turns, and rotations count where a metric says
/// they do. `None` when the text is not a flat list of WCA or SiGN words
/// (groups, repeats, commutators, SSE) or a word does not fit the cube; count
/// the expanded [`MoveSequence`] then.
pub fn count_text(text: &str, kind: PuzzleKind, metric: Metric) -> Option<usize> {
    let units = text::units_of_text(text, kind)?;
    Some(count::count_units(kind, &units, metric))
}

/// All seven counts of `text`, see [`count_text`].
pub fn count_all_text(text: &str, kind: PuzzleKind) -> Option<Vec<(Metric, usize)>> {
    let units = text::units_of_text(text, kind)?;
    Some(
        Metric::ALL
            .into_iter()
            .map(|metric| (metric, count::count_units(kind, &units, metric)))
            .collect(),
    )
}

/// All seven counts of `moves`.
pub fn count_all(moves: &MoveSequence, kind: PuzzleKind) -> Vec<(Metric, usize)> {
    Metric::ALL
        .into_iter()
        .map(|metric| (metric, count(moves, kind, metric)))
        .collect()
}

/// `htm 20, qtm 26, stm 20`: the headline counts for the cube, as the CLI
/// prints them after `moves N`.
pub fn summary(moves: &MoveSequence, kind: PuzzleKind) -> String {
    join(
        Metric::headline(kind)
            .iter()
            .map(|&metric| (metric, count(moves, kind, metric))),
    )
}

/// The same for a compressed sequence.
pub fn summary_layer_moves(moves: &[LayerMove], kind: PuzzleKind) -> String {
    join(
        Metric::headline(kind)
            .iter()
            .map(|&metric| (metric, count_layer_moves(moves, kind, metric))),
    )
}

/// `htm 20, qtm 26, ...` from metric and count pairs.
pub fn join(counts: impl IntoIterator<Item = (Metric, usize)>) -> String {
    let parts: Vec<String> = counts
        .into_iter()
        .map(|(metric, count)| format!("{metric} {count}"))
        .collect();
    parts.join(", ")
}
