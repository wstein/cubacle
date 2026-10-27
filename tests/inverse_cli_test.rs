//! `solve --inverse` and `moves --from-end`.

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

const SCRAMBLE: &str = "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";

fn inverse_of(moves: &str, size: &str) -> String {
    cubacle(&["moves", "--size", size, "--inverse", moves]).1
}

#[test]
fn the_inverse_of_a_solution_is_the_moves_that_build_the_start() {
    let scramble = "R U F D2 L' B R2 U' F2 D L2 B' U2 R F";
    let (ok, plain, _) = cubacle(&["solve", "-s", scramble]);
    assert!(ok);
    let (ok, back, error) = cubacle(&["solve", "-s", scramble, "--inverse"]);
    assert!(ok, "{error}");
    assert_eq!(back, inverse_of(&plain, "3"));
    assert!(error.contains("moves 15"), "{error}");
}

#[test]
fn the_counts_stay_those_of_the_solution() {
    let (_, plain, plain_error) = cubacle(&["solve", "--size", "4", SCRAMBLE]);
    let (_, back, back_error) = cubacle(&["solve", "--size", "4", "--inverse", SCRAMBLE]);
    assert_ne!(plain, back);
    assert_eq!(
        plain_error.lines().find(|l| l.starts_with("metrics")),
        back_error.lines().find(|l| l.starts_with("metrics"))
    );
}

#[test]
fn the_phases_come_in_reverse_order_with_undo_names() {
    let (ok, _, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "--inverse",
        "--group",
        "phases",
        "-v",
        SCRAMBLE,
    ]);
    assert!(ok, "{error}");
    let phases: Vec<&str> = error.lines().filter(|l| l.starts_with("phase ")).collect();
    assert_eq!(phases.len(), 4, "{error}");
    assert!(phases[0].starts_with("phase 1: undo phase 4: "), "{error}");
    assert!(phases[3].starts_with("phase 4: undo phase 1: "), "{error}");
}

#[test]
fn every_and_ranges_work_on_the_inverse() {
    let (ok, text, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "--inverse",
        "--every",
        "10",
        "--optimize",
        "ranges",
        SCRAMBLE,
    ]);
    assert!(ok, "{error}");
    assert!(text.contains(" · "), "{text}");
}

#[test]
fn pictures_start_from_the_solved_cube() {
    let (ok, text, error) = cubacle(&["solve", "-s", "R U F", "--inverse", "--icons"]);
    assert!(ok, "{error}");
    assert!(text.contains("Start state: AAAAAAAAAAAA"), "{text}");
    assert!(text.contains("Moves:"), "{text}");
}

#[test]
fn a_target_is_the_start_of_the_inverse() {
    let (ok, forward, _) = cubacle(&["solve", "-s", "R U F", "--to", "L D"]);
    assert!(ok);
    let (ok, back, _) = cubacle(&["solve", "-s", "R U F", "--to", "L D", "--inverse"]);
    assert!(ok);
    assert_eq!(back, inverse_of(&forward, "3"));
}

#[test]
fn senseless_combinations_are_refused() {
    for other in [vec!["--now", "R"], vec!["--all"], vec!["--count", "3"]] {
        let mut args = vec!["solve", "-s", "R U F", "--inverse"];
        args.extend(other);
        let (ok, _, error) = cubacle(&args);
        assert!(!ok);
        assert!(error.contains("cannot be used with"), "{error}");
    }
}

#[test]
fn from_end_takes_the_last_moves_only() {
    let (ok, text, _) = cubacle(&["moves", "--from-end", "2", "R U F D L B"]);
    assert!(ok);
    assert_eq!(text, "L B");
    let (ok, text, _) = cubacle(&["moves", "--inverse", "--from-end", "3", "R U F D L B"]);
    assert!(ok);
    assert_eq!(text, "B' L' D'");
    let (ok, _, error) = cubacle(&["moves", "--from-end", "9", "R U"]);
    assert!(!ok);
    assert!(error.contains("only 2 moves"), "{error}");
}
