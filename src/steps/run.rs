//! The steps of a picture list: one action per picture, with the text the
//! move line writes for it.
//!
//! A plain solution has one step per move; a compressed one (`--optimize
//! ranges`) has one step per range or block move, so a block of layers is one
//! arrow in one picture.

use cubacle::notation::Notation;
use cubacle::phases::Phase;
use cubacle::puzzle::{LayerMove, MoveSequence, PuzzleKind, Turn};
use cubacle::ranges;
use cubacle::render::Action;

use super::lone_turns;

/// What is drawn, step by step.
pub(crate) struct Run {
    pub actions: Vec<Action>,
    /// The notation of each step for the move line.
    pub written: Vec<String>,
    /// Whether each step is a quarter turn between two half turns.
    pub lone: Vec<bool>,
}

impl Run {
    /// One step per move.
    pub(crate) fn plain(solution: &MoveSequence, notation: Notation, kind: PuzzleKind) -> Run {
        let moves = solution.moves();
        let turns: Vec<Turn> = moves.iter().map(|mv| mv.turn).collect();
        Run {
            actions: moves.iter().map(|mv| Action::Layer(*mv)).collect(),
            written: moves
                .iter()
                .map(|mv| notation.write_move(kind, mv))
                .collect(),
            lone: lone_turns(&turns),
        }
    }

    /// One step per compressed move.
    pub(crate) fn packed(moves: &[LayerMove], notation: Notation, kind: PuzzleKind) -> Run {
        let turns: Vec<Turn> = moves.iter().map(|mv| mv.turn).collect();
        Run {
            actions: moves.iter().map(|mv| Action::block(*mv)).collect(),
            written: moves
                .iter()
                .map(|mv| ranges::write_move(notation, kind, mv))
                .collect(),
            lone: lone_turns(&turns),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.actions.len()
    }
}

/// A solution compressed as a whole, so a range may span a phase border: the
/// steps, and the phases renumbered to them. A step belongs to the phase in
/// which it starts.
pub(crate) fn compress_phases(
    kind: PuzzleKind,
    solution: &MoveSequence,
    found: &[Phase],
) -> (Vec<LayerMove>, Vec<Phase>) {
    let indexed = ranges::compress_indexed(kind, solution);
    let steps: Vec<LayerMove> = indexed.iter().map(|(mv, _)| *mv).collect();
    let covered = !found.is_empty()
        && found[0].moves.start == 0
        && found.windows(2).all(|w| w[0].moves.end == w[1].moves.start)
        && found.last().is_some_and(|p| p.moves.end == solution.len());
    if !covered {
        return (steps, Vec::new());
    }
    let mut phases = Vec::new();
    let mut next = 0;
    for phase in found {
        let start = next;
        while next < indexed.len() && indexed[next].1 < phase.moves.end {
            next += 1;
        }
        phases.push(Phase {
            number: phase.number,
            name: phase.name,
            moves: start..next,
        });
    }
    (steps, phases)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubacle::generate::scramble;
    use cubacle::puzzle::Facelets;

    /// The drawn steps, applied one by one, reach the state of the plain moves,
    /// and there are never more steps than moves.
    #[test]
    fn the_steps_replay_to_the_state_of_the_plain_moves() {
        for kind in [
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            for _ in 0..20 {
                // runs of one axis, as a solution has them
                let moves = scramble(kind, 60);
                let cut = moves.len() / 3;
                let phase = |number, range| Phase {
                    number,
                    name: "p",
                    moves: range,
                };
                let found = [phase(1, 0..cut), phase(2, cut..moves.len())];
                let (steps, phases) = compress_phases(kind, &moves, &found);
                assert!(steps.len() <= moves.len());
                assert_eq!(phases.last().map(|p| p.moves.end), Some(steps.len()));
                let run = Run::packed(&steps, Notation::Wca, kind);
                let start = Facelets::solved(kind);
                let mut state = start.clone();
                for action in &run.actions {
                    state = action.apply(&state).unwrap();
                }
                assert_eq!(state, start.apply(&moves).unwrap(), "{kind:?}");
            }
        }
    }

    #[test]
    fn a_range_spans_a_phase_border_and_belongs_to_where_it_starts() {
        let solution: MoveSequence = "U 2R 3R U".parse().unwrap();
        let phase = |number, moves| Phase {
            number,
            name: "p",
            moves,
        };
        let found = [phase(1, 0..2), phase(2, 2..4)];
        let (steps, phases) = compress_phases(PuzzleKind::Cube5, &solution, &found);
        assert_eq!(steps.len(), 3);
        assert_eq!(phases[0].moves, 0..2);
        assert_eq!(phases[1].moves, 2..3);
    }

    /// The pictures are as many as the words of the compressed answer.
    #[test]
    fn the_pictures_count_the_compressed_words() {
        for kind in [
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            for _ in 0..20 {
                let moves = scramble(kind, 80);
                let cut = moves.len() / 2;
                let phase = |number, range| Phase {
                    number,
                    name: "p",
                    moves: range,
                };
                let found = [phase(1, 0..cut), phase(2, cut..moves.len())];
                let (steps, _) = compress_phases(kind, &moves, &found);
                let (_, words) = ranges::compressed_text(Notation::Wca, kind, &moves);
                assert_eq!(steps.len(), words);
                assert_eq!(Run::packed(&steps, Notation::Wca, kind).len(), words);
            }
        }
    }
}
