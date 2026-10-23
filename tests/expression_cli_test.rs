//! Move expressions (`(...)*N`, and later slices and rotations) on the command line.

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
fn facelets_expands_repeated_groups() {
    let (ok, grouped, _) = cubacle(&["facelets", "(R U)*3 F"]);
    assert!(ok);
    let (ok, plain, _) = cubacle(&["facelets", "R U R U R U F"]);
    assert!(ok);
    assert_eq!(grouped, plain);
    let (ok, solved, _) = cubacle(&["facelets", "(R U R' U')*6"]);
    assert!(ok);
    assert_eq!(
        solved,
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
    );
}

#[test]
fn convert_takes_an_expression_as_the_cube_with_size() {
    let (ok, from_expression, _) = cubacle(&["convert", "--size", "4", "((R U)*2 3Rw)*2"]);
    assert!(ok);
    let (ok, from_scramble, _) = cubacle(&[
        "convert",
        "--size",
        "4",
        "--scramble",
        "R U R U 3Rw R U R U 3Rw",
    ]);
    assert!(ok);
    assert_eq!(from_expression, from_scramble);
}

#[test]
fn a_broken_expression_is_reported() {
    let (ok, _, error) = cubacle(&["facelets", "(R U"]);
    assert!(!ok);
    assert!(error.contains("unbalanced"), "{error}");
    let (ok, _, error) = cubacle(&["facelets", "(R U)*0"]);
    assert!(!ok);
    assert!(error.contains("'*'"), "{error}");
}
