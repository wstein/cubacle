//! `solve -v` names a known pattern on stderr; a closed pipe ends quietly.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

const CONFIG: &str = concat!(env!("CARGO_TARGET_TMPDIR"), "/pattern-hint.toml");

fn cubacle() -> Command {
    std::fs::write(CONFIG, "[patterns]\nfiles = [\"builtin\"]\n").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_cubacle"));
    command.env("CUBACLE_CONFIG", CONFIG);
    command
}

fn solve(args: &[&str]) -> (String, String) {
    let output = cubacle().arg("solve").args(args).output().unwrap();
    assert!(output.status.success());
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn verbose_names_the_pattern() {
    let (_, err) = solve(&["-v", "U2 D2 R2 L2 F2 B2"]);
    assert!(err.contains("pattern: checkerboard"), "{err}");
}

#[test]
fn the_hint_is_for_verbose_only_and_changes_no_answer() {
    let (plain, err) = solve(&["U2 D2 R2 L2 F2 B2"]);
    assert!(!err.contains("pattern:"), "{err}");
    let (verbose, _) = solve(&["-v", "U2 D2 R2 L2 F2 B2"]);
    assert_eq!(plain, verbose);
}

#[test]
fn no_hint_for_an_ordinary_state() {
    let (_, err) = solve(&["-v", "R U F"]);
    assert!(!err.contains("pattern"), "{err}");
}

#[test]
fn a_closed_pipe_is_quiet() {
    let mut child = cubacle()
        .arg("patterns")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    assert!(!first.is_empty());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}
