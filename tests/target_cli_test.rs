//! `solve --to`: the printed moves take the start to the target.

use cubacle::notation::parse_expression;
use cubacle::puzzle::{reaches, Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never the user's real table cache
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

/// Runs `solve` from `start` to `target` and checks by replay that the answer
/// takes one to the other; returns the number of moves.
fn solves_to(size: &str, start: &str, to: &str, target: &Facelets) -> usize {
    let (ok, solution, error) = cubacle(&["solve", "--size", size, start, "--to", to]);
    assert!(ok, "{start} -> {to}: {error}");
    let start = cube_of(size, start);
    let moves: MoveSequence = solution.parse().expect("the answer is move notation");
    assert!(reaches(&start, &moves, target), "{solution}");
    moves.len()
}

#[test]
fn random_2x2_states_are_reached_up_to_a_rotation_of_the_whole_cube() {
    for _ in 0..6 {
        let (_, start, _) = cubacle(&["random", "--size", "2"]);
        let (_, target, _) = cubacle(&["random", "--size", "2"]);
        let (ok, solution, error) = cubacle(&["solve", &start, "--to", &target]);
        assert!(ok, "{start} -> {target}: {error}");
        let moves: MoveSequence = solution.parse().unwrap();
        let (start, target) = (
            Facelets::parse(&start).unwrap(),
            Facelets::parse(&target).unwrap(),
        );
        assert!(reaches(&start, &moves, &target), "{solution}");
    }
}

fn cube_of(size: &str, moves: &str) -> Facelets {
    let kind = PuzzleKind::of_size(size.parse().unwrap()).unwrap();
    cube(kind, moves)
}

#[test]
fn a_move_expression_target_is_reached_on_every_small_size() {
    for (size, start, to) in [
        ("2", "R U F2 R'", "U R2 F'"),
        ("3", "R U R' U' F2 D", "L2 B' U F D2"),
        ("3", "(R U)*3", "(F R U)*2 L"),
        ("4", "R U Rw Fw", "U2 R' L"),
        ("4", "Rw U2 Fw", "D L2 B"),
    ] {
        solves_to(size, start, to, &cube_of(size, to));
    }
}

#[test]
fn a_5x5_target_is_reached() {
    solves_to(
        "5",
        "R U F2 R' B L2",
        "U2 D' L F",
        &cube_of("5", "U2 D' L F"),
    );
}

#[test]
fn token_facelets_and_colour_targets_agree_with_the_moves() {
    let moves = "L2 B' U F D2 R";
    let target = cube(PuzzleKind::Cube3, moves);
    let (_, token, _) = cubacle(&["convert", "--to", "token", "--scramble", moves]);
    let (_, colours, _) = cubacle(&["convert", "--to", "color", "--scramble", moves]);
    let facelets = target.to_string();
    let plain = cubacle(&["solve", "R U R' U' F2 D", "--to", moves]);
    assert!(plain.0, "{}", plain.2);
    for to in [&token, &facelets, &colours] {
        let got = cubacle(&["solve", "R U R' U' F2 D", "--to", to]);
        assert!(got.0, "{to}: {}", got.2);
        assert_eq!(got.1, plain.1, "{to}");
    }
    solves_to("3", "R U R' U' F2 D", &token, &target);
}

#[test]
fn a_target_equal_to_the_start_needs_no_moves() {
    let (ok, solution, error) = cubacle(&["solve", "R U F", "--to", "R U F"]);
    assert!(ok, "{error}");
    assert_eq!(solution, "");
    assert!(error.contains("moves 0"), "{error}");
}

#[test]
fn a_target_one_turn_from_solved_is_reached() {
    solves_to("3", "R U F2 D' L", "U", &cube(PuzzleKind::Cube3, "U"));
}

#[test]
fn start_and_target_must_have_one_size() {
    let (_, token, _) = cubacle(&["convert", "--size", "2", "--scramble", "R U"]);
    let (ok, _, error) = cubacle(&["solve", "R U F", "--to", &token]);
    assert!(!ok);
    assert!(
        error.contains("3x3x3") && error.contains("2x2x2"),
        "{error}"
    );
}

#[test]
fn an_impossible_target_is_rejected() {
    // Two stickers of different faces swapped: every colour count is right,
    // no cube looks like it.
    let mut stickers: Vec<char> = Facelets::solved(PuzzleKind::Cube3)
        .to_string()
        .chars()
        .collect();
    stickers.swap(8, 9);
    let impossible: String = stickers.into_iter().collect();
    let (ok, _, error) = cubacle(&["solve", "R U F", "--to", &impossible]);
    assert!(!ok);
    assert!(error.contains("impossible"), "{error}");
    assert!(!error.contains("internal"), "{error}");
}

#[test]
fn a_target_with_the_centres_moved_is_reached_on_a_4x4() {
    // refused before the centre fix-up; see centre_fixup_cli_test.rs
    let target = cube(PuzzleKind::Cube4, "Rw");
    let size = "4";
    solves_to(size, "R U", "Rw", &target);
}

#[test]
fn named_targets_are_reached() {
    for (name, moves) in [
        (
            "superflip",
            "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
        ),
        ("checkerboard", "U2 D2 R2 L2 F2 B2"),
        ("cube-in-cube", "F L F U' R U F2 L2 U' L' B D' B' L2 U"),
    ] {
        solves_to("3", "R U F2 D' L", name, &cube(PuzzleKind::Cube3, moves));
    }
}

#[test]
fn the_names_are_refused_on_other_sizes() {
    for name in ["superflip", "checkerboard", "cube-in-cube"] {
        let (ok, _, error) = cubacle(&["solve", "--size", "4", "R U", "--to", name]);
        assert!(!ok);
        assert!(
            error.contains("3x3x3") && error.contains("4x4x4"),
            "{error}"
        );
    }
}

#[test]
fn a_name_inside_a_longer_text_is_no_name() {
    let (ok, _, error) = cubacle(&["solve", "R U", "--to", "superflip R"]);
    assert!(!ok);
    assert!(error.contains("not a cube"), "{error}");
}

#[test]
fn a_garbage_target_names_the_option() {
    let (ok, _, error) = cubacle(&["solve", "R U", "--to", "not a cube"]);
    assert!(!ok);
    assert!(error.contains("--to"), "{error}");
}
