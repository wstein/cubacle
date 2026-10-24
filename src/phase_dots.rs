//! `solve --group phases`: the solution written with ` · ` between solver phases.

use cubacle::notation::Notation;
use cubacle::phases::Phase;
use cubacle::puzzle::{MoveSequence, PuzzleKind};
use cubacle::ranges::compressed_text;

/// How `solve` marks the solver phases in its answer.
#[derive(Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum SolveGroup {
    /// A plain list of moves
    #[default]
    None,
    /// A " · " between the phases of the solver (none for a one-phase solution)
    Phases,
}

/// The solution as text and its move count. Every non-empty phase is written
/// on its own, so ranges never merge across a dot; with `dots` they are joined
/// by ` · `, otherwise by a space.
pub fn write(
    notation: Notation,
    kind: PuzzleKind,
    solution: &MoveSequence,
    phases: &[Phase],
    ranges: bool,
    dots: bool,
) -> (String, usize) {
    let parts: Vec<MoveSequence> = if phases.is_empty() || !dots {
        vec![solution.clone()]
    } else {
        phases
            .iter()
            .filter(|p| !p.moves.is_empty())
            .map(|p| MoveSequence::new(solution.moves()[p.moves.clone()].to_vec()))
            .collect()
    };
    let mut words = Vec::new();
    let mut count = 0;
    for part in &parts {
        if ranges {
            let (text, n) = compressed_text(notation, kind, part);
            words.push(text);
            count += n;
        } else {
            words.push(notation.write(kind, part));
            count += part.len();
        }
    }
    (words.join(if dots { " · " } else { " " }), count)
}
