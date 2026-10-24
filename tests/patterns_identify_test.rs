//! The `patterns identify` subcommand: what the structure of a cube says.

use std::process::Command;

fn identify(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("patterns")
        .arg("identify")
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn a_solved_cube_has_every_symmetry() {
    let (ok, text) = identify(&["R R'"]);
    assert!(ok);
    assert!(
        text.contains("symmetry: 48 of 48 operations fix it (24 rotations, 24 mirrored)"),
        "{text}"
    );
}

#[test]
fn a_scrambled_cube_has_only_the_identity() {
    let (ok, text) = identify(&["R U R' U'"]);
    assert!(ok);
    assert!(text.contains("symmetry: 1 of 48"), "{text}");
}

#[test]
fn a_checkerboard_keeps_all_symmetries_up_to_colours() {
    let (ok, text) = identify(&["--size", "4", "-s", "R2 L2 U2 D2 F2 B2"]);
    assert!(ok);
    assert!(text.contains("symmetry: 48 of 48"), "{text}");
}

#[test]
fn a_bad_cube_is_an_error() {
    let (ok, _) = identify(&["not a cube"]);
    assert!(!ok);
}
