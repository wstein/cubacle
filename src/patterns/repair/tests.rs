use super::*;
use crate::patterns::{self, Pattern};
use crate::puzzle::PuzzleKind;

fn planted(name: &str, kind: PuzzleKind, extra: &str, before: bool) -> Facelets {
    let pattern = patterns::find(name).unwrap();
    let moves: MoveSequence = extra.parse().unwrap();
    if before {
        let g = pattern
            .generator(kind.size(), None)
            .unwrap()
            .sequence()
            .unwrap();
        Facelets::solved(kind)
            .apply(&moves)
            .unwrap()
            .apply(&g)
            .unwrap()
    } else {
        pattern.state(kind).unwrap().apply(&moves).unwrap()
    }
}

#[test]
fn alphabet_moves_are_inverse_and_distinct() {
    let al = Alphabet::new(PuzzleKind::Cube4, Kit::Blocks);
    assert_eq!(al.len(), 81);
    let identity: Vec<u16> = (0..96).collect();
    for k in 0..al.len() {
        let i = al.inverse[k] as usize;
        assert_eq!(compose(&al.perms[k], &al.perms[i]), identity);
    }
    assert_eq!(al.count_up_to(1), 82);
}

#[test]
fn there_are_24_rotations() {
    assert_eq!(rotations(PuzzleKind::Cube3).len(), 24);
}

#[test]
fn finds_extra_moves_after_a_pattern() {
    let cube = planted("chessboard", PuzzleKind::Cube3, "R U2", false);
    let all: Vec<&Pattern> = patterns::all().iter().collect();
    let out = repair(&cube, &all, &Limits::default());
    let fix = out.found.expect("a fix");
    assert!(fix.blocks <= 2, "{} {}", fix.id, fix.moves);
}
