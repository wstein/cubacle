//! The `metrics:` lines on standard error, and `moves --metrics/--expand/--optimise`.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn moves_counts_the_words_as_written_on_stderr() {
    let (ok, out, err) = cubacle(&["moves", "R2 L' M x"]);
    assert!(ok, "{err}");
    // Standard output is the lowered sequence, as before.
    assert_eq!(out, "R2 L' 2L Rw L'");
    // The count is of the four words: x is a free rotation, M two face turns.
    assert_eq!(err, "metrics: htm 4, qtm 5, stm 3");
}

#[test]
fn metrics_flag_prints_all_seven() {
    let (ok, _, err) = cubacle(&["moves", "--metrics", "R2 L' M x"]);
    assert!(ok, "{err}");
    assert_eq!(
        err,
        "metrics: htm 4, qtm 5, stm 3, etm 4, obtm 4, btm 3, atm 1"
    );
}

#[test]
fn big_cubes_show_block_counts() {
    let (ok, out, err) = cubacle(&["moves", "--size", "5", "2-3Rw 3Rw2"]);
    assert!(ok, "{err}");
    assert_eq!(out, "3Rw R' 3Rw2");
    assert_eq!(err, "metrics: obtm 3, qtm 4, stm 5, btm 2");
}

#[test]
fn convert_metrics_needs_moves() {
    let (ok, out, err) = cubacle(&["convert", "--metrics", "-s", "R U2"]);
    assert!(ok, "{err}");
    assert_eq!(out.len(), 54);
    assert_eq!(
        err,
        "metrics: htm 2, qtm 3, stm 2, etm 2, obtm 2, btm 2, atm 2"
    );
    let (ok, _, err) = cubacle(&["convert", "--metrics", &out]);
    assert!(!ok);
    assert!(err.contains("needs a move expression"), "{err}");
}

#[test]
fn solve_prints_the_metrics_after_the_move_count() {
    let (ok, out, err) = cubacle(&["solve", "-s", "R U2"]);
    assert!(ok, "{err}");
    let count = out.split_whitespace().count();
    let lines: Vec<&str> = err.lines().collect();
    assert_eq!(lines[0], format!("moves {count}"));
    let htm = format!("metrics: htm {count}, qtm ");
    assert!(lines[1].starts_with(&htm), "{err}");
    assert!(lines[1].contains(", stm "), "{err}");
}
