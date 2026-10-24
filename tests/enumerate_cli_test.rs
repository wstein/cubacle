//! `solve --all`, `--count`, `--depth`, `--raw`: lists of solutions.

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
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

fn solves(scramble: &str, solution: &str) -> bool {
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&scramble.parse::<MoveSequence>().unwrap())
        .unwrap();
    let moves: MoveSequence = solution.parse().unwrap();
    start.apply(&moves).unwrap().is_solved()
}

#[test]
fn all_lists_the_shortest_solutions_and_each_one_solves() {
    let scramble = "R U R' U'";
    let run = cubacle(&["solve", "--all", "--scramble", scramble]);
    assert!(run.ok, "{}", run.err);
    assert_eq!(run.out.lines().count(), 1, "{}", run.out);
    for line in run.out.lines() {
        assert!(solves(scramble, line), "{line}");
    }
    assert!(run.err.contains("moves 4"), "{}", run.err);
}

#[test]
fn commuting_moves_are_one_line_unless_raw() {
    let canonical = cubacle(&["solve", "--all", "--scramble", "R L"]);
    let raw = cubacle(&["solve", "--all", "--raw", "--scramble", "R L"]);
    assert!(canonical.ok && raw.ok);
    assert_eq!(canonical.out.lines().count(), 1);
    assert_eq!(raw.out.lines().count(), 2);
}

#[test]
fn a_count_stops_the_list_and_depth_adds_longer_solutions() {
    let scramble = "R2 U2 F2";
    let run = cubacle(&[
        "solve",
        "--count",
        "3",
        "--depth",
        "8",
        "--moves",
        "half-turns",
        "--scramble",
        scramble,
    ]);
    assert!(run.ok, "{}", run.err);
    let lines: Vec<&str> = run.out.lines().collect();
    assert_eq!(lines.len(), 3, "{}", run.out);
    let lengths: Vec<usize> = lines.iter().map(|l| l.split(' ').count()).collect();
    assert!(lengths.windows(2).all(|w| w[0] <= w[1]), "{lengths:?}");
    assert!(lines.iter().all(|l| solves(scramble, l)));
}

#[test]
fn list_options_need_a_list_and_a_3x3x3() {
    let run = cubacle(&["solve", "--depth", "3", "--scramble", "R"]);
    assert!(!run.ok);
    assert!(run.err.contains("--all or --count"), "{}", run.err);
    let run = cubacle(&["solve", "--size", "4", "--all", "--scramble", "R"]);
    assert!(!run.ok);
    assert!(run.err.contains("3x3x3"), "{}", run.err);
    let run = cubacle(&["solve", "--all", "--improve", "--scramble", "R"]);
    assert!(!run.ok);
    let run = cubacle(&["solve", "--count", "0", "--scramble", "R"]);
    assert!(!run.ok);
}

#[test]
fn a_cube_outside_the_set_is_reported_not_listed() {
    let run = cubacle(&["solve", "--all", "--moves", "half-turns", "--scramble", "R"]);
    assert!(!run.ok);
    assert!(
        run.err.contains("not solvable with half-turns"),
        "{}",
        run.err
    );
    assert!(run.out.is_empty());
}
