//! Random 6x6x6 and 7x7x7 states.
//!
//! Like the 4x4x4 and 5x5x5 generators, the pieces of each orbit are drawn
//! independently and uniformly: the corners (and on an odd cube the middle
//! edges, as a uniform 3x3x3 core), the 24 wings of every depth, and the
//! colours of every centre orbit. The format Orbit64 counts the states of
//! these cubes the same way (a permutation per wing orbit, a colour
//! arrangement per centre orbit), so every such combination is reachable.

use rand::RngExt;

use super::{orientations, pieces, place, random_state, shuffled};
use crate::puzzle::{centre_orbits, wing_orbits, Face, Facelets, PuzzleKind};

pub(super) fn random(kind: PuzzleKind, rng: &mut impl RngExt) -> Facelets {
    let n = kind.size();
    let mut stickers = Facelets::solved(kind).stickers().to_vec();

    if n % 2 == 1 {
        let core = random_state(PuzzleKind::Cube3).expect("the 3x3x3 has a generator");
        let at = [0, n / 2, n - 1];
        for (index, &face) in core.stickers().iter().enumerate() {
            let (side, local) = (index / 9, index % 9);
            stickers[side * n * n + at[local / 3] * n + at[local % 3]] = face;
        }
    } else {
        let (corners, _) = pieces(kind);
        let perm = shuffled(rng, corners.len());
        let twist = orientations(rng, corners.len(), 3);
        place(kind, &mut stickers, &corners, &perm, &twist);
    }

    for orbit in wing_orbits(kind) {
        let slots: Vec<Vec<usize>> = orbit.iter().map(|wing| wing.to_vec()).collect();
        let perm = shuffled(rng, slots.len());
        place(kind, &mut stickers, &slots, &perm, &vec![0; slots.len()]);
    }

    for orbit in centre_orbits(kind) {
        let colours: Vec<Face> = orbit
            .iter()
            .map(|&cell| Face::ALL[cell / kind.face_len()])
            .collect();
        for (&cell, source) in orbit.iter().zip(shuffled(rng, orbit.len())) {
            stickers[cell] = colours[source];
        }
    }
    Facelets::from_stickers(stickers).expect("a permutation of the solved colours")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_states_keep_the_colour_counts_and_vary() {
        let mut rng = rand::rng();
        for kind in [PuzzleKind::Cube6, PuzzleKind::Cube7] {
            let states: Vec<_> = (0..10).map(|_| random(kind, &mut rng)).collect();
            for cube in &states {
                assert_eq!(cube.kind(), kind);
                if kind == PuzzleKind::Cube7 {
                    for face in Face::ALL {
                        assert_eq!(cube.stickers()[face as usize * 49 + 24], face);
                    }
                }
            }
            assert!(states.windows(2).any(|pair| pair[0] != pair[1]));
        }
    }
}
