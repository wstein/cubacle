//! `canon` prints the canonical Orbit64 token of a state in any input form.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

/// The token of a scramble on a cube of `size`.
fn canon(size: &str, scramble: &str) -> String {
    let (ok, out, err) = cubacle(&["canon", "--size", size, "-s", scramble]);
    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    out
}

/// The facelets of a scramble with the letters renamed through `from` to `to`.
fn renamed(size: &str, scramble: &str, from: &str, to: &str) -> String {
    let (ok, out, err) = cubacle(&["convert", "--size", size, "--to", "facelets", scramble]);
    assert!(ok, "{err}");
    out.chars()
        .map(|c| from.find(c).map_or(c, |i| to.as_bytes()[i] as char))
        .collect()
}

#[test]
fn the_solved_cube_has_the_zero_token() {
    assert_eq!(canon("3", ""), "AAAAAAAAAAAA");
    assert_eq!(canon("2", ""), "AAAAA");
}

#[test]
fn a_state_and_its_mirror_image_have_one_token() {
    // R U mirrored left to right is L' U'
    assert_eq!(canon("3", "R U"), canon("3", "L' U'"));
    assert_eq!(canon("3", "R U"), "C0RkZMybxEmw");
    assert_ne!(canon("3", "R U"), canon("3", "R U'"));
}

#[test]
fn a_color_renamed_state_has_the_same_token() {
    // U and D swapped with R and L: a half turn of the whole cube about F
    let plain = canon("3", "R U R' U'");
    let facelets = renamed("3", "R U R' U'", "URDL", "DLUR");
    let (ok, out, err) = cubacle(&["canon", &facelets]);
    assert!(ok, "{err}");
    assert_eq!(out, plain);
    // the colors renamed by a permutation that is no turn of the cube
    let facelets = renamed("3", "R U R' U'", "URFDLB", "FBULRD");
    let (ok, out, err) = cubacle(&["canon", &facelets]);
    assert!(ok, "{err}");
    assert_eq!(out, plain);
}

#[test]
fn a_4x4_and_a_7x7_have_mirror_invariant_tokens() {
    let four = canon("4", "Rw U");
    assert_eq!(four.len(), 27);
    assert_eq!(four, canon("4", "Lw' U'"));
    let facelets = renamed("4", "Rw U", "URFDLB", "FBULRD");
    let (ok, out, err) = cubacle(&["canon", &facelets]);
    assert!(ok, "{err}");
    assert_eq!(out, four);
    let seven = canon("7", "R U Rw");
    assert_eq!(seven.len(), 90);
    assert_eq!(seven, canon("7", "L' U' Lw'"));
}

#[test]
fn a_token_is_read_back_and_is_its_own_canonical_token() {
    for (size, scramble) in [("3", "R U F"), ("4", "Rw U"), ("7", "R U Rw")] {
        let token = canon(size, scramble);
        let (ok, out, err) = cubacle(&["canon", &token]);
        assert!(ok, "{err}");
        assert_eq!(out, token);
    }
}

#[test]
fn verbose_and_json_name_the_schema_and_the_size() {
    let (ok, out, _) = cubacle(&["canon", "-v", "--size", "4", "-s", "Rw"]);
    assert!(ok);
    let token = canon("4", "Rw");
    assert_eq!(
        out,
        format!("canonical: {token}\nschema: orbit64-4x4-draft@1\nsize: 4x4x4")
    );
    let (ok, out, _) = cubacle(&["canon", "--format", "json", "-s", "R U"]);
    assert!(ok);
    assert_eq!(
        out,
        "{\"canonical\":{\"token\":\"C0RkZMybxEmw\",\"schema\":\"orbit64-3x3-draft@1\"},\"size\":3}"
    );
}

#[test]
fn an_impossible_state_is_an_error_with_a_nonzero_exit() {
    // the URF corner twisted in place
    let (ok, out, err) = cubacle(&[
        "canon",
        "UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    ]);
    assert!(!ok);
    assert!(out.is_empty(), "{out}");
    assert!(err.starts_with("error: "), "{err}");
    let (ok, _, err) = cubacle(&["canon", "not a cube"]);
    assert!(!ok);
    assert!(err.starts_with("error: "), "{err}");
}
