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

#[test]
fn axis_groups_do_not_split_rotations_at_token_boundaries() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args([
            "moves",
            "--size",
            "5",
            "--icons",
            "--graphics",
            "plain",
            "--group",
            "axis",
            "L' x",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let groups: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("group "))
        .collect();
    assert_eq!(groups, ["group 1 of 1: L' x"]);
}

#[test]
fn inverse_accepts_maximum_expression_depth() {
    let mut expression = String::from("x");
    for _ in 0..100 {
        expression = format!("({expression})");
    }
    check_steps(&["--group", "none", "--inverse"], &expression, &["x'"]);
}

#[test]
fn axis_groups_keep_adjacent_rotations_together() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args([
            "moves",
            "--size",
            "5",
            "--icons",
            "--graphics",
            "plain",
            "--group",
            "axis",
            "L' x x'",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let groups: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("group "))
        .collect();
    assert_eq!(groups, ["group 1 of 1: L' x x'"]);
}

#[test]
fn inverse_reverses_marked_groups_and_rotations() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args([
            "moves",
            "--size",
            "5",
            "--icons",
            "--graphics",
            "plain",
            "--group",
            "dots",
            "--inverse",
            "R x · y U",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let groups: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("group "))
        .collect();
    assert_eq!(groups, ["group 1 of 2: U' y'", "group 2 of 2: x' R'"]);
}

#[test]
fn ungrouped_icons_do_not_enforce_group_mark_limit() {
    let mut expression = String::from("x");
    for _ in 0..100 {
        expression = format!("({expression})");
    }
    expression.push_str("*2001");
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args([
            "moves", "--size", "5", "--icons", "--group", "none", "--format", "svg",
        ])
        .arg(expression)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("svg and rgba hold one icon"), "{error}");
}
