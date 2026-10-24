//! `solve --group phases`: the solution written with ` · ` between solver phases.

use cubacle::notation::{axis_units, Item, Notation};
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

/// The solution as text and its move count. Every part is written on its own,
/// so ranges never merge across a dot; with dots the parts are joined by
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
