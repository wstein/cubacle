//! `solve --moves SET`: the solution uses only the moves of the set.

use std::process::Command;

use cubacle::notation::parse_alphabet;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};

struct Run {
    ok: bool,
    out: String,
    err: String,
}

fn cubacle(args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

/// Whether `solution` solves the cube `scramble` makes and uses only `set`.
fn solves_within(scramble: &str, solution: &str, set: &str) -> bool {
    let kind = PuzzleKind::Cube3;
    let start = Facelets::solved(kind)
        .apply(&scramble.parse::<MoveSequence>().unwrap())
        .unwrap();
    let moves: MoveSequence = solution.parse().unwrap();
    let alphabet = parse_alphabet(set, kind).unwrap();
    moves.moves().iter().all(|mv| alphabet.contains(mv)) && start.apply(&moves).unwrap().is_solved()
}

#[test]
fn half_turns_solve_a_half_turn_cube_with_half_turns() {
    let scramble = "R2 U2 F2 D2 L2 B2 U2";
    let run = cubacle(&["solve", "--moves", "half-turns", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    assert!(
        solves_within(scramble, &run.out, "half-turns"),
        "{}",
        run.out
    );
    assert!(run.err.contains("moves "), "{}", run.err);
}

#[test]
fn two_generators_solve_with_two_generators() {
    let scramble = "R U R' U' R U2 R' U R U' R U R' U2 R";
    let run = cubacle(&["solve", "--moves", "<R,U>", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    assert!(solves_within(scramble, &run.out, "<R,U>"), "{}", run.out);
    assert!(run.out.split_whitespace().count() <= 15);
}

#[test]
fn a_list_is_taken_literally() {
    // Only R, U and U2 exist: R' has to be three R.
    let scramble = "R' U";
    let run = cubacle(&["solve", "--moves", "R U U2", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    assert!(solves_within(scramble, &run.out, "R U U2"), "{}", run.out);
}

#[test]
fn qtm_answers_in_quarter_turns() {
    let scramble = "R2 U F2 D' L B2 R'";
    let run = cubacle(&["solve", "--moves", "qtm", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    assert!(solves_within(scramble, &run.out, "qtm"), "{}", run.out);
    assert!(!run.out.contains('2'), "{}", run.out);
}

#[test]
fn htm_is_the_default_and_changes_nothing() {
    let scramble = "R U F' D2 L B R2 U' F";
    let plain = cubacle(&["solve", "--scramble", scramble]);
    let named = cubacle(&["solve", "--moves", "htm", "--scramble", scramble]);
    assert!(plain.ok && named.ok);
    assert_eq!(plain.out, named.out);
    assert_eq!(plain.err, named.err);
}

#[test]
fn a_cube_outside_the_subgroup_is_reported_at_once() {
    for (set, scramble) in [("half-turns", "R"), ("<R,U>", "F"), ("quarter-slices", "R")] {
        let run = cubacle(&["solve", "--moves", set, "--scramble", scramble]);
        assert!(!run.ok, "{set}");
        assert!(
            run.err.contains(&format!("not solvable with {set}")),
            "{set}: {}",
            run.err
        );
    }
}

#[test]
fn the_move_limit_and_the_budget_still_apply() {
    let scramble = "R U R' U' R U2 R' U R U' R U R' U2";
    let run = cubacle(&[
        "solve",
        "--moves",
        "<R,U>",
        "-m",
        "3",
        "--scramble",
        scramble,
    ]);
    assert!(!run.ok);
    assert!(run.err.contains("at most 3 moves"), "{}", run.err);
    let run = cubacle(&[
        "solve",
        "--moves",
        "<R,U>",
        "-b",
        "0",
        "--scramble",
        scramble,
    ]);
    assert!(!run.ok);
    assert!(run.err.contains("time budget"), "{}", run.err);
}

#[test]
fn bad_sets_and_conflicting_options_are_errors() {
    let run = cubacle(&["solve", "--moves", "R X9", "--scramble", "R"]);
    assert!(!run.ok);
    assert!(run.err.contains("--moves"), "{}", run.err);
    let run = cubacle(&["solve", "--moves", "<R,U>", "--improve", "--scramble", "R"]);
    assert!(!run.ok);
    let run = cubacle(&["solve", "--moves", "<>", "--scramble", "R"]);
    assert!(!run.ok);
}

fn big(set: &str, scramble: &str) -> Run {
    cubacle(&[
        "solve",
        "--size",
        "5",
        "--moves",
        set,
        "--scramble",
        scramble,
    ])
}

#[test]
fn a_big_cube_is_solved_by_pattern_macros_of_the_set() {
    let scramble = "R2 L2 U2 D2 F2 B2";
    let run = big("half-turns", scramble);
    assert!(run.ok, "{}", run.err);
    let kind = PuzzleKind::Cube5;
    let start = Facelets::solved(kind)
        .apply(&scramble.parse::<MoveSequence>().unwrap())
        .unwrap();
    let moves: MoveSequence = run.out.parse().unwrap();
    let alphabet = parse_alphabet("half-turns", kind).unwrap();
    assert!(moves.moves().iter().all(|mv| alphabet.contains(mv)));
    assert!(start.apply(&moves).unwrap().is_solved());
}

#[test]
fn a_big_cube_set_without_a_macro_is_an_error_not_a_hang() {
    let run = big("R", "R2 L2 U2 D2 F2 B2");
    assert!(!run.ok);
    assert!(run.err.contains("cannot be searched"), "{}", run.err);
}

#[test]
fn a_cube_that_is_no_product_of_the_macros_is_not_called_unsolvable() {
    let run = big("half-turns", "R2 U2 3Rw2 Uw2 F2");
    assert!(!run.ok);
    assert!(
        run.err.contains("does not mean the cube cannot be solved"),
        "{}",
        run.err
    );
}

#[test]
fn patterns_find_keeps_to_the_moves_given() {
    let run = cubacle(&[
        "patterns",
        "find",
        "--moves",
        "half-turns",
        "--scramble",
        "R2 L2 U2 D2 F2 B2",
    ]);
    assert!(run.ok, "{}", run.err);
    assert!(!run.out.contains('\''), "{}", run.out);
    assert_eq!(run.out.split_whitespace().count(), 6);
}
