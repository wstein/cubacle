//! The inverse of a solution, as `solve --inverse` and `moves --inverse` print it.

use cubacle::phases::Phase;
use cubacle::puzzle::MoveSequence;

/// The moves that undo `moves`: reversed, each turned the other way.
pub(crate) fn moves(moves: &MoveSequence) -> MoveSequence {
    moves.inverse()
}

/// The phases of the inverse of a solution of `total` moves: the list reversed,
/// every range mirrored, named `undo phase N: <name>` after the phase it undoes.
/// The names live as long as the process (a command prints one answer).
pub(crate) fn phases(found: &[Phase], total: usize) -> Vec<Phase> {
    found
        .iter()
        .rev()
        .enumerate()
        .map(|(at, phase)| Phase {
            number: at + 1,
            name: Box::leak(
                format!("undo phase {}: {}", phase.number, phase.name).into_boxed_str(),
            ),
            moves: (total - phase.moves.end)..(total - phase.moves.start),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inverse_reverses_and_turns_back() {
        let m: MoveSequence = "R U2 F'".parse().unwrap();
        assert_eq!(moves(&m).to_string(), "F U2 R'");
    }

    #[test]
    fn phases_are_mirrored_and_named_after_the_originals() {
        let phase = |number, name, moves| Phase {
            number,
            name,
            moves,
        };
        let found = [
            phase(1, "a", 0..3),
            phase(2, "b", 3..3),
            phase(3, "c", 3..10),
        ];
        let shown: Vec<_> = phases(&found, 10)
            .into_iter()
            .map(|p| (p.number, p.name, p.moves))
            .collect();
        assert_eq!(
            shown,
            [
                (1, "undo phase 3: c", 0..7),
                (2, "undo phase 2: b", 7..7),
                (3, "undo phase 1: a", 7..10),
            ]
        );
    }
}
