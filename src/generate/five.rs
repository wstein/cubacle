//! Random 5x5x5 states.
//!
//! With the face centres fixed, a state is a 3x3x3 state (corners and middle
//! edges), a colour arrangement of the x-centres and of the t-centres, and a
//! permutation of the 24 wings. Every such combination is reachable: the
//! centres are only seen by colour, so the parity ties between the piece
//! kinds can always be met by exchanging two equal centres, and the 5x5x5
//! engine solves each one by reduction (`engines::five`, tested on generated
//! states). The 3x3x3 part is uniform by [`super::random_state`]; the
//! arrangements and the wing permutation are uniform shuffles.

use rand::RngExt;

use super::{random_state, shuffled};
use crate::puzzle::{five_orbits, Face, Facelets, PuzzleKind};

pub(super) fn random(rng: &mut impl RngExt) -> Facelets {
    let core = random_state(PuzzleKind::Cube3).expect("the 3x3x3 has a generator");
    let mut stickers = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
    for (index, &face) in core.stickers().iter().enumerate() {
        let (side, local) = (index / 9, index % 9);
        stickers[side * 25 + (local / 3) * 10 + (local % 3) * 2] = face;
    }

    let orbits = five_orbits();
    for slots in [&orbits.x, &orbits.t] {
        let colours: Vec<Face> = slots.iter().map(|&slot| stickers[slot]).collect();
        for (&slot, source) in slots.iter().zip(shuffled(rng, slots.len())) {
            stickers[slot] = colours[source];
        }
    }

    let colours: Vec<[Face; 2]> = orbits
        .wings
        .iter()
        .map(|wing| wing.map(|slot| stickers[slot]))
        .collect();
    for (wing, source) in orbits.wings.iter().zip(shuffled(rng, colours.len())) {
        stickers[wing[0]] = colours[source][0];
        stickers[wing[1]] = colours[source][1];
    }
    Facelets::from_stickers(stickers).expect("a permutation of the solved colours")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_states_keep_the_centres_and_the_colour_counts() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let cube = random(&mut rng);
            assert_eq!(cube.kind(), PuzzleKind::Cube5);
            for face in Face::ALL {
                assert_eq!(cube.stickers()[face as usize * 25 + 12], face);
            }
        }
    }
}
