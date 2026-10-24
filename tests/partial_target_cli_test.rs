//! `solve --to` with open stickers (X or ?): the moves reach every constrained
//! sticker, complete targets keep their old output, and errors are clear.

use cubacle::notation::{parse_expression, parse_partial};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
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

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

#[test]
fn complete_targets_print_what_they_always_printed() {
    // recorded with the binary before partial targets existed
    let (ok, out, err) = cubacle(&["solve", "R U R' U' F2 D", "--to", "L2 B"]);
    assert!(ok, "{err}");
    assert_eq!(out, "U' R2 D' U2 R2 D' U2 F2 D U' R2 U' R' U' L2 R' B");
    assert_eq!(err, "moves 17\nmetrics: htm 17, qtm 24, stm 17");
    let (ok, out, err) = cubacle(&["solve", "--size", "2", "R U F2", "--to", "D L'"]);
    assert!(ok, "{err}");
    assert_eq!(out, "F2 D' F' D F'");
    assert_eq!(err, "moves 5\nmetrics: htm 5, qtm 6, stm 5");
}

#[test]
fn a_2x2_target_with_open_stickers_is_reached_and_proved_shortest() {
    let start = cube(PuzzleKind::Cube2, "R U F2 R' U'");
    for to in [
        "UUUUXXXXXXXXXXXXXXXXXXXX",
        "UUUU????????????????????",
        "WWWWXXXXXXXXXXXXXXXXXXXX",
    ] {
        let (ok, out, err) = cubacle(&["solve", &start.to_string(), "--to", to]);
        assert!(ok, "{to}: {err}");
        assert!(err.contains("shortest"), "{err}");
        let moves: MoveSequence = out.parse().unwrap();
        let target = parse_partial(to, None).unwrap().unwrap();
        assert!(target.reached_by(&start, &moves), "{to}: {out}");
    }
}

#[test]
fn nothing_constrained_needs_no_move() {
    let start = cube(PuzzleKind::Cube2, "R U F2 R' U'");
    let (ok, out, err) = cubacle(&[
        "solve",
        &start.to_string(),
        "--to",
        "XXXXXXXXXXXXXXXXXXXXXXXX",
    ]);
    assert!(ok, "{err}");
    assert_eq!(out, "");
    assert!(err.starts_with("moves 0"), "{err}");
}

#[test]
fn unreachable_and_contradictory_targets_say_so() {
    let start = cube(PuzzleKind::Cube2, "R U");
    // the three stickers of one corner all U: no corner has that
    let (ok, _, err) = cubacle(&[
        "solve",
        &start.to_string(),
        "--to",
        "XXXUUXXXXUXXXXXXXXXXXXXX",
    ]);
    assert!(!ok);
    assert!(err.contains("unreachable"), "{err}");
    let (ok, _, err) = cubacle(&[
        "solve",
        &start.to_string(),
        "--to",
        "UUUUUUUUUUUUXXXXXXXXXXXX",
    ]);
    assert!(!ok);
    assert!(err.contains("contradictory"), "{err}");
}

#[test]
fn the_size_must_match_and_unsupported_forms_are_named() {
    let (ok, _, err) = cubacle(&[
        "solve",
        "--size",
        "3",
        "R U",
        "--to",
        "UUUUXXXXXXXXXXXXXXXXXXXX",
    ]);
    assert!(!ok);
    assert!(err.contains("3x3x3") && err.contains("2x2x2"), "{err}");
    let (ok, _, err) = cubacle(&[
        "solve",
        "R U",
        "--to",
        "UUUUUUUUUXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    ]);
    assert!(!ok);
    assert!(err.contains("unsupported"), "{err}");
    let (ok, _, err) = cubacle(&[
        "solve",
        "--size",
        "2",
        "R U",
        "--to",
        "UUUUXXXXXXXXXXXXXXXXXXXX",
        "--all",
    ]);
    assert!(!ok);
    assert!(err.contains("--all"), "{err}");
}
