//! Rotation tokens remain individual steps in move pictures.
use std::process::Command;

#[test]
fn rotations_are_drawn_as_input_steps() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args([
            "moves",
            "--size",
            "5",
            "--icons",
            "--graphics",
            "plain",
            "R2 L2 x y z x2 y2 z2 x' y' z'",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("move "))
        .collect();
    assert_eq!(lines.len(), 11);
    for (index, token) in [
        "R2", "L2", "x", "y", "z", "x2", "y2", "z2", "x'", "y'", "z'",
    ]
    .iter()
    .enumerate()
    {
        assert!(
            lines[index].starts_with(&format!("move {} of 11: {token}", index + 1)),
            "{}",
            lines[index]
        );
    }
}

fn check_steps(options: &[&str], expression: &str, expected: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["moves", "--size", "5", "--icons", "--graphics", "plain"])
        .args(options)
        .arg(expression)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("move "))
        .collect();
    assert_eq!(lines.len(), expected.len(), "{expression}: {lines:?}");
    for (line, token) in lines.iter().zip(expected) {
        assert!(line.contains(&format!(": {token}")), "{line}");
    }
}

#[test]
fn inverse_keeps_rotation() {
    check_steps(&["--inverse"], "R x", &["x'", "R'"]);
}

#[test]
fn from_end_keeps_complete_rotation() {
    check_steps(&["--from-end", "2"], "R x", &["x"]);
}

#[test]
fn mixed_sse_keeps_rotation() {
    check_steps(&["--notation", "sse"], "TR x", &["TR", "x"]);
}

#[test]
fn none_keeps_rotation() {
    check_steps(&["--group", "none"], "R x", &["R", "x"]);
}

fn check_groups(options: &[&str], expression: &str, groups: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["moves", "--size", "5", "--icons", "--graphics", "plain"])
        .args(options)
        .arg(expression)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("group "))
        .collect();
    assert_eq!(lines, groups);
    assert_eq!(
        text.lines().filter(|line| line.starts_with("x ")).count(),
        1
    );
    assert_eq!(
        text.lines().filter(|line| line.starts_with("y ←")).count(),
        1
    );
}

#[test]
fn axis_groups_keep_rotations() {
    check_groups(
        &["--group", "axis"],
        "R x U y",
        &["group 1 of 2: R x", "group 2 of 2: U y"],
    );
}

#[test]
fn marked_groups_keep_rotations() {
    check_groups(
        &["--group", "dots"],
        "R x · y U",
        &["group 1 of 2: R x", "group 2 of 2: y U"],
    );
}
