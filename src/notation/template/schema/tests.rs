use super::*;

fn scheme() -> ColourScheme {
    "URFDLB".parse().unwrap()
}

fn parse2(text: &str) -> Result<Schema, TemplateError> {
    parse(text, PuzzleKind::Cube2, &scheme())
}

fn error2(text: &str) -> String {
    parse2(text).unwrap_err().to_string()
}

#[test]
fn six_faces_of_a_2x2() {
    let schema =
        parse2("a1 a1 b1 b1 / a2 a2 b2 b2 / . . ? ? / U R F D / x x x x / y, y, y, y").unwrap();
    assert_eq!(schema.slots.len(), 24);
    assert_eq!(schema.names, ["a1", "b1", "a2", "b2", "x", "y"]);
    assert_eq!(schema.slots[0], Slot::Var(0));
    assert_eq!(schema.slots[2], Slot::Var(1));
    assert_eq!(schema.slots[8], Slot::Any);
    assert_eq!(schema.slots[12], Slot::Fixed(Face::U));
    assert_eq!(schema.slots[15], Slot::Fixed(Face::D));
}

#[test]
fn a_number_is_part_of_the_name() {
    // a and a1 are two variables; a1 is not "a on face 1".
    let schema = parse2("a a a1 a1 / a a a1 a1 / . . . . / . . . . / . . . . / . . . .").unwrap();
    assert_eq!(schema.names, ["a", "a1"]);
}

#[test]
fn faces_may_end_with_newlines_and_blank_lines() {
    let text = "a a a a\n\nb b b b\nc c c c / d d d d\n\n\ne e e e\nf f f f\n";
    let schema = parse2(text).unwrap();
    assert_eq!(schema.names, ["a", "b", "c", "d", "e", "f"]);
    assert_eq!(schema.slots.len(), 24);
}

#[test]
fn every_dont_care_is_independent() {
    let schema = parse2(". ? . ? / . . . . / . . . . / . . . . / . . . . / . . . .").unwrap();
    assert!(schema.names.is_empty());
    assert!(schema.slots.iter().all(|slot| *slot == Slot::Any));
}

#[test]
fn fixed_colors_go_through_the_scheme() {
    let western: ColourScheme = "western".parse().unwrap();
    let schema = parse(
        "W W W W / R R R R / G G G G / Y Y Y Y / O O O O / B B B B",
        PuzzleKind::Cube2,
        &western,
    )
    .unwrap();
    assert_eq!(schema.slots[0], Slot::Fixed(Face::U));
    assert_eq!(schema.slots[4], Slot::Fixed(Face::R));
    assert_eq!(schema.slots[23], Slot::Fixed(Face::B));
    // Z is not in any built-in scheme.
    let text = "Z Z Z Z / a a a a / a a a a / a a a a / a a a a / a a a a";
    assert_eq!(
        parse(text, PuzzleKind::Cube2, &western)
            .unwrap_err()
            .to_string(),
        "line 1, column 1: unknown color 'Z', the scheme has WRGYOB"
    );
}

#[test]
fn custom_scheme_letters_are_fixed_colors() {
    let custom: ColourScheme = "ABCDEF".parse().unwrap();
    let text = "A A A A / B B B B / C C C C / D D D D / E E E E / F F F F";
    let schema = parse(text, PuzzleKind::Cube2, &custom).unwrap();
    assert_eq!(schema.slots[20], Slot::Fixed(Face::B));
}

#[test]
fn sticker_count_errors_name_the_face_and_position() {
    assert_eq!(
        error2("a a a / b b b b / c c c c / d d d d / e e e e / f f f f"),
        "line 1, column 7: face 1 has 3 stickers, expected 4 (2x2)"
    );
    assert_eq!(
        error2("a a a a a / b"),
        "line 1, column 9: face 1 has more than 4 stickers (2x2)"
    );
    assert_eq!(
        error2("a a a a / b b b b\nc c c c"),
        "line 2, column 8: expected 6 faces, found 3"
    );
    assert_eq!(
        error2("a a a a / b b b b / c c c c / d d d d / e e e e / f f f f / g g g g"),
        "line 1, column 61: a cube has 6 faces, the grid has more"
    );
    assert_eq!(error2(""), "line 1, column 1: expected 6 faces, found 0");
}

#[test]
fn bad_tokens_are_reported() {
    assert_eq!(
        error2("a a a a / b b b # "),
        "line 1, column 17: unexpected character '#', expected a lowercase letter, an uppercase letter, \".\" or \"?\""
    );
    assert_eq!(
        error2("a a a a / // b"),
        "line 1, column 11: empty face before \"/\""
    );
    assert_eq!(
        error2("aA a a a"),
        "line 1, column 2: expected a separator after the token, found 'A'"
    );
    assert_eq!(
        error2("U1 a a a"),
        "line 1, column 2: expected a separator after the token, found '1'"
    );
}

#[test]
fn other_sizes_check_n_times_n() {
    let face = |n: usize| vec!["a"; n * n].join(" ");
    for kind in PuzzleKind::ALL {
        let n = kind.size();
        let text = vec![face(n); 6].join(" / ");
        let schema = parse(&text, kind, &scheme()).unwrap();
        assert_eq!(schema.slots.len(), kind.facelet_count());
        let short = vec![face(n - 1); 6].join(" / ");
        assert!(parse(&short, kind, &scheme()).is_err());
    }
}
