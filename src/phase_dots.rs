//! `solve --group phases`: the solution written with ` · ` between solver phases.

use cubacle::notation::{axis_units, Item, Notation};
use cubacle::phases::Phase;
use cubacle::puzzle::{MoveSequence, PuzzleKind};
use cubacle::ranges::{compress_indexed, compressed_text};

/// How `solve` marks the solver phases in its answer.
#[derive(Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum SolveGroup {
    /// A plain list of moves
    #[default]
    None,
    /// A " · " between the phases of the solver (none for a one-phase solution)
    Phases,
    /// A " · " between hand units by axis: turns about one axis on disjoint
    /// layers (`U D'`) are one unit. With `phases` also, a unit never crosses
    /// a phase border and the border is a dot too
    Axis,
}

/// The groupings asked for, as `--group phases,axis` lists them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Grouping {
    pub phases: bool,
    pub axis: bool,
}

impl Grouping {
    pub fn of(groups: &[SolveGroup]) -> Grouping {
        Grouping {
            phases: groups.contains(&SolveGroup::Phases),
            axis: groups.contains(&SolveGroup::Axis),
        }
    }

    /// Whether the answer has dots at all.
    pub fn dots(self) -> bool {
        self.phases || self.axis
    }
}

/// The solution compressed as a whole and cut at the phase borders: a range
/// that spans a border stays in the phase where it starts.
fn packed_phases(
    notation: Notation,
    kind: PuzzleKind,
    solution: &MoveSequence,
    phases: &[Phase],
) -> (String, usize) {
    let indexed = compress_indexed(kind, solution);
    let mut words = Vec::new();
    let mut next = 0;
    for phase in phases.iter().filter(|p| !p.moves.is_empty()) {
        let start = next;
        while next < indexed.len() && indexed[next].1 < phase.moves.end {
            next += 1;
        }
        if next > start {
            let part: Vec<_> = indexed[start..next].iter().map(|(mv, _)| *mv).collect();
            words.push(cubacle::ranges::write(notation, kind, &part));
        }
    }
    (words.join(" · "), indexed.len())
}

/// The parts of a solution of `total` moves cut after every `every` moves
/// (the last one may be shorter), as phases named `moves`.
pub fn every(total: usize, every: usize) -> Vec<Phase> {
    (0..total)
        .step_by(every.max(1))
        .enumerate()
        .map(|(i, from)| Phase {
            number: i + 1,
            name: "moves",
            moves: from..(from + every.max(1)).min(total),
        })
        .collect()
}

/// The parts of a solution cut after every `every` steps of its compressed
/// form (`--optimize ranges`): phases in the moves of the solution, each
/// starting where its first compressed step starts.
pub fn every_packed(kind: PuzzleKind, solution: &MoveSequence, every: usize) -> Vec<Phase> {
    let indexed = compress_indexed(kind, solution);
    let starts: Vec<usize> = indexed
        .iter()
        .step_by(every.max(1))
        .map(|(_, start)| *start)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, &from)| Phase {
            number: i + 1,
            name: "moves",
            moves: from..starts.get(i + 1).copied().unwrap_or(solution.len()),
        })
        .collect()
}

/// The solution as text and its move count. With `axis` every part is written
/// on its own, so ranges never merge across a dot; with dots the parts are joined by
/// ` · `, otherwise by a space. The parts are the non-empty solver phases
/// (`phases`), cut again into hand units by axis (`axis`; a unit never crosses
/// a phase border, the border only ever ends a unit), or the whole solution.
pub fn write(
    notation: Notation,
    kind: PuzzleKind,
    solution: &MoveSequence,
    phases: &[Phase],
    ranges: bool,
    grouping: Grouping,
) -> (String, usize) {
    let borders: Vec<usize> = if grouping.phases {
        phases
            .iter()
            .filter(|p| !p.moves.is_empty())
            .map(|p| p.moves.start)
            .collect()
    } else {
        Vec::new()
    };
    let moves = solution.moves();
    let parts: Vec<MoveSequence> = if grouping.axis {
        axis_units(moves, kind, &borders)
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Group(unit) => Some(MoveSequence::new(unit.flatten())),
                Item::Move(_) => None,
            })
            .collect()
    } else if borders.is_empty() {
        vec![solution.clone()]
    } else if ranges {
        return packed_phases(notation, kind, solution, phases);
    } else {
        phases
            .iter()
            .filter(|p| !p.moves.is_empty())
            .map(|p| MoveSequence::new(moves[p.moves.clone()].to_vec()))
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
    (words.join(if grouping.dots() { " · " } else { " " }), count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phase(number: usize, moves: std::ops::Range<usize>) -> Phase {
        Phase {
            number,
            name: "test",
            moves,
        }
    }

    #[test]
    fn a_fixed_size_cut_covers_the_moves() {
        let parts = every(25, 10);
        let ranges: Vec<_> = parts.iter().map(|p| p.moves.clone()).collect();
        assert_eq!(ranges, [0..10, 10..20, 20..25]);
        assert!(every(0, 10).is_empty());
    }

    #[test]
    fn a_fixed_size_cut_counts_compressed_steps() {
        let solution: MoveSequence = "R L U D F B R2 L2".parse().unwrap();
        let kind = PuzzleKind::Cube3;
        let parts = every_packed(kind, &solution, 2);
        assert!(!parts.is_empty());
        let grouping = Grouping {
            phases: true,
            axis: false,
        };
        let (text, count) = write(Notation::Wca, kind, &solution, &parts, true, grouping);
        let cuts = text.matches(" · ").count();
        assert_eq!(cuts, count.div_ceil(2) - 1, "{text}");
    }

    #[test]
    fn a_phase_border_ends_an_axial_unit() {
        let solution: MoveSequence = "U D' R".parse().unwrap();
        let phases = [phase(1, 0..1), phase(2, 1..3)];
        let text = |phases_on, axis| {
            let grouping = Grouping {
                phases: phases_on,
                axis,
            };
            write(
                Notation::Wca,
                PuzzleKind::Cube3,
                &solution,
                &phases,
                false,
                grouping,
            )
            .0
        };
        assert_eq!(text(false, false), "U D' R");
        assert_eq!(text(true, false), "U · D' R");
        assert_eq!(text(false, true), "U D' · R");
        assert_eq!(text(true, true), "U · D' · R");
    }
}
