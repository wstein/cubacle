//! The `metrics:` lines the commands print on standard error after `moves N`.
//! Standard output is never touched.

use cubacle::metrics::{self, Metric};
use cubacle::puzzle::{LayerMove, MoveSequence, PuzzleKind};

/// `metrics: htm 20, qtm 26, stm 20` for a sequence of plain moves.
pub fn line(kind: PuzzleKind, moves: &MoveSequence) -> String {
    format!("metrics: {}", metrics::summary(moves, kind))
}

/// The same for a compressed sequence of layer ranges.
pub fn compressed_line(kind: PuzzleKind, moves: &[LayerMove]) -> String {
    format!("metrics: {}", metrics::summary_layer_moves(moves, kind))
}

/// All seven counts of a compressed sequence.
pub fn compressed_all_line(kind: PuzzleKind, moves: &[LayerMove]) -> String {
    let counts = Metric::ALL
        .into_iter()
        .map(|metric| (metric, metrics::count_layer_moves(moves, kind, metric)));
    format!("metrics: {}", metrics::join(counts))
}

/// All seven counts of `moves`.
pub fn all_line(kind: PuzzleKind, moves: &MoveSequence) -> String {
    format!(
        "metrics: {}",
        metrics::join(metrics::count_all(moves, kind))
    )
}

/// The counts of move text as written, so `M`, `x` and `2-3Rw` count as the
/// one turn they are; text with groups or repeats is counted after expansion
/// (`moves`). `all` selects all seven metrics instead of the headline ones.
pub fn text_line(text: &str, kind: PuzzleKind, moves: &MoveSequence, all: bool) -> String {
    let wanted: &[Metric] = if all {
        &Metric::ALL
    } else {
        Metric::headline(kind)
    };
    let counts = wanted.iter().map(|&metric| {
        let n = metrics::count_text(text, kind, metric)
            .unwrap_or_else(|| metrics::count(moves, kind, metric));
        (metric, n)
    });
    format!("metrics: {}", metrics::join(counts))
}

/// The headline counts of flat move text (a pattern generator), `None` when
/// the text is not flat.
pub fn words_line(text: &str, kind: PuzzleKind) -> Option<String> {
    let counts: Option<Vec<(Metric, usize)>> = Metric::headline(kind)
        .iter()
        .map(|&metric| metrics::count_text(text, kind, metric).map(|n| (metric, n)))
        .collect();
    counts.map(|counts| format!("metrics: {}", metrics::join(counts)))
}

/// The smallest and largest headline counts over the sequences added, for
/// `solve --all`: `metrics: htm 20, qtm 24-26, stm 20`.
pub struct Spread {
    kind: PuzzleKind,
    ranges: Vec<(Metric, usize, usize)>,
}

impl Spread {
    pub fn new(kind: PuzzleKind) -> Spread {
        Spread {
            kind,
            ranges: Vec::new(),
        }
    }

    pub fn add(&mut self, moves: &MoveSequence) {
        let counts: Vec<(Metric, usize)> = Metric::headline(self.kind)
            .iter()
            .map(|&metric| (metric, metrics::count(moves, self.kind, metric)))
            .collect();
        if self.ranges.is_empty() {
            self.ranges = counts.iter().map(|&(m, n)| (m, n, n)).collect();
            return;
        }
        for (range, (_, n)) in self.ranges.iter_mut().zip(counts) {
            range.1 = range.1.min(n);
            range.2 = range.2.max(n);
        }
    }

    /// The line, `None` when nothing was added.
    pub fn line(&self) -> Option<String> {
        if self.ranges.is_empty() {
            return None;
        }
        let parts: Vec<String> = self
            .ranges
            .iter()
            .map(|&(metric, low, high)| {
                if low == high {
                    format!("{metric} {low}")
                } else {
                    format!("{metric} {low}-{high}")
                }
            })
            .collect();
        Some(format!(
            "metrics over the listed solutions: {}",
            parts.join(", ")
        ))
    }
}
