//! `patterns find`: the shortest known generator of a cube state, and the
//! memory cap that ends a search.

use std::process::Command;

use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};

struct Run {
    ok: bool,
    out: String,
    err: String,
}

fn cubacle(args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never touch the real home: the first run writes a global config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

/// The state that `moves` build from the solved cube.
fn built(kind: PuzzleKind, moves: &str) -> Facelets {
    let sequence: MoveSequence = moves.parse().expect("moves");
    Facelets::solved(kind).apply(&sequence).expect("fits")
}

#[test]
fn a_3x3_state_gets_its_shortest_generator() {
    let run = cubacle(&["patterns", "find", "--scramble", "R U R' U'"]);
    assert!(run.ok, "{}", run.err);
    assert_eq!(run.out, "R U R' U'");
    assert!(run.err.contains("moves 4"), "{}", run.err);
    assert!(run.err.contains("shortest"), "{}", run.err);
}

#[test]
fn a_pattern_on_a_bigger_cube_is_built_by_the_generator_printed() {
    let scramble = "2R2 2L2 2U2 2D2 2F2 2B2";
    let run = cubacle(&["patterns", "find", "--size", "4", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    let kind = PuzzleKind::Cube4;
    assert_eq!(built(kind, &run.out), built(kind, scramble), "{}", run.out);
    let count = run.out.split_whitespace().count();
    assert!(count <= 6, "{count} moves: {}", run.out);
}

#[test]
fn a_state_as_facelets_is_read_like_solve_reads_it() {
    let state = built(PuzzleKind::Cube3, "R2 L2 U2 D2 F2 B2").to_string();
    let run = cubacle(&["patterns", "find", &state]);
    assert!(run.ok, "{}", run.err);
    assert!(run.err.contains("moves 6"), "{}", run.err);
    assert_eq!(
        built(PuzzleKind::Cube3, &run.out),
        built(PuzzleKind::Cube3, "R2 L2 U2 D2 F2 B2")
    );
}

#[test]
fn a_cap_that_is_reached_is_not_a_proof_of_nothing() {
    // A scramble of 18 moves is far beyond what 1 MB of states can reach.
    let run = cubacle(&[
        "patterns",
        "find",
        "--max-memory",
        "1",
        "--scramble",
        "R U2 D' B D' L2 D F2 U2 B2 D F' D R2 B U' F2 D' L2",
    ]);
    assert!(!run.ok);
    assert!(
        run.err.contains("nothing found within the cap of 1 MB"),
        "{}",
        run.err
    );
    assert!(run.err.contains("does not mean"), "{}", run.err);
}

#[test]
fn a_solved_cube_needs_no_moves() {
    let run = cubacle(&["patterns", "find", "--scramble", "R R'"]);
    assert!(run.ok, "{}", run.err);
    assert_eq!(run.out, "");
    assert!(run.err.contains("moves 0"), "{}", run.err);
}
