//! The whole-cube rotations, as wide turns.
//!
//! New code (the reference prints the rotation as `x`, `y`, `z` letters; this
//! solver's solutions contain layer turns only, so the rotation is made of
//! them).
//!
//! `x` is `Rw Lw'`, `y` is `Uw Dw'` and `z` is `Fw Bw'`, each turn of two
//! layers. Every one of the 24 rotations is at most two of these, so at most
//! four turns.

use crate::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// The 24 rotations of the cube, each as the shortest list of wide turns
/// found, the identity first.
pub(super) struct Rotations {
    list: Vec<Vec<Move>>,
}

fn axis(axis: usize, turn: Turn) -> Vec<Move> {
    let (a, b) = match axis {
        0 => (Face::R, Face::L),
        1 => (Face::U, Face::D),
        _ => (Face::F, Face::B),
    };
    let back = match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    };
    vec![Move::new(a, 2, true, turn), Move::new(b, 2, true, back)]
}

impl Rotations {
    pub(super) fn new() -> Rotations {
        let turns = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];
        let mut candidates: Vec<Vec<Move>> = vec![Vec::new()];
        for a in 0..3 {
            for ta in turns {
                candidates.push(axis(a, ta));
            }
        }
        for a in 0..3 {
            for ta in turns {
                for b in 0..3 {
                    if a == b {
                        continue;
                    }
                    for tb in turns {
                        let mut both = axis(a, ta);
                        both.extend(axis(b, tb));
                        candidates.push(both);
                    }
                }
            }
        }
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let mut seen: Vec<Facelets> = Vec::new();
        let mut list = Vec::new();
        for moves in candidates {
            let effect = solved
                .apply(&MoveSequence::new(moves.clone()))
                .expect("wide turns exist on a 4x4x4");
            if !seen.contains(&effect) {
                seen.push(effect);
                list.push(moves);
            }
        }
        assert_eq!(list.len(), 24);
        Rotations { list }
    }

    /// The shortest rotation after which `done` holds for `cube`.
    pub(super) fn find(
        &self,
        cube: &Facelets,
        done: impl Fn(&Facelets) -> bool,
    ) -> Option<&[Move]> {
        self.list.iter().map(Vec::as_slice).find(|moves| {
            cube.apply(&MoveSequence::new(moves.to_vec()))
                .is_ok_and(|turned| done(&turned))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_24_rotations_of_at_most_four_turns() {
        let r = Rotations::new();
        assert_eq!(r.list.len(), 24);
        assert!(r.list[0].is_empty());
        assert!(r.list.iter().all(|moves| moves.len() <= 4));
        assert_eq!(r.list.iter().filter(|m| m.len() == 4).count(), 14);
    }

    #[test]
    fn a_rotated_solved_cube_is_brought_back() {
        let r = Rotations::new();
        for moves in &r.list {
            let rotated = Facelets::solved(PuzzleKind::Cube4)
                .apply(&MoveSequence::new(moves.clone()))
                .unwrap();
            let back = r
                .find(&rotated, Facelets::is_solved)
                .expect("a rotation back");
            let home = rotated.apply(&MoveSequence::new(back.to_vec())).unwrap();
            assert!(home.is_solved());
        }
    }
}
