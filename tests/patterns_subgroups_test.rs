//! The subgroup tests of the pattern detector against the solver's own move
//! sets: a word in a set's letters must never be refuted by its subgroup test.

use cubacle::notation::parse_alphabet;
use cubacle::patterns::{detect, Verdict};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};

/// A deterministic word of `length` letters of the set.
fn word(set: &str, kind: PuzzleKind, seed: u64, length: usize) -> Facelets {
    let alphabet = parse_alphabet(set, kind).expect("a move set");
    let moves = alphabet.moves();
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut chosen = Vec::new();
    for _ in 0..length {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        chosen.push(moves[(x % moves.len() as u64) as usize]);
    }
    Facelets::solved(kind)
        .apply(&MoveSequence::new(chosen))
        .unwrap()
}

fn verdict(cube: &Facelets, name: &str) -> Verdict {
    detect(cube)
        .subgroups
        .iter()
        .find(|m| m.name.contains(name))
        .unwrap_or_else(|| panic!("no test named {name}"))
        .verdict
}

#[test]
fn half_turn_words_are_in_the_half_turn_group_and_g1() {
    for seed in 1..30 {
        let cube = word("half-turns", PuzzleKind::Cube3, seed, 25);
        assert_eq!(verdict(&cube, "half-turn group"), Verdict::Member);
        assert_eq!(verdict(&cube, "U/D axis"), Verdict::Member);
    }
}

#[test]
fn g1_words_are_in_g1_and_rarely_in_the_half_turn_group() {
    let mut outside = 0;
    for seed in 1..30 {
        let cube = word("<U,D,R2,L2,F2,B2>", PuzzleKind::Cube3, seed, 25);
        assert_eq!(verdict(&cube, "U/D axis"), Verdict::Member);
        if verdict(&cube, "half-turn group") == Verdict::NotMember {
            outside += 1;
        }
    }
    assert!(outside > 20, "{outside}");
}

#[test]
fn words_in_the_generated_groups_are_never_refuted() {
    for seed in 1..30 {
        let ru = word("<R,U>", PuzzleKind::Cube3, seed, 30);
        assert_eq!(verdict(&ru, "<R,U>"), Verdict::Possible);
        assert_eq!(verdict(&ru, "<R,U,F>"), Verdict::Possible);
        let ruf = word("<R,U,F>", PuzzleKind::Cube3, seed, 30);
        assert_eq!(verdict(&ruf, "<R,U,F>"), Verdict::Possible);
    }
}

#[test]
fn a_move_outside_the_group_is_refuted() {
    let cube = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"L".parse().unwrap())
        .unwrap();
    assert_eq!(verdict(&cube, "<R,U>"), Verdict::NotMember);
    assert_eq!(verdict(&cube, "<R,U,F>"), Verdict::NotMember);
    assert_eq!(verdict(&cube, "half-turn group"), Verdict::NotMember);
    assert_eq!(verdict(&cube, "U/D axis"), Verdict::NotMember);
}

#[test]
fn piece_class_tests_on_big_cubes() {
    let kind = PuzzleKind::Cube5;
    // Two center stickers of different faces swapped: only centers are out of place.
    let mut stickers = Facelets::solved(kind).stickers().to_vec();
    stickers.swap(6, 25 + 6);
    let centers = Facelets::from_stickers(stickers).unwrap();
    let report = detect(&centers);
    let find = |name: &str| {
        report
            .subgroups
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.verdict)
    };
    assert_eq!(find("corners only"), Some(Verdict::NotMember));
    assert_eq!(find("edges only"), Some(Verdict::NotMember));
    assert_eq!(find("centers only"), Some(Verdict::Member));
}
