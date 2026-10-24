//! A middle dot `·` separates moves like a space, in plain move lists and in expressions.

use cubacle::notation::{parse_expression, parse_moves};
use cubacle::puzzle::PuzzleKind;

#[test]
fn a_middle_dot_separates_plain_moves() {
    let dotted = parse_moves("R·U·R'·U'").unwrap();
    assert_eq!(dotted, parse_moves("R U R' U'").unwrap());
    let mixed = parse_moves("R · U  ·R'· U'").unwrap();
    assert_eq!(mixed, dotted);
}

#[test]
fn a_middle_dot_separates_moves_in_expressions() {
    let kind = PuzzleKind::Cube3;
    let spaced = parse_expression("(R U R' U')*3 [F, R]", kind).unwrap();
    let dotted = parse_expression("(R·U·R'·U')*3·[F,·R]", kind).unwrap();
    assert_eq!(dotted, spaced);
}

#[test]
fn a_middle_dot_separates_ranges_and_wide_moves() {
    let kind = PuzzleKind::Cube5;
    let spaced = parse_expression("2-3r Rw' 3Uw2 M", kind).unwrap();
    let dotted = parse_expression("2-3r·Rw'·3Uw2·M", kind).unwrap();
    assert_eq!(dotted, spaced);
}

#[test]
fn a_lone_or_doubled_dot_is_harmless_but_a_dot_inside_a_move_is_an_error() {
    assert_eq!(
        parse_moves("R··U").unwrap(),
        parse_moves("R U").unwrap(),
        "empty items between dots are skipped like extra spaces"
    );
    assert!(parse_moves("R·'").is_err());
}

#[test]
fn dots_between_spaces_group_sse_moves() {
    let kind = PuzzleKind::Cube7;
    let plain = parse_expression(
        "VB VD VL VD2 VL' VD VB2 VD' VL' VR' VU' VB VU2 VB' VU2 VF'",
        kind,
    );
    let grouped = parse_expression(
        "VB VD VL VD2 VL' VD VB2 VD' · VL' VR' · VU' VB VU2 VB' VU2 VF'",
        kind,
    );
    assert_eq!(grouped.unwrap(), plain.unwrap());
}
