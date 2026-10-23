//! Group suffixes, commutators, conjugates and layer ranges inside expressions.

use super::*;
use crate::notation::Notation;
use crate::puzzle::Facelets;

const SIZES: [PuzzleKind; 6] = [
    PuzzleKind::Cube2,
    PuzzleKind::Cube3,
    PuzzleKind::Cube4,
    PuzzleKind::Cube5,
    PuzzleKind::Cube6,
    PuzzleKind::Cube7,
];

fn expand(text: &str, kind: PuzzleKind) -> Result<String, ExprError> {
    parse_expression(text, kind).map(|moves| moves.to_string())
}

fn ok(text: &str) -> String {
    expand(text, PuzzleKind::Cube7).unwrap_or_else(|e| panic!("{text}: {e}"))
}

#[test]
fn a_group_takes_a_bare_count_a_star_and_an_inverse() {
    assert_eq!(ok("(R U)2"), "R U R U");
    assert_eq!(ok("(R U)*2"), "R U R U");
    assert_eq!(ok("(R U)'"), "U' R'");
    assert_eq!(ok("(R U2)'"), "U2 R'");
    assert_eq!(ok("(R U)2'"), "U' R' U' R'");
    assert_eq!(ok("(R U)'2"), "U' R' U' R'");
    assert_eq!(ok("(R U)*2'"), "U' R' U' R'");
    assert_eq!(ok("(R U)1"), "R U");
    assert_eq!(ok("((R U)2 F)'"), "F' U' R' U' R'");
    // a move keeps its own suffix: R2 is a half turn, not two repeats
    assert_eq!(ok("R2 R*2"), "R2 R R");
    assert_eq!(ok("(R)2 U"), "R R U");
}

#[test]
fn commutators_and_conjugates_expand() {
    assert_eq!(ok("[R, U]"), "R U R' U'");
    assert_eq!(ok("[R,U]"), "R U R' U'");
    assert_eq!(ok("[R: U]"), "R U R'");
    assert_eq!(ok("[R U: F]"), "R U F U' R'");
    assert_eq!(ok("[R U, F]"), "R U F U' R' F'");
    assert_eq!(ok("[R: [U, F]]"), "R U F U' F' R'");
    assert_eq!(ok("[[R, U]: F2]"), "R U R' U' F2 U R U' R'");
    assert_eq!(ok("[2-3r, U]"), "3Rw R' U R 3Rw' U'");
    assert_eq!(ok("[R, U]2"), ok("(R U R' U')2"));
    assert_eq!(ok("[R, U]'"), "U R U' R'");
    assert_eq!(ok("[R, U]*3"), ok("[R, U] [R, U] [R, U]"));
}

#[test]
fn a_commutator_and_its_swap_cancel_on_every_size() {
    for kind in SIZES {
        let cube = |text: &str| {
            let moves = parse_expression(text, kind).unwrap();
            Facelets::solved(kind).apply(&moves).unwrap()
        };
        assert!(cube("[R, U] [U, R]").is_solved(), "{kind:?}");
        assert!(cube("[F: [R, U]] [F: [R, U]]'").is_solved());
        assert!(cube("[R U: F2] [R U: F2]").is_solved());
        if kind.size() >= 4 {
            assert!(cube("[2-3r: U2]2").is_solved(), "{kind:?}");
        }
        // a conjugate by a rotation relabels the moves it conjugates
        assert_eq!(cube("[x: U]"), cube("F"), "{kind:?}");
        assert_eq!(cube("[y: F]"), cube("R"), "{kind:?}");
    }
}

#[test]
fn expressions_round_trip_through_both_notations_on_every_size() {
    let text = "[F: [R, U]] (2-3r U')2 [L2: 2-2d] (u f')' x [y: R2] 2-3Lw";
    for kind in SIZES {
        let moves = match parse_expression(text, kind) {
            Ok(moves) => moves,
            // the small cubes lack layers or middle slices of the long text
            Err(_) => continue,
        };
        for notation in [Notation::Wca, Notation::Sign] {
            let written = notation.write(&moves);
            let again = parse_expression(&written, kind).unwrap();
            assert_eq!(again, moves, "{notation:?} on {kind:?}: {written}");
        }
    }
    let seven = parse_expression(text, PuzzleKind::Cube7).unwrap();
    assert!(!seven.is_empty());
    // smaller sizes: the same shapes with fewer layers
    for kind in SIZES {
        let short = "[F: [R, U]] (R U')2 [L2: D] (U F')' x [y: R2]";
        let moves = parse_expression(short, kind).unwrap();
        let again = parse_expression(&Notation::Sign.write(&moves), kind).unwrap();
        assert_eq!(again, moves, "{kind:?}");
    }
}

