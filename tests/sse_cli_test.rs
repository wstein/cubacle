//! SSE (Superset ENG) on the command line: read by every command that reads
//! moves, written by `--notation sse`.

use cubacle::notation::{parse_expression, parse_moves};
use cubacle::puzzle::{Facelets, PuzzleKind};
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

#[test]
fn facelets_and_convert_read_sse_like_wca() {
    for (size, sse, wca) in [
        ("5", "TR N2-3U' V3F (R NL)2", "Rw 3Uw' U 4Fw F' R 2L R 2L"),
        ("3", "MR SU CF2 R-", "2R U D' z2 R'"),
        ("7", "WL M2R T4B2", "6Lw L' 4Rw Rw' 4Bw2"),
    ] {
        let a = cubacle(&["facelets", "--size", size, sse]);
        let b = cubacle(&["facelets", "--size", size, wca]);
        assert!(a.0 && b.0, "{a:?} {b:?}");
        assert_eq!(a.1, b.1, "{sse}");
        let c = cubacle(&["convert", "--size", size, sse]);
        assert!(c.0, "{c:?}");
        assert_eq!(c.1, a.1, "{sse}");
    }
}

#[test]
fn a_bad_sse_token_is_named_with_its_position() {
    let (ok, _, error) = cubacle(&["facelets", "--size", "4", "R N2R"]);
    assert!(!ok);
    assert!(
        error.contains("character 2") && error.contains("N2R"),
        "{error}"
    );
    assert!(error.contains("5x5"), "{error}");
    let (ok, _, error) = cubacle(&["facelets", "--size", "2", "TR"]);
    assert!(!ok);
    assert!(
        error.contains("TR") && error.contains("leave a layer"),
        "{error}"
    );
}

#[test]
fn scramble_and_moves_write_sse() {
    for size in [2usize, 3, 4, 5, 6, 7] {
        let kind = PuzzleKind::of_size(size).unwrap();
        let (ok, text, error) = cubacle(&[
            "scramble",
            "--size",
            &size.to_string(),
            "-n",
            "40",
            "--notation",
            "sse",
        ]);
        assert!(ok, "{error}");
        assert!(!text.contains('w'), "{text}");
        assert_eq!(parse_expression(&text, kind).unwrap().len(), 40, "{text}");
    }
    let (ok, text, error) = cubacle(&[
        "moves",
        "--size",
        "7",
        "--notation",
        "sse",
        "3Rw 4R 2-4Uw' 2L 6Fw R",
    ]);
    assert!(ok, "{error}");
    assert_eq!(text, "T3R MR T4U' U NL T6F R");
}

#[test]
fn a_4x4_solution_in_sse_replays() {
    let scramble = "R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l";
    let (ok, wca, error) = cubacle(&["solve", "--size", "4", "-s", scramble]);
    assert!(ok, "{error}");
    let (ok, sse, error) = cubacle(&["solve", "--size", "4", "-s", scramble, "--notation", "sse"]);
    assert!(ok, "{error}");
    assert!(!sse.contains('w'), "{sse}");
    let kind = PuzzleKind::Cube4;
    let moves = parse_expression(&sse, kind).unwrap();
    assert_eq!(moves.len(), parse_moves(&wca).unwrap().len());
    let cube = Facelets::solved(kind)
        .apply(&parse_moves(scramble).unwrap())
        .unwrap();
    assert!(cube.apply(&moves).unwrap().is_solved());
    // an SSE scramble is read too, within the rules of the 4x4 page
    let (ok, _, error) = cubacle(&["solve", "--size", "4", "-s", "R U NR"]);
    assert!(!ok && error.contains("N twists"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--size", "4", "-s", "R U TR MU- WF"]);
    assert!(ok, "{error}");
}

#[test]
fn optimize_ranges_writes_sse_ranges() {
    let scramble = "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";
    let args = ["solve", "--size", "4", "--optimize", "ranges"];
    let (ok, text, error) = cubacle(&[&args[..], &["--notation", "sse", scramble]].concat());
    assert!(ok, "{error}");
    let kind = PuzzleKind::Cube4;
    let cube = Facelets::solved(kind)
        .apply(&parse_moves(scramble).unwrap())
        .unwrap();
    let moves = parse_expression(&text, kind).unwrap();
    assert!(cube.apply(&moves).unwrap().is_solved());
}

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
const NOW: &str = "FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL";
const SOLUTION: &str = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";

#[test]
fn now_reads_an_sse_solution_and_writes_sse() {
    let dashed = SOLUTION.replace('\'', "-");
    let run = |solution: &str, notation: &str| {
        cubacle(&[
            "solve",
            "-s",
            ORIGINAL,
            "--now",
            NOW,
            "--solution",
            solution,
            "--notation",
            notation,
        ])
    };
    let (ok, sse, error) = run(&dashed, "sse");
    assert!(ok, "{error}");
    let (ok, wca, error) = run(SOLUTION, "wca");
    assert!(ok, "{error}");
    // a 3x3 solution has face moves only: the three notations agree
    assert_eq!(sse, wca);
}
