use super::*;
use crate::puzzle::Facelets;

fn expand(text: &str, kind: PuzzleKind) -> Result<String, ExprError> {
    parse_expression(text, kind).map(|moves| moves.to_string())
}

#[test]
fn plain_moves_read_as_before() {
    assert_eq!(expand("R U R' U'", PuzzleKind::Cube3).unwrap(), "R U R' U'");
    assert_eq!(expand("", PuzzleKind::Cube3).unwrap(), "");
}

#[test]
fn a_group_repeats() {
    assert_eq!(expand("(R U)*3", PuzzleKind::Cube3).unwrap(), "R U R U R U");
    assert_eq!(expand("R*2 U", PuzzleKind::Cube3).unwrap(), "R R U");
    assert_eq!(expand("(R U)*1", PuzzleKind::Cube3).unwrap(), "R U");
}

#[test]
fn groups_nest_and_need_no_spaces() {
    assert_eq!(
        expand("((R U')*2 F)*2 D", PuzzleKind::Cube3).unwrap(),
        "R U' R U' F R U' R U' F D"
    );
    assert_eq!(
        expand("(R(U F)*2)*2", PuzzleKind::Cube3).unwrap(),
        "R U F U F R U F U F"
    );
}

#[test]
fn six_sexy_moves_are_the_identity() {
    let moves = parse_expression("(R U R' U')*6", PuzzleKind::Cube3).unwrap();
    let cube = crate::puzzle::Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves)
        .unwrap();
    assert!(cube.is_solved());
}

#[test]
fn malformed_expressions_are_rejected() {
    for text in [
        "(R U", "R U)", "((R)", "*2", "R*", "R*0", "(R)*x", "R *2", "R**2",
    ] {
        assert!(expand(text, PuzzleKind::Cube3).is_err(), "{text:?}");
    }
    assert_eq!(
        expand("(R U", PuzzleKind::Cube3),
        Err(ExprError::Unbalanced { position: 0 })
    );
}

#[test]
fn moves_must_fit_the_cube() {
    assert!(matches!(
        expand("(3R)*2", PuzzleKind::Cube3),
        Err(ExprError::DoesNotFit { .. })
    ));
    assert!(expand("(3R)*2", PuzzleKind::Cube4).is_ok());
}

#[test]
fn runaway_repeats_are_refused() {
    assert_eq!(
        expand("((R)*1000)*1000", PuzzleKind::Cube3),
        Err(ExprError::TooLong)
    );
}

fn cube(text: &str, kind: PuzzleKind) -> Facelets {
    let moves = parse_expression(text, kind).unwrap_or_else(|e| panic!("{text}: {e}"));
    Facelets::solved(kind).apply(&moves).unwrap()
}

fn sticker(cube: &Facelets, face: Face, index: usize) -> char {
    cube.to_string()
        .chars()
        .nth(face.index() * 9 + index)
        .unwrap()
}

#[test]
fn middle_slices_on_a_3x3_are_the_inner_layer_of_l_d_f() {
    let kind = PuzzleKind::Cube3;
    for (slice, layer) in [("M", "2L"), ("E", "2D"), ("S", "2F")] {
        for suffix in ["", "'", "2"] {
            assert_eq!(
                cube(&format!("{slice}{suffix}"), kind),
                cube(&format!("{layer}{suffix}"), kind),
                "{slice}{suffix}"
            );
        }
    }
}

#[test]
fn slices_follow_l_d_and_f() {
    let kind = PuzzleKind::Cube3;
    // L carries the top to the front, D the left to the front, F the top to the right.
    assert_eq!(sticker(&cube("M", kind), Face::F, 4), 'U');
    assert_eq!(sticker(&cube("E", kind), Face::F, 4), 'L');
    assert_eq!(sticker(&cube("S", kind), Face::R, 4), 'U');
}

#[test]
fn rotations_follow_r_u_and_f() {
    let kind = PuzzleKind::Cube3;
    assert_eq!(sticker(&cube("x", kind), Face::U, 4), 'F');
    assert_eq!(sticker(&cube("y", kind), Face::L, 4), 'F');
    assert_eq!(sticker(&cube("z", kind), Face::R, 4), 'U');
}

#[test]
fn rotations_are_a_face_and_slices() {
    let kind = PuzzleKind::Cube3;
    assert_eq!(cube("x", kind), cube("R M' L'", kind));
    assert_eq!(cube("y", kind), cube("U E' D'", kind));
    assert_eq!(cube("z", kind), cube("F S B'", kind));
    for size in [
        PuzzleKind::Cube2,
        kind,
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
    ] {
        assert_eq!(cube("x2", size), cube("x x", size));
        assert_eq!(cube("y'", size), cube("y y y", size));
        assert_eq!(cube("z2'", size), cube("z2", size));
    }
}

#[test]
fn a_rotation_and_its_inverse_cancel_on_every_size() {
    for kind in [
        PuzzleKind::Cube2,
        PuzzleKind::Cube3,
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
        PuzzleKind::Cube6,
        PuzzleKind::Cube7,
    ] {
        assert!(cube("x y z z' y' x'", kind).is_solved(), "{kind:?}");
        assert!(cube("(x y z)*4 (z' y' x')*4", kind).is_solved());
        assert!(cube("x*4 y2*2 z'*4", kind).is_solved());
    }
}

