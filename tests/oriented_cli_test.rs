//! A cube whose face centers a slice turn moved is solved as the cube named
//! after its centers: the printed moves leave every face one color.

use std::process::Command;

use cubacle::notation::parse_expression;
use cubacle::puzzle::{oriented_by_centers, solves_by_color, Facelets, PuzzleKind};

struct Run {
    code: Option<i32>,
    out: String,
    err: String,
}

fn solve(size: &str, scramble: &str) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", "--size", size, "--scramble", scramble])
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/oriented-tables"),
        )
        .output()
        .expect("cubacle runs");
    Run {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

fn kind_of(size: &str) -> PuzzleKind {
    match size {
        "3" => PuzzleKind::Cube3,
        "5" => PuzzleKind::Cube5,
        _ => PuzzleKind::Cube7,
    }
}

/// Solves, checks the answer on the cube as given, and returns the run.
fn assert_solved(size: &str, scramble: &str) -> Run {
    let run = solve(size, scramble);
    assert_eq!(run.code, Some(0), "{size} {scramble}: {}", run.err);
    let kind = kind_of(size);
    let moves = parse_expression(&run.out, kind).expect("moves");
    let start = Facelets::solved(kind)
        .apply(&parse_expression(scramble, kind).unwrap())
        .unwrap();
    assert!(solves_by_color(&start, &moves), "{size} {scramble}");
    run
}

#[test]
fn slice_turns_of_the_3x3x3_are_solved() {
    for scramble in ["2R", "R U 2R2 F 2U D'", "2R 2U 2F", "2F' R2 U 2L D2 B 2U'"] {
        let run = assert_solved("3", scramble);
        assert!(run.err.contains("note: the face centers"), "{}", run.err);
    }
}

#[test]
fn slice_turns_of_the_5x5x5_are_solved() {
    for scramble in [
        "3R",
        "R U 3R2 F 2L 3U Rw",
        // the generator of the pattern centres-cycled
        "2-4Uw 2-4Rw' 2-4Uw' 2-4Rw",
    ] {
        let run = assert_solved("5", scramble);
        assert!(run.err.contains("note: the face centers"), "{}", run.err);
    }
}

#[test]
fn slice_turns_of_the_7x7x7_are_solved() {
    for scramble in ["2-6Rw", "4R 3U' 4F2 Rw"] {
        let run = assert_solved("7", scramble);
        assert!(run.err.contains("note: the face centers"), "{}", run.err);
    }
}

#[test]
fn a_plain_scramble_is_solved_as_before_without_a_note() {
    for (size, scramble) in [
        ("3", "R U F2 L' D B2"),
        ("5", "R U 2F2 Rw' L D"),
        ("7", "R U 3F2 L'"),
    ] {
        let run = solve(size, scramble);
        assert_eq!(run.code, Some(0), "{size}: {}", run.err);
        assert!(!run.err.contains("note:"), "{size}: {}", run.err);
        let kind = kind_of(size);
        let moves = parse_expression(&run.out, kind).unwrap();
        let start = Facelets::solved(kind)
            .apply(&parse_expression(scramble, kind).unwrap())
            .unwrap();
        assert!(start.apply(&moves).unwrap().is_solved(), "{size}");
    }
}

#[test]
fn the_even_cubes_never_needed_a_renaming() {
    for (size, scramble) in [("4", "Rw Lw' U R"), ("6", "3Rw 3Lw' U R")] {
        let run = solve(size, scramble);
        assert_eq!(run.code, Some(0), "{size}: {}", run.err);
        assert!(!run.err.contains("note:"), "{size}: {}", run.err);
    }
}

#[test]
fn swapped_centers_are_still_an_impossible_cube() {
    let mut stickers = Facelets::solved(PuzzleKind::Cube3).stickers().to_vec();
    stickers.swap(4, 9 + 4);
    let cube = Facelets::from_stickers(stickers).unwrap();
    assert_eq!(oriented_by_centers(&cube), None);
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", &cube.to_string()])
        .output()
        .expect("cubacle runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("impossible"), "{err}");
}

#[test]
fn a_target_with_moved_centers_still_says_what_is_wrong() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", "--scramble", "2R", "--to", "superflip"])
        .output()
        .expect("cubacle runs");
    assert_ne!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
}
