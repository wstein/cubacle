//! `solve --within K` on invented states: the route is optimal in its
//! alphabet, replays, and the exit codes and refusals are as documented.

use std::process::Command;

use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};

struct Run {
    code: i32,
    out: String,
    err: String,
}

fn cubacle(args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn kind(size: usize) -> PuzzleKind {
    match size {
        2 => PuzzleKind::Cube2,
        3 => PuzzleKind::Cube3,
        4 => PuzzleKind::Cube4,
        5 => PuzzleKind::Cube5,
        6 => PuzzleKind::Cube6,
        _ => PuzzleKind::Cube7,
    }
}

fn cube_of(size: usize, moves: &str) -> Facelets {
    let moves: MoveSequence = moves.parse().unwrap();
    Facelets::solved(kind(size)).apply(&moves).unwrap()
}

/// The printed route, parsed (an empty route prints an empty line).
fn route(run: &Run) -> MoveSequence {
    run.out.trim().parse().unwrap()
}

#[test]
fn short_scrambles_are_solved_optimally_on_every_size() {
    let scrambles = [
        (2, "R U F' R' U2 F"),
        (3, "R U2 F' D L2 B"),
        (4, "R Uw2 F' Dw L2 B"),
        (5, "Rw U' F Dw2 L B'"),
        (6, "R Uw F'"),
        (7, "Rw U 3Fw'"),
    ];
    for (size, scramble) in scrambles {
        let words: Vec<&str> = scramble.split(' ').collect();
        let counts: Vec<usize> = if size >= 6 {
            vec![words.len()]
        } else {
            (1..=words.len()).collect()
        };
        for len in counts {
            let prefix = words[..len].join(" ");
            let size_text = size.to_string();
            let run = cubacle(&["solve", "-s", &prefix, "--size", &size_text, "--within"]);
            assert_eq!(run.code, 0, "{size} {prefix}: {}", run.err);
            let found = route(&run);
            assert!(found.len() <= len, "{size} {prefix}: {}", run.out);
            let cube = cube_of(size, &prefix);
            assert!(cube.apply(&found).unwrap().is_solved(), "{size} {prefix}");
        }
    }
}

#[test]
fn a_slice_scramble_needs_slices_or_blocks() {
    let base = ["solve", "-s", "2R 2U 2F", "--size", "4", "--within=3"];
    let plain = cubacle(&base);
    assert_eq!(plain.code, 2, "{}", plain.err);
    assert_eq!(plain.out, "");
    assert_eq!(
        plain.err.trim(),
        "error: no route of at most 3 moves in the outer-block alphabet (obtm)"
    );
    for flag in ["--slices", "--blocks"] {
        let mut args = base.to_vec();
        args.push(flag);
        let run = cubacle(&args);
        assert_eq!(run.code, 0, "{flag}: {}", run.err);
        assert_eq!(route(&run).len(), 3, "{flag}");
        assert!(cube_of(4, "2R 2U 2F")
            .apply(&route(&run))
            .unwrap()
            .is_solved());
    }
}

#[test]
fn the_target_is_moves_or_a_state() {
    let start = cube_of(3, "R U");
    let target = cube_of(3, "R U F D2");
    let want = |run: &Run| {
        assert_eq!(run.code, 0, "{}", run.err);
        let found = route(run);
        assert_eq!(found.len(), 2);
        assert_eq!(start.apply(&found).unwrap(), target);
    };
    want(&cubacle(&[
        "solve", "-s", "R U", "--to", "R U F D2", "--within",
    ]));
    let state = target.to_string();
    let start_state = start.to_string();
    want(&cubacle(&[
        "solve",
        &start_state,
        "--to",
        &state,
        "--within",
    ]));
}

#[test]
fn a_target_with_mixed_centers_works() {
    for (size, slices) in [(4usize, "2R 2U'"), (7, "3R 2U' 5F")] {
        let target = cube_of(size, slices);
        let start = cube_of(size, &format!("{slices} F R"));
        let size_text = size.to_string();
        for target_text in [slices.to_owned(), target.to_string()] {
            let run = cubacle(&[
                "solve",
                "-s",
                &format!("{slices} F R"),
                "--size",
                &size_text,
                "--to",
                &target_text,
                "--within=3",
                "--slices",
            ]);
            assert_eq!(run.code, 0, "{size}: {}", run.err);
            let found = route(&run);
            assert!(found.len() <= 2, "{size}: {}", run.out);
            assert_eq!(start.apply(&found).unwrap(), target, "{size}");
        }
    }
}

#[test]
fn then_solve_falls_back_to_the_normal_solver() {
    let scramble = "R U2 F' D L2 B R' U F2 D'";
    let run = cubacle(&["solve", "-s", scramble, "--within=3", "--then-solve"]);
    assert_eq!(run.code, 0, "{}", run.err);
    assert!(
        run.err
            .contains("no route within 3 moves; solved with the normal solver"),
        "{}",
        run.err
    );
    assert!(cube_of(3, scramble)
        .apply(&route(&run))
        .unwrap()
        .is_solved());
    let bare = cubacle(&["solve", "-s", scramble, "--within=3"]);
    assert_eq!(bare.code, 2);
    assert_eq!(bare.out, "");
}

#[test]
fn a_hit_does_not_run_the_fallback() {
    let run = cubacle(&["solve", "-s", "R U", "--within=3", "--then-solve"]);
    assert_eq!(run.code, 0);
    assert!(!run.err.contains("normal solver"), "{}", run.err);
    assert_eq!(route(&run).len(), 2);
}

#[test]
fn the_limits_have_their_own_exit_code() {
    let memory = cubacle(&["solve", "-s", "R U", "--within=4", "--max-memory", "0"]);
    assert_eq!(memory.code, 3);
    assert_eq!(memory.err.trim(), "error: memory limit");
    assert_eq!(memory.out, "");
    let time = cubacle(&[
        "solve",
        "-s",
        "Rw U 3Fw' 2L Dw B' R",
        "--size",
        "7",
        "--within=7",
        "--blocks",
        "--max-time",
        "1",
    ]);
    assert_eq!(time.code, 3, "{}", time.err);
    assert_eq!(time.err.trim(), "error: time limit");
}

#[test]
fn the_options_that_cannot_combine_are_refused_by_name() {
    for args in [
        &["--optimal"][..],
        &["--moves", "htm"][..],
        &["--count", "2"][..],
        &["--all"][..],
        &[
            "--now",
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        ][..],
        &["--fast"][..],
        &["-m", "20"][..],
    ] {
        let mut full = vec!["solve", "-s", "R U", "--within=3"];
        full.extend_from_slice(args);
        let run = cubacle(&full);
        assert_eq!(run.code, 1, "{args:?}: {}", run.err);
        assert_eq!(run.out, "", "{args:?}");
        let name = args[0];
        assert!(
            run.err.contains(name) || name == "-m",
            "{args:?}: {}",
            run.err
        );
        assert!(run.err.contains("--within"), "{args:?}: {}", run.err);
    }
    for args in [&["--then-solve"][..], &["--blocks"][..]] {
        let mut full = vec!["solve", "-s", "R U"];
        full.extend_from_slice(args);
        let run = cubacle(&full);
        assert_eq!(run.code, 1, "{args:?}: {}", run.err);
        assert!(run.err.contains("needs --within"), "{}", run.err);
    }
}

#[test]
fn the_printing_options_work() {
    let sign = cubacle(&[
        "solve",
        "-s",
        "R Uw",
        "--size",
        "4",
        "--within=2",
        "--notation",
        "sign",
    ]);
    assert_eq!(sign.code, 0, "{}", sign.err);
    assert!(sign.err.contains("metrics: obtm 2"), "{}", sign.err);
    let ranges = cubacle(&[
        "solve",
        "-s",
        "R Uw",
        "--size",
        "4",
        "--within=2",
        "--optimise",
        "ranges",
        "--group",
        "axis",
    ]);
    assert_eq!(ranges.code, 0, "{}", ranges.err);
}

#[test]
fn an_impossible_goal_is_no_route() {
    // The centres of the 3x3x3 never move with face turns: a start with a
    // turned middle layer cannot reach the solved cube in that alphabet at all.
    let run = cubacle(&["solve", "-s", "M", "--within=7"]);
    assert_eq!(run.code, 2, "{}", run.err);
    let with = cubacle(&["solve", "-s", "M", "--within=7", "--slices"]);
    assert_eq!(with.code, 0, "{}", with.err);
}
