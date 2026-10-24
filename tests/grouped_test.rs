//! `parse_grouped` keeps the hand units of `·` and `( )`; the flat moves stay those of
//! `parse_expression`.

use cubacle::notation::{parse_expression, parse_grouped, Group, Item};
use cubacle::puzzle::PuzzleKind;

const OWNER: &str = "F' · U D · L' U' (ND' NU · R U2 L' · NF NB' · L U2 R') U L · U' D' · F";

/// Shape: each root item as the number of moves of a unit, or the sizes of the units of a block.
fn shape(group: &Group) -> Vec<Vec<usize>> {
    group
        .items
        .iter()
        .map(|item| match item {
            Item::Move(_) => vec![1],
            Item::Group(g) if g.is_unit() => vec![g.items.len()],
            Item::Group(g) => g
                .items
                .iter()
                .map(|i| match i {
                    Item::Group(u) => u.flatten().len(),
                    Item::Move(_) => 1,
                })
                .collect(),
        })
        .collect()
}

#[test]
fn the_owner_example_has_nine_groups() {
    let kind = PuzzleKind::of_size(7).unwrap();
    let tree = parse_grouped(OWNER, kind).unwrap();
    // F' | U D | L' U' | ( 4 units ) | U L | U' D' | F
    let units = shape(&tree);
    assert_eq!(units.len(), 7);
    assert_eq!(units[3].len(), 4);
    assert_eq!(units[2], vec![2]);
    assert_eq!(
        tree.flatten(),
        parse_expression(OWNER, kind).unwrap().moves()
    );
}

#[test]
fn flat_moves_equal_the_expression_for_every_input() {
    let kind = PuzzleKind::Cube3;
    for text in [
        "R U R' U'",
        "R·U·R'·U'",
        "(R U)*3 · F",
        "(R · U)3",
        "((R · U) F)2' · D",
        "[R · U, F · D] · L",
        "[R: U · F]",
        "(R U)'",
        "·· R ·· U ··",
        "(((R · U) · F) · D) · L",
        "()",
        "(·)*5 R",
        "",
    ] {
        let flat = parse_expression(text, kind).unwrap();
        let tree = parse_grouped(text, kind).unwrap();
        assert_eq!(tree.flatten(), flat.moves(), "{text}");
    }
}

#[test]
fn errors_are_the_same() {
    let kind = PuzzleKind::Cube3;
    for text in ["(R", "R)", "R*", "Q", "[R]", "(R U)*0", "R,"] {
        assert_eq!(
            parse_grouped(text, kind)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            parse_expression(text, kind)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            "{text}"
        );
    }
}

#[test]
fn repetition_repeats_the_groups() {
    let kind = PuzzleKind::Cube3;
    let tree = parse_grouped("(R · U)*2 F", kind).unwrap();
    assert_eq!(shape(&tree), vec![vec![1, 1], vec![1, 1], vec![1]]);
    let plain = parse_grouped("R U · F", kind).unwrap();
    assert_eq!(shape(&plain), vec![vec![2], vec![1]]);
}

#[test]
fn inversion_mirrors_the_groups() {
    let kind = PuzzleKind::Cube3;
    let tree = parse_grouped("(R U · F)'", kind).unwrap();
    // F' | U' R' as two units
    assert_eq!(shape(&tree), vec![vec![1, 2]]);
}

#[test]
fn commutator_groups_follow_the_moves() {
    let kind = PuzzleKind::Cube3;
    let tree = parse_grouped("[R · U, F]", kind).unwrap();
    // R | U F U' | R' F': the dot of the first part is mirrored in its inverse
    assert_eq!(tree.flatten().len(), 6);
    assert_eq!(shape(&tree), vec![vec![1], vec![3], vec![2]]);
}

#[test]
fn deeper_parentheses_are_flattened() {
    let kind = PuzzleKind::Cube3;
    let tree = parse_grouped("(R (U F) · D) L", kind).unwrap();
    assert_eq!(shape(&tree), vec![vec![3, 1], vec![1]]);
}
