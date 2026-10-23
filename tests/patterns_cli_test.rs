//! The `patterns` subcommand: the list and one pattern.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
    )
}

#[test]
fn lists_patterns_of_a_size() {
    let (ok, text) = cubacle(&["patterns", "--size", "3"]);
    assert!(ok);
    assert!(text.contains("checkerboard"), "{text}");
    assert!(text.contains("3:6/3"), "{text}");
    assert!(!text.contains("six-centre-spots"), "{text}");
}

#[test]
fn shows_a_pattern_by_alias() {
    let (ok, text) = cubacle(&["patterns", "show", "chess", "pattern", "--size", "3"]);
    assert!(ok);
    assert!(text.contains("range metric, 3 moves, optimal"), "{text}");
    assert!(text.contains("2U2 2R2 2F2"), "{text}");
}

#[test]
fn unknown_pattern_and_size_fail() {
    assert!(!cubacle(&["patterns", "show", "no such pattern"]).0);
    assert!(!cubacle(&["patterns", "show", "superflip", "--size", "2"]).0);
}