#[test]
fn slice_turns_cancel_on_odd_cubes() {
    for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
        assert!(cube("M*4 E' E S2 S2", kind).is_solved(), "{kind:?}");
    }
    // The middle layer of a 7x7x7 is its fourth.
    assert_eq!(cube("M", PuzzleKind::Cube7), cube("4L", PuzzleKind::Cube7));
}

#[test]
fn even_cubes_refuse_slices() {
    for kind in [PuzzleKind::Cube2, PuzzleKind::Cube4, PuzzleKind::Cube6] {
        for word in ["M", "E'", "S2"] {
            assert!(matches!(
                expand(word, kind),
                Err(ExprError::NoMiddleLayer { .. })
            ));
        }
    }
}

#[test]
fn bad_slice_and_rotation_words_are_rejected() {
    for text in ["Mw", "M3'", "xw", "2M", "x4", "M''", "m", "X"] {
        assert!(expand(text, PuzzleKind::Cube5).is_err(), "{text:?}");
    }
}

#[test]
fn a_nested_expression_with_slices_and_rotations_is_a_valid_cube() {
    let text = "((M' U)*4 y' z')*3";
    let kind = PuzzleKind::Cube7;
    let moves = parse_expression(text, kind).unwrap();
    // Each rotation is two turns: 3 * (4 * 2 + 2 * 2) moves.
    assert_eq!(moves.len(), 3 * (4 * 2 + 2 * 2));
    let state = cube(text, kind);
    assert!(!state.is_solved());
    let token = crate::notation::orbit64::encode(&state).unwrap();
    assert_eq!(crate::notation::orbit64::decode(&token).unwrap(), state);
    assert_eq!(
        crate::notation::parse_cube(&state.to_string(), None).unwrap(),
        state
    );
}

const ALL_KINDS: [PuzzleKind; 6] = [
    PuzzleKind::Cube2,
    PuzzleKind::Cube3,
    PuzzleKind::Cube4,
    PuzzleKind::Cube5,
    PuzzleKind::Cube6,
    PuzzleKind::Cube7,
];

#[test]
fn a_rotation_leaves_every_face_one_color() {
    for kind in ALL_KINDS {
        let face_len = kind.face_len();
        for word in ["x", "y", "z'", "x2", "y'", "z"] {
            let text = cube(word, kind).to_string();
            for face in text.as_bytes().chunks(face_len) {
                assert!(face.iter().all(|&c| c == face[0]), "{word} on {kind:?}");
            }
        }
    }
}

#[test]
fn conjugating_by_a_rotation_relabels_the_face() {
    for kind in ALL_KINDS {
        for (conjugated, plain) in [
            ("x U x'", "F"),
            ("x F x'", "D"),
            ("y F y'", "R"),
            ("z U z'", "L"),
        ] {
            assert_eq!(
                cube(conjugated, kind),
                cube(plain, kind),
                "{conjugated} {kind:?}"
            );
        }
    }
    for kind in &ALL_KINDS[1..] {
        assert_eq!(cube("y 2F y'", *kind), cube("2R", *kind), "{kind:?}");
    }
    for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
        assert_eq!(cube("y M y'", kind), cube("S", kind), "{kind:?}");
    }
}

#[test]
fn nesting_is_limited_instead_of_overflowing_the_stack() {
    let deep = format!("{}R{}", "(".repeat(40_000), ")".repeat(40_000));
    assert_eq!(expand(&deep, PuzzleKind::Cube3), Err(ExprError::TooDeep));
    let ok = format!("{}R{}", "(".repeat(MAX_DEPTH), ")".repeat(MAX_DEPTH));
    assert_eq!(expand(&ok, PuzzleKind::Cube3).unwrap(), "R");
}

#[test]
fn a_count_too_big_for_a_number_is_too_long() {
    assert_eq!(
        expand("R*99999999999999999999", PuzzleKind::Cube3),
        Err(ExprError::TooLong)
    );
    assert_eq!(
        expand("R*0", PuzzleKind::Cube3),
        Err(ExprError::BadRepeat { position: 1 })
    );
}

#[test]
fn an_unclosed_group_is_reported_where_it_opens() {
    assert_eq!(
        expand("F (R (U", PuzzleKind::Cube3),
        Err(ExprError::Unbalanced { position: 5 })
    );
}

#[test]
fn moves_written_together_are_cut_at_unambiguous_places() {
    let three = PuzzleKind::Cube3;
    let five = PuzzleKind::Cube5;
    assert_eq!(expand("R'U'", three).unwrap(), "R' U'");
    assert_eq!(
        expand("R U'F", three).unwrap(),
        expand("R U' F", three).unwrap()
    );
    assert_eq!(expand("NBL", five).unwrap(), expand("NB L", five).unwrap());
    assert_eq!(
        expand("TL'NR", five).unwrap(),
        expand("TL' NR", five).unwrap()
    );
    assert_eq!(
        expand("F2NR'", five).unwrap(),
        expand("F2 NR'", five).unwrap()
    );
    assert_eq!(expand("(R'U')2", three).unwrap(), "R' U' R' U'");
}

#[test]
fn ambiguous_or_unlikely_words_stay_errors() {
    let three = PuzzleKind::Cube3;
    // `F2R` could be `F2 R` or `F 2R`; `RU` and `R22` have no cut at all.
    for word in ["F2R", "RU", "R22", "R'U'Q", "R-U"] {
        assert!(parse_expression(word, three).is_err(), "{word}");
    }
}
