use super::*;
use crate::puzzle::Facelets;

fn seq(text: &str) -> MoveSequence {
    text.parse().unwrap()
}

fn show(kind: PuzzleKind, text: &str, notation: Notation) -> String {
    write(notation, kind, &compress(kind, &seq(text)))
}

#[test]
fn adjacent_layers_become_a_range() {
    assert_eq!(show(PuzzleKind::Cube5, "2R 3R", Notation::Sign), "2-3r");
    assert_eq!(show(PuzzleKind::Cube5, "2R 3R", Notation::Wca), "2-3Rw");
    assert_eq!(
        show(PuzzleKind::Cube5, "2R' 3R' 2L", Notation::Sign),
        "2-4r'"
    );
}

#[test]
fn layer_three_from_two_wide_turns() {
    assert_eq!(show(PuzzleKind::Cube5, "3Rw Rw'", Notation::Sign), "3R");
    assert_eq!(show(PuzzleKind::Cube5, "3Rw Rw'", Notation::Wca), "3R");
}

#[test]
fn nearer_face_is_used() {
    // Layers 4-5 from U of a 6x6x6 are layers 2-3 from D, turned the other way.
    assert_eq!(show(PuzzleKind::Cube6, "4U 5U", Notation::Sign), "2-3d'");
}

#[test]
fn nothing_shorter_is_left_alone() {
    assert_eq!(
        show(PuzzleKind::Cube4, "R U R' U'", Notation::Wca),
        "R U R' U'"
    );
    assert_eq!(show(PuzzleKind::Cube4, "R L", Notation::Wca), "R L");
}

#[test]
fn cancelling_turns_vanish() {
    assert_eq!(show(PuzzleKind::Cube3, "R L2 R'", Notation::Wca), "L2");
}

#[test]
fn compressed_replays_to_the_same_state() {
    let cases = [
        (PuzzleKind::Cube4, "Rw 2R' Lw U 2U Uw' 3Fw Fw' 2B 2D' D"),
        (PuzzleKind::Cube5, "3Rw Rw' 2R 4R U 3Uw 2U 3U' L 2L 3L R' D"),
        (PuzzleKind::Cube6, "3Rw Rw' 4R 5R2 3L' U 2U 4U 3D D2 Dw"),
        (PuzzleKind::Cube7, "2R 3R 4R 5R 6L2 3U 4U' 5U2 2F 3F' F 6B"),
    ];
    for (kind, text) in cases {
        let moves = seq(text);
        let packed = compress(kind, &moves);
        assert!(packed.len() <= moves.len(), "{text}");
        assert!(packed.iter().all(|mv| mv.fits(kind)), "{text}");
        let solved = Facelets::solved(kind);
        let want = solved.apply(&moves).unwrap();
        assert_eq!(want, solved.apply(&expand(&packed)).unwrap(), "{text}");
    }
}

/// Destination of every distinct sticker, independent of facelet colors.
fn permutation(kind: PuzzleKind, sequence: &MoveSequence) -> Vec<usize> {
    let mut result: Vec<_> = (0..kind.facelet_count()).collect();
    for mv in sequence.moves() {
        let step = crate::puzzle::destinations(kind, mv);
        for at in &mut result {
            *at = step[*at];
        }
    }
    result
}

#[test]
fn objectives_preserve_distinct_sticker_permutations() {
    let mut seed = 0x1284_901au64;
    for kind in PuzzleKind::ALL {
        let rotations = crate::puzzle::rotation_maps(kind);
        for _ in 0..100 {
            let mut sequence = Vec::new();
            for _ in 0..32 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let face = Face::ALL[((seed >> 32) % 6) as usize];
                let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise]
                    [((seed >> 40) % 3) as usize];
                let depth = 1 + ((seed >> 45) as usize % (kind.size() - 1));
                let wide = depth > 1 && (seed >> 50) & 1 == 0;
                sequence.push(Move::new(face, depth as u8, wide, turn));
            }
            let input = MoveSequence::new(sequence);
            let expected = permutation(kind, &input);
            let moves = expand(&optimize(kind, &input, Optimize::Moves));
            assert_eq!(permutation(kind, &moves), expected, "{kind:?} {input}");
            let easy = expand(&optimize(kind, &input, Optimize::Easy));
            let got = permutation(kind, &easy);
            assert!(
                rotations.iter().any(|rotation| {
                    got.iter()
                        .zip(&expected)
                        .all(|(&at, &want)| rotation[at] == want)
                }),
                "{kind:?} {input}"
            );
        }
    }
}
