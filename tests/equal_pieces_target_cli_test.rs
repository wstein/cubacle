//! `solve --to` on the 4x4x4 and larger: equal-coloured centres are paired
//! fixed points first, so a target whose faces keep one colour per kind of
//! centre is reached even when its centres sit on other faces.

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

fn cube(size: usize, moves: &str) -> Facelets {
    let kind = PuzzleKind::of_size(size).unwrap();
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

/// Solves `start` to `to` and checks by replay; returns the number of moves.
fn reaches_target(size: usize, start: &str, to: &str) -> usize {
    let (ok, solution, error) = cubacle(&["solve", "--size", &size.to_string(), start, "--to", to]);
    assert!(ok, "{size}: {start} -> {to}: {error}");
    let moves: MoveSequence = solution.parse().expect("the answer is move notation");
    assert!(reaches(&cube(size, start), &moves, &cube(size, to)));
    moves.len()
}

#[test]
fn a_4x4_target_with_centres_on_other_faces_is_reached() {
    // refused with "the target has centres out of place" before the pairing
    reaches_target(4, "R U", "Rw2 Lw2");
    reaches_target(4, "R U", "Rw2 Lw2 Uw2 Dw2 Fw2 Bw2");
}

#[test]
fn a_5x5_target_with_centres_on_other_faces_is_reached() {
    reaches_target(5, "R U F2", "Rw2 Lw2 Uw2 Dw2 Fw2 Bw2");
}

#[test]
fn outer_targets_are_reached_on_the_6x6_and_7x7() {
    for size in [6, 7] {
        reaches_target(size, "R U", "R2 L2 U2 D2 F2 B2");
    }
}

#[test]
fn centres_of_one_kind_in_two_colours_on_a_face_are_refused() {
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "R U", "--to", "Rw2 Lw2"]);
    assert!(!ok);
    assert!(error.contains("centres of different colours"), "{error}");
    assert!(error.contains("patterns show <id>"), "{error}");
}