#[test]
fn ranges_work_inside_groups_and_brackets() {
    assert_eq!(ok("(2-3r U)2"), "3Rw R' U 3Rw R' U");
    assert_eq!(ok("(2-3r)'"), "R 3Rw'");
    assert_eq!(ok("[2-3Rw2: U]"), "3Rw2 R2 U R2 3Rw2");
    // `*N` repeats the whole range word
    assert_eq!(ok("2-3r*2"), "3Rw R' 3Rw R'");
    assert_eq!(
        expand("2-3r", PuzzleKind::Cube3),
        Err(ExprError::DoesNotFit {
            position: 0,
            mv: "3Rw".parse().unwrap(),
            kind: PuzzleKind::Cube3
        })
    );
}

fn syntax(text: &str) -> Option<usize> {
    match expand(text, PuzzleKind::Cube5) {
        Err(ExprError::Syntax { position, .. }) => Some(position),
        _ => None,
    }
}

#[test]
fn bracket_mistakes_are_reported_where_they_are() {
    assert_eq!(syntax("[R U]"), Some(0));
    assert_eq!(syntax("[R, U, F]"), Some(5));
    assert_eq!(syntax("[R: U, F]"), Some(5));
    assert_eq!(syntax("[, U]"), Some(1));
    assert_eq!(syntax("[R: ]"), Some(2));
    assert_eq!(syntax("[]"), Some(0));
    assert_eq!(syntax("R, U"), Some(1));
    assert_eq!(syntax("R: U"), Some(1));
    assert_eq!(syntax("(R, U)"), Some(2));
    assert_eq!(syntax("[(R, U), F]"), Some(3));
    assert_eq!(syntax("(R U)''"), Some(6));
    assert_eq!(syntax("(R U)'2'"), Some(7));
    assert_eq!(syntax("[R, U]''"), Some(7));
}

#[test]
fn unbalanced_brackets_name_the_bracket() {
    for (text, position) in [
        ("[R, U", 0),
        ("[R: [U, F]", 0),
        ("F [R, (U", 6),
        ("(R]", 2),
        ("[R, U)", 5),
        ("R U]", 3),
        ("R U)", 3),
        ("([R, U)]", 6),
    ] {
        assert_eq!(
            expand(text, PuzzleKind::Cube5),
            Err(ExprError::Unbalanced { position }),
            "{text}"
        );
    }
}

#[test]
fn bad_words_and_repeats_carry_their_position() {
    assert!(matches!(
        expand("R Q", PuzzleKind::Cube3),
        Err(ExprError::Move { position: 2, .. })
    ));
    assert!(matches!(
        expand("(R U) [F, rw]", PuzzleKind::Cube5),
        Err(ExprError::Sign { position: 10, .. })
    ));
    assert!(matches!(
        expand("U 3-2r", PuzzleKind::Cube5),
        Err(ExprError::Sign { position: 2, .. })
    ));
    assert_eq!(
        expand("(R)0", PuzzleKind::Cube3),
        Err(ExprError::BadRepeat { position: 3 })
    );
    assert_eq!(
        expand("(R)*", PuzzleKind::Cube3),
        Err(ExprError::BadRepeat { position: 3 })
    );
    let message = expand("U [F, Q]", PuzzleKind::Cube3)
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("character 6") && message.contains("\"Q\""),
        "{message}"
    );
}

#[test]
fn the_move_limit_holds_for_every_construct() {
    let kind = PuzzleKind::Cube3;
    assert_eq!(expand("(R U)*50000", kind).map(|s| !s.is_empty()), Ok(true));
    assert_eq!(expand("(R U)*50001", kind), Err(ExprError::TooLong));
    assert_eq!(expand("(R U)50001", kind), Err(ExprError::TooLong));
    assert_eq!(expand("[(R U)*25001, F]", kind), Err(ExprError::TooLong));
    assert_eq!(expand("[(R U)*25001: F]", kind), Err(ExprError::TooLong));
    assert_eq!(expand("[R, U]*25001", kind), Err(ExprError::TooLong));
    assert_eq!(
        expand("[R, U]99999999999999999999", kind),
        Err(ExprError::TooLong)
    );
    // doubling brackets stay bounded
    let mut text = String::from("R");
    for _ in 0..40 {
        text = format!("[{text}, U]");
    }
    assert_eq!(expand(&text, kind), Err(ExprError::TooLong));
}

#[test]
fn bracket_nesting_is_limited_too() {
    let deep = format!("{}R{}", "[R: ".repeat(40_000), "]".repeat(40_000));
    assert_eq!(expand(&deep, PuzzleKind::Cube3), Err(ExprError::TooDeep));
    let mixed = format!("{}R{}", "([R: ".repeat(60), "])".repeat(60));
    assert_eq!(expand(&mixed, PuzzleKind::Cube3), Err(ExprError::TooDeep));
}
