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
fn slices_and_rotations_follow_the_documented_identities() {
    for (short, long) in [
        ("x", "R M' L'"),
        ("y", "U E' D'"),
        ("z", "F S B'"),
        ("M", "2L"),
        ("E'", "2D'"),
        ("S2", "2F2"),
    ] {
        let (ok, a, _) = cubacle(&["facelets", short]);
        assert!(ok);
        let (ok, b, _) = cubacle(&["facelets", long]);
        assert!(ok);
        assert_eq!(a, b, "{short} against {long}");
    }
}

#[test]
fn an_even_cube_refuses_a_middle_slice() {
    let (ok, _, error) = cubacle(&["facelets", "--size", "4", "M"]);
    assert!(!ok);
    assert!(error.contains("middle layer"), "{error}");
}

#[test]
fn the_nested_slice_and_rotation_expression_converts_on_a_7x7() {
    let expression = "((M' U)*4 y' z')*3";
    let (ok, facelets, error) = cubacle(&["convert", "--size", "7", expression]);
    assert!(ok, "{error}");
    assert_eq!(facelets.len(), 6 * 49);
    let (ok, token, _) = cubacle(&["convert", "--size", "7", "--to", "token", expression]);
    assert!(ok);
    let (ok, back, _) = cubacle(&["convert", &token]);
    assert!(ok);
    assert_eq!(back, facelets);
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
