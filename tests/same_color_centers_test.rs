//! Solving and patterns are color based: centers of one color that trade
//! places leave the cube, the answer of `solve` and the report of `patterns
//! identify` unchanged. There is no strict mode.

use std::process::Command;

use cubacle::notation::parse_expression;
use cubacle::patterns::{cycles_of_moves, detect, patternness};
use cubacle::puzzle::{Facelets, PuzzleKind};

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

fn cubacle(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        // never touch the real home: the first run writes a global config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
    )
}

/// `swap` only trades centers of one color (checked on the pieces), so the
/// cube after `scramble` and the cube after `swap scramble` look the same and
/// every color-based answer about them agrees.
fn same_colors_same_answers(kind: PuzzleKind, scramble: &str, swap: &str) {
    let size = kind.size().to_string();

    // the swap really moves pieces ...
    let moves = parse_expression(swap, kind).expect("swap");
    let cycles = cycles_of_moves(kind, &moves).expect("cycles");
    assert!(
        cycles.orbits.iter().any(|orbit| !orbit.cycles.is_empty()),
        "{swap} moves no piece"
    );
    // ... but leaves the cube looking the same
    assert!(cube(kind, swap).is_solved(), "{swap} changes a color");

    let plain = cube(kind, scramble);
    let traded = cube(kind, &format!("{swap} {scramble}"));
    assert_eq!(plain, traded);

    // the solver sees colors only
    let (ok_a, solution_a) = cubacle(&["solve", "--size", &size, &plain.to_string()]);
    let (ok_b, solution_b) = cubacle(&["solve", "--size", &size, &traded.to_string()]);
    assert!(ok_a && ok_b, "{solution_a} / {solution_b}");
    assert_eq!(solution_a, solution_b);

    // patterns see colors only: fingerprint, patternness and library match
    let (report_a, report_b) = (detect(&plain), detect(&traded));
    assert_eq!(report_a, report_b);
    assert_eq!(report_a.fingerprint.code(), report_b.fingerprint.code());
    assert_eq!(patternness(&plain), patternness(&traded));
    let (_, text_a) = cubacle(&["patterns", "identify", "--size", &size, &plain.to_string()]);
    let (_, text_b) = cubacle(&["patterns", "identify", "--size", &size, &traded.to_string()]);
    assert!(text_a.contains("fingerprint:"), "{text_a}");
    assert_eq!(text_a, text_b);
}

#[test]
fn four_same_colored_centers_are_equal() {
    same_colors_same_answers(
        PuzzleKind::Cube4,
        "R U F' 2R 2U' L2 D",
        "2R2 2U2 2R2 3R2 2U2 3R2",
    );
}

#[test]
fn five_same_colored_centers_are_equal() {
    same_colors_same_answers(
        PuzzleKind::Cube5,
        "R U2 F' 2R 2U' L2 D",
        "2R 4R 2R2 2U2 4R' 2R' 2U2 2R2",
    );
}

#[test]
fn six_same_colored_centers_are_equal() {
    same_colors_same_answers(
        PuzzleKind::Cube6,
        "R U F' 2R 3U' L2 D'",
        "3R2 2U2 3R2 4R2 2U2 4R2",
    );
}
