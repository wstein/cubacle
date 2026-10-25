//! `solve --to TARGET` without a cube starts from the solved cube.

use cubacle::notation::parse_expression;
use cubacle::puzzle::{reaches, Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
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

const NOTE: &str = "start: the solved cube (no cube given; routing to the target)";

fn state(size: usize, moves: &str) -> Facelets {
    let kind = PuzzleKind::of_size(size).expect("size");
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("fits")
}

/// Solves `--to <state of the scramble>` with no cube and replays the answer.
fn routes(size: usize, scramble: &str) {
    let target = state(size, scramble);
    let (ok, solution, error) = cubacle(&["solve", "--to", &target.to_string()]);
    assert!(ok, "{size}x{size}: {error}");
    assert!(error.contains(NOTE), "{error}");
    let moves: MoveSequence = solution.parse().expect("move notation");
    assert!(reaches(&Facelets::solved(target.kind()), &moves, &target));
}

#[test]
fn state_targets_of_every_size() {
    routes(2, "R U F' R2");
    routes(3, "R U F' R2 D L'");
    routes(4, "R U F' D2");
    routes(5, "R U F' D2");
    routes(6, "R U F' D2");
    routes(7, "R U F' D2");
}

#[test]
fn move_target_uses_size() {
    let (ok, solution, error) = cubacle(&["solve", "--size", "4", "--to", "R U F'"]);
    assert!(ok, "{error}");
    assert!(error.contains(NOTE));
    let moves: MoveSequence = solution.parse().expect("moves");
    assert!(reaches(
        &Facelets::solved(PuzzleKind::Cube4),
        &moves,
        &state(4, "R U F'")
    ));
}

#[test]
fn icons_pass_through() {
    let target = state(3, "R U F'").to_string();
    let (ok, out, error) = cubacle(&["solve", "--icons", "--to", &target]);
    assert!(ok, "{error}");
    assert!(out.contains("move 1 of"), "{out}");
}

#[test]
fn size_mismatch_is_refused() {
    let target = state(4, "R U").to_string();
    let (ok, _, error) = cubacle(&["solve", "--size", "3", "--to", &target]);
    assert!(!ok);
    assert!(error.contains("--size"), "{error}");
}

#[test]
fn no_cube_error_text() {
    for args in [&["solve"][..], &["solve", "-m", "5"], &["convert"]] {
        let (ok, _, error) = cubacle(args);
        assert!(!ok);
        assert!(
            error.contains("error: no cube given; give a cube state or --scramble (or use --to TARGET to start from the solved cube)"),
            "{error}"
        );
    }
}

#[test]
fn now_needs_a_real_cube() {
    let (ok, _, error) = cubacle(&["solve", "--now", "x", "--to", "R"]);
    assert!(!ok);
    assert!(error.contains("no cube given"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--now", "x"]);
    assert!(!ok);
    assert!(error.contains("no cube given"), "{error}");
}

#[test]
fn scramble_conflict_stays() {
    let (ok, _, error) = cubacle(&["solve", "--scramble", "R", "U", "--to", "F"]);
    assert!(!ok, "{error}");
}
