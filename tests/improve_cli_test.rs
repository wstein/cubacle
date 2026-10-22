use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use std::process::Command;
use std::time::{Duration, Instant};

const SCRAMBLE: &str = "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'";

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

fn solves(solution: &str) -> usize {
    let moves: MoveSequence = solution.parse().expect("moves");
    let cube = Facelets::solved(PuzzleKind::Cube3)
        .apply(&SCRAMBLE.parse().unwrap())
        .unwrap()
        .apply(&moves)
        .unwrap();
    assert!(cube.is_solved(), "{solution}");
    moves.len()
}

#[test]
fn probes_are_deterministic_and_never_longer() {
    let (ok, first, error) = cubacle(&["solve", "-s", SCRAMBLE]);
    assert!(ok, "{error}");
    let (ok, a, error) = cubacle(&["solve", "-s", SCRAMBLE, "--probes", "500"]);
    assert!(ok, "{error}");
    let (_, b, _) = cubacle(&["solve", "-s", SCRAMBLE, "--probes", "500"]);
    assert_eq!(a, b);
    assert!(solves(&a) <= solves(&first));
}

#[test]
fn improve_and_parallel_return_a_valid_solution_within_the_budget() {
    for extra in [&["--improve"][..], &["--improve", "--parallel"]] {
        let mut args = vec!["solve", "-s", SCRAMBLE, "-b", "200", "-v"];
        args.extend(extra);
        let started = Instant::now();
        let (ok, solution, error) = cubacle(&args);
        assert!(ok, "{extra:?}: {error}");
        solves(&solution);
        assert!(error.contains("improved from"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}

#[test]
fn improve_flags_are_refused_where_they_do_not_fit() {
    let cases: [(&[&str], &str); 7] = [
        (&["--parallel", "-b", "100"], "together with --improve"),
        (&["--improve", "--parallel"], "needs a time budget"),
        (&["--probes", "5", "--improve"], "cannot be used"),
        (&["--size", "2", "--improve"], "already optimal"),
        (&["--size", "6", "--probes", "5"], "not available"),
        (&["--size", "7", "--improve"], "not available"),
        (
            &["--size", "4", "--improve", "--parallel", "-b", "100"],
            "3x3x3 only",
        ),
    ];
    for (flags, message) in cases {
        let mut args = vec!["solve", "-s", "R U"];
        for flag in flags {
            args.push(flag);
        }
        let (ok, _, error) = cubacle(&args);
        assert!(!ok, "{flags:?}");
        assert!(error.contains(message), "{flags:?}: {error}");
    }
}

/// A short scramble of a 4x4x4 or 5x5x5 and the length of the answer.
fn reduction_answer(size: &str, extra: &[&str]) -> (String, usize) {
    let scramble = "R U F' Rw2 Uw Fw' L D2 B";
    let mut args = vec!["solve", "-s", scramble, "--size", size];
    args.extend(extra);
    let (ok, solution, error) = cubacle(&args);
    assert!(ok, "{size} {extra:?}: {error}");
    let kind = PuzzleKind::of_size(size.parse().unwrap()).unwrap();
    let moves: MoveSequence = solution.parse().expect("moves");
    let cube = Facelets::solved(kind)
        .apply(&scramble.parse().unwrap())
        .unwrap()
        .apply(&moves)
        .unwrap();
    assert!(cube.is_solved(), "{size} {extra:?}: {solution}");
    (solution, moves.len())
}

#[test]
fn probes_and_improve_work_on_the_4x4x4_and_5x5x5() {
    for size in ["4", "5"] {
        let (_, plain) = reduction_answer(size, &[]);
        let (a, probed) = reduction_answer(size, &["--probes", "2000"]);
        let (b, _) = reduction_answer(size, &["--probes", "2000"]);
        assert_eq!(a, b, "{size}: --probes is deterministic");
        assert!(probed <= plain, "{size}: {probed} against {plain}");
        let (_, improved) = reduction_answer(size, &["--improve", "-b", "2000"]);
        assert!(improved <= plain, "{size}: {improved} against {plain}");
    }
}

/// A 5x5x5 whose plain answer has 69 moves; `-m 67` finds 66. A high probe
/// minimum finds 66 without a limit, for any thread count.
#[test]
fn a_high_probe_minimum_finds_the_short_answer_of_a_5x5x5() {
    let token = "Am9suG17g2aJzonSCHWNZHX9CPO6oqgWCGgVoOTI_bQ";
    let cube = cubacle::notation::parse_cube(token, None).expect("a token");
    let mut answers = Vec::new();
    for threads in ["1", "4"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["solve", token, "--probes", "10000"])
            .env("CUBACLE_THREADS", threads)
            .env(
                "CUBACLE_CACHE_DIR",
                concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
            )
            .output()
            .expect("cubacle runs");
        assert!(output.status.success());
        let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let moves: MoveSequence = text.parse().expect("moves");
        assert!(cube.apply(&moves).unwrap().is_solved());
        assert_eq!(moves.len(), 66);
        answers.push(text);
    }
    assert_eq!(answers[0], answers[1]);
}
