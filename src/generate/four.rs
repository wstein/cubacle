//! Random 4x4x4 states.
//!
//! Like the reference solver's generator, the eight corners (with twists
//! summing to zero), the 24 wings and the 24 centers are drawn independently
//! and uniformly. On a 4x4x4 the three piece kinds are not tied by parity,
//! and the centers are seen by color only, so every such combination is
//! reachable; `engines::four` solves each one by reduction.

use rand::RngExt;

use super::{orientations, pieces, place, shuffled};
use crate::puzzle::{locate, Face, Facelets, PuzzleKind};

pub(super) fn random(rng: &mut impl RngExt) -> Facelets {
    let kind = PuzzleKind::Cube4;
    let (corners, wings) = pieces(kind);
    let mut stickers = Facelets::solved(kind).stickers().to_vec();

    let wings: Vec<Vec<usize>> = wings.iter().map(|wing| chiral(kind, wing)).collect();
    let wing_perm = shuffled(rng, wings.len());
    place(
        kind,
        &mut stickers,
        &wings,
        &wing_perm,
        &vec![0; wings.len()],
    );

    let corner_perm = shuffled(rng, corners.len());
    let corner_twist = orientations(rng, corners.len(), 3);
    place(kind, &mut stickers, &corners, &corner_perm, &corner_twist);

    let centers: Vec<usize> = (0..kind.facelet_count())
        .filter(|&index| {
            let (position, _) = locate(kind, index);
            position.iter().filter(|c| c.abs() == 3).count() == 1
        })
        .collect();
    let colors: Vec<Face> = centers.iter().map(|&index| stickers[index]).collect();
    for (&slot, source) in centers.iter().zip(shuffled(rng, centers.len())) {
        stickers[slot] = colors[source];
    }
    Facelets::from_stickers(stickers).expect("a permutation of the solved colors")
}

/// The two stickers of a wing, ordered so that the cross product of their
/// normals points along the wing's offset from the edge middle. Turns keep
/// that handedness, so exchanging wings sticker by sticker never mirrors one.
fn chiral(kind: PuzzleKind, wing: &[usize]) -> Vec<usize> {
    let (position, first) = locate(kind, wing[0]);
    let (_, second) = locate(kind, wing[1]);
    let cross = [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ];
    let dot: i32 = (0..3)
        .filter(|&a| position[a].abs() == 1)
        .map(|a| cross[a] * position[a])
        .sum();
    if dot > 0 {
        vec![wing[0], wing[1]]
    } else {
        vec![wing[1], wing[0]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_states_keep_the_color_counts_and_vary() {
        let mut rng = rand::rng();
        let states: Vec<_> = (0..20).map(|_| random(&mut rng)).collect();
        for cube in &states {
            assert_eq!(cube.kind(), PuzzleKind::Cube4);
            for face in Face::ALL {
                let count = cube.stickers().iter().filter(|&&f| f == face).count();
                assert_eq!(count, 16);
            }
        }
        assert!(states.windows(2).any(|pair| pair[0] != pair[1]));
    }
}
