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
