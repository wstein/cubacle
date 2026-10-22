//! Random cube states and scrambles.
//!
//! Depends only on [`crate::puzzle`]; it knows nothing about the engines, so
//! any engine's input can be produced here.

use std::collections::BTreeMap;

mod five;

use rand::RngExt;

use crate::puzzle::{locate, Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// A random scramble of `length` turns: outer-face turns, plus the second
/// layer on a 5x5x5 (the layers that leave the face centres in place).
///
/// No two consecutive turns act on the same face, and turns of opposite faces
/// always come in the same order, so the sequence has no redundant pairs such
/// as `R R` or `R L R`.
pub fn scramble(kind: PuzzleKind, length: usize) -> MoveSequence {
    let layers = if kind == PuzzleKind::Cube5 { 2 } else { 1 };
    let mut rng = rand::rng();
    let mut last_axis = 18;
    let mut moves = Vec::with_capacity(length);
    while moves.len() < length {
        let index = rng.random_range(0..18);
        let axis = index / 3;
        if axis == last_axis || (axis % 3 == last_axis % 3 && axis > last_axis) {
            continue;
        }
        last_axis = axis;
        let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][index % 3];
        let depth = rng.random_range(1..=layers);
        moves.push(Move::new(Face::ALL[axis], depth, false, turn));
    }
    MoveSequence::new(moves)
}

/// A random reachable state, or `None` for sizes without a generator (the
/// 2x2x2, 3x3x3 and 5x5x5 have one).
///
/// The 2x2x2 is a uniformly random arrangement of the eight corners with
/// twists summing to zero. The 3x3x3 adds the edges, with a flip sum of zero
/// and the permutation parities of corners and edges tied together. The
/// 5x5x5 (centres fixed) is a uniform 3x3x3 core with shuffled centres and
/// wings; see `five.rs` for why every such state is reachable.
pub fn random_state(kind: PuzzleKind) -> Option<Facelets> {
    let mut rng = rand::rng();
    if kind == PuzzleKind::Cube5 {
        return Some(five::random(&mut rng));
    }
    if !matches!(kind, PuzzleKind::Cube2 | PuzzleKind::Cube3) {
        return None;
    }
    let (corners, edges) = pieces(kind);

    let corner_perm = shuffled(&mut rng, corners.len());
    let corner_twist = orientations(&mut rng, corners.len(), 3);
    let mut stickers = Facelets::solved(kind).stickers().to_vec();
    if edges.is_empty() {
        place(kind, &mut stickers, &corners, &corner_perm, &corner_twist);
    } else {
        let mut edge_perm = shuffled(&mut rng, edges.len());
        if inversions(&corner_perm) % 2 != inversions(&edge_perm) % 2 {
            edge_perm.swap(0, 1);
        }
        let edge_flip = orientations(&mut rng, edges.len(), 2);
        place(kind, &mut stickers, &edges, &edge_perm, &edge_flip);
        place(kind, &mut stickers, &corners, &corner_perm, &corner_twist);
    }
    Some(Facelets::from_stickers(stickers).expect("a permutation of the solved colours"))
}

/// The sticker indices of every corner and every edge, each in the cyclic
/// order that defines orientation: the reference sticker (on the U/D axis,
/// else the F/B axis) first, the others following with a fixed handedness.
fn pieces(kind: PuzzleKind) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let reach = kind.size() as i32 - 1;
    let mut by_position: BTreeMap<[i32; 3], Vec<(usize, [i32; 3])>> = BTreeMap::new();
    for index in 0..kind.facelet_count() {
        let (position, normal) = locate(kind, index);
        let outer = position.iter().filter(|c| c.abs() == reach).count();
        if outer >= 2 {
            by_position
                .entry(position)
                .or_default()
                .push((index, normal));
        }
    }
    let axis = |normal: [i32; 3]| normal.iter().position(|&c| c != 0).expect("unit normal");
    let (mut corners, mut edges) = (Vec::new(), Vec::new());
    for (position, mut stickers) in by_position {
        if stickers.len() == 3 {
            // Order y, z, x when the corner is right-handed, y, x, z otherwise.
            let handed = position.iter().map(|c| c.signum()).product::<i32>() > 0;
            let order: [usize; 3] = if handed { [1, 2, 0] } else { [1, 0, 2] };
            stickers.sort_by_key(|&(_, normal)| order.iter().position(|&a| a == axis(normal)));
            corners.push(stickers.into_iter().map(|(index, _)| index).collect());
        } else {
            let priority = |normal| [2usize, 0, 1][axis(normal)];
            stickers.sort_by_key(|&(_, normal)| priority(normal));
            edges.push(stickers.into_iter().map(|(index, _)| index).collect());
        }
    }
    (corners, edges)
}

