//! `solve --icons --optimize ranges`: the pictures follow the compressed
//! solution, one step per range or block move.

use std::process::Command;

use cubacle::notation::parse_expression;
use cubacle::puzzle::{Facelets, PuzzleKind};

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
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The pictures of a run as text: the number of move steps and the `Moves:` line.
fn steps_and_moves(out: &str) -> (usize, String) {
    let steps = out.lines().filter(|l| l.starts_with("move ")).count();
    let moves = out
        .lines()
        .find_map(|l| l.strip_prefix("Moves: "))
        .expect("a Moves: line")
        .to_owned();
    (steps, moves)
}

fn check(size: usize, kind: PuzzleKind, scramble: &str) -> String {
    let n = size.to_string();
    let pictures = |extra: &[&str]| {
        let mut args = vec![
            "solve",
            "--size",
            &n,
            "-s",
            scramble,
            "--slices",
            "--icons",
            "--graphics",
            "plain",
        ];
        args.extend_from_slice(extra);
        cubacle(&args)
    };
    let (ok, plain, error) = pictures(&[]);
    assert!(ok, "{size}: {error}");
    let (ok, packed, error) = pictures(&["--optimize", "ranges"]);
    assert!(ok, "{size}: {error}");
    let (plain_steps, plain_moves) = steps_and_moves(&plain);
    let (packed_steps, packed_moves) = steps_and_moves(&packed);
    assert!(
        packed_steps <= plain_steps,
        "{size}: {packed_steps} > {plain_steps}"
    );
    // one picture per word of the compressed answer
    assert_eq!(
        packed_steps,
        packed_moves.split_whitespace().count(),
        "{size}"
    );
    assert!(
        error.contains(&format!("compressed moves {packed_steps}")),
        "{size}: {error}"
    );
    // replayed from the start, both reach the same state
    let start = Facelets::solved(kind)
        .apply(&parse_expression(scramble, kind).unwrap())
        .unwrap();
    let end = |text: &str| start.apply(&parse_expression(text, kind).unwrap()).unwrap();
    assert_eq!(end(&packed_moves), end(&plain_moves), "{size}");
    packed_moves
}

#[test]
fn the_pictures_of_a_compressed_solution_replay_to_the_plain_end_state() {
    let _ = check(
        4,
        PuzzleKind::Cube4,
        "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw",
    );
    let _ = check(
        5,
        PuzzleKind::Cube5,
        "R U 2R 3U' Fw 2L' 3B2 D 2D' R' U2 3F Uw Rw' F2 Lw",
    );
    // the 7x7x7 answer of this scramble has a block, 2-3Bw2
    for (size, kind) in [(6, PuzzleKind::Cube6), (7, PuzzleKind::Cube7)] {
        let moves = check(
            size,
            kind,
            "R U 2R 3U' Fw 2L' 3B2 D 2D' R' U2 3F Uw Rw' F2 Lw",
        );
        assert_eq!(moves.contains("2-3Bw2"), size == 7, "{moves}");
    }
}

#[test]
fn a_range_is_one_step_with_its_own_label() {
    // two inner layers turned alike are one block, one picture, one arrow
    let (ok, out, error) = cubacle(&[
        "solve",
        "--size",
        "5",
        "--to",
        "2-3Rw",
        "--icons",
        "--graphics",
        "plain",
        "--optimize",
        "ranges",
    ]);
    assert!(ok, "{error}");
    assert!(out.contains("move 1 of 1: 2-3Rw"), "{out}");
    assert_eq!(steps_and_moves(&out), (1, "2-3Rw".to_owned()));
    assert!(error.contains("compressed moves 1"), "{error}");
}

#[test]
fn group_stays_refused_with_icons_and_says_why() {
    let (ok, _, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "-s",
        "R U",
        "--icons",
        "--optimize",
        "ranges",
        "--group",
        "phases",
    ]);
    assert!(!ok);
    assert!(
        error.contains("--group cannot be combined with --icons: the pictures are already headed"),
        "{error}"
    );
}
