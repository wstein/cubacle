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
fn convert_writes_colours_in_the_chosen_scheme() {
    let token = "A4ib8aejxj4Y";
    let (ok, western, _) = cubacle(&["convert", token, "--to", "color"]);
    assert!(ok);
    assert_eq!(
        western,
        "GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO"
    );
    let (ok, named, _) = cubacle(&["convert", token, "--to", "colour", "--scheme", "western"]);
    assert!(ok);
    assert_eq!(named, western);
    let (ok, japanese, _) = cubacle(&["convert", token, "--to", "color", "--scheme", "jp"]);
    assert!(ok);
    assert_ne!(japanese, western);
    // The letters read back to the same cube with the scheme that wrote them.
    for (letters, scheme) in [(&western, "boy"), (&japanese, "jp")] {
        let (ok, facelets, _) = cubacle(&["convert", "--scheme", scheme, letters]);
        assert!(ok);
        let (_, expected, _) = cubacle(&["convert", token]);
        assert_eq!(facelets, expected);
    }
}

#[test]
fn colours_cover_the_expression_on_a_7x7_and_survive_a_round_trip() {
    let expression = "((M' U)*4 y' z')*3";
    let args = ["convert", "--size", "7", expression];
    let (ok, colours, error) = cubacle(&[
        "convert", "--size", "7", expression, "--to", "color", "--scheme", "western",
    ]);
    assert!(ok, "{error}");
    assert_eq!(colours.len(), 6 * 49);
    assert!(colours.chars().all(|c| "WRGYOB".contains(c)));
    let (ok, facelets, _) = cubacle(&args);
    assert!(ok);
    let (ok, back, error) = cubacle(&["convert", "--scheme", "western", &colours]);
    assert!(ok, "{error}");
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