/// Puts piece `perm[slot]`, turned by `twist[slot]`, into every slot.
fn place(
    kind: PuzzleKind,
    stickers: &mut [Face],
    slots: &[Vec<usize>],
    perm: &[usize],
    twist: &[usize],
) {
    for (slot, indices) in slots.iter().enumerate() {
        let piece = &slots[perm[slot]];
        for (k, &index) in indices.iter().enumerate() {
            let source = piece[(k + twist[slot]) % piece.len()];
            stickers[index] = Face::ALL[source / kind.face_len()];
        }
    }
}

pub(super) fn shuffled(rng: &mut impl RngExt, len: usize) -> Vec<usize> {
    let mut items: Vec<usize> = (0..len).collect();
    for i in (1..len).rev() {
        items.swap(i, rng.random_range(0..=i));
    }
    items
}

/// Random orientations in `0..radix` whose sum is a multiple of `radix`.
fn orientations(rng: &mut impl RngExt, count: usize, radix: usize) -> Vec<usize> {
    let mut values: Vec<usize> = (0..count - 1).map(|_| rng.random_range(0..radix)).collect();
    let sum: usize = values.iter().sum();
    values.push((radix - sum % radix) % radix);
    values
}

fn inversions(perm: &[usize]) -> usize {
    (0..perm.len())
        .flat_map(|i| (i + 1..perm.len()).map(move |j| (i, j)))
        .filter(|&(i, j)| perm[i] > perm[j])
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrambles_have_the_requested_length_and_no_redundant_pairs() {
        for length in [0, 1, 25, 100] {
            let moves = scramble(PuzzleKind::Cube3, length);
            assert_eq!(moves.len(), length);
            for pair in moves.moves().windows(2) {
                assert_ne!(pair[0].face, pair[1].face);
                let (a, b) = (pair[0].face.index(), pair[1].face.index());
                assert!(!(a % 3 == b % 3 && b > a), "{} {}", pair[0], pair[1]);
            }
            assert!(moves.fits(PuzzleKind::Cube3));
        }
    }

    #[test]
    fn a_5x5_scramble_turns_the_first_two_layers_and_keeps_the_centres() {
        let moves = scramble(PuzzleKind::Cube5, 200);
        assert!(moves.moves().iter().all(|m| m.fits(PuzzleKind::Cube5)));
        assert!(moves.moves().iter().any(|m| m.depth == 2));
        assert!(moves.moves().iter().all(|m| m.depth <= 2 && !m.wide));
        let cube = Facelets::solved(PuzzleKind::Cube5).apply(&moves).unwrap();
        for face in Face::ALL {
            assert_eq!(cube.stickers()[face as usize * 25 + 12], face);
        }
    }

    #[test]
    fn only_the_2x2_3x3_and_5x5_have_a_state_generator() {
        assert!(random_state(PuzzleKind::Cube2).is_some());
        assert!(random_state(PuzzleKind::Cube3).is_some());
        assert!(random_state(PuzzleKind::Cube5).is_some());
        assert!(random_state(PuzzleKind::Cube4).is_none());
    }

    #[test]
    fn generated_states_are_well_formed_and_vary() {
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3] {
            let states: Vec<_> = (0..20).map(|_| random_state(kind).unwrap()).collect();
            assert!(states.iter().all(|s| s.kind() == kind));
            assert!(states.windows(2).any(|pair| pair[0] != pair[1]));
        }
    }

    #[test]
    fn piece_counts_are_those_of_a_cube() {
        let (corners, edges) = pieces(PuzzleKind::Cube3);
        assert_eq!((corners.len(), edges.len()), (8, 12));
        let (corners, edges) = pieces(PuzzleKind::Cube2);
        assert_eq!((corners.len(), edges.len()), (8, 0));
    }
}
