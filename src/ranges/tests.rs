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
