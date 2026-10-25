//! A scramble that moves the fixed face centres of an odd cube is refused with
//! an error and a non-zero exit code, on every size, and never silently.

use std::process::Command;

fn solve(size: &str, scramble: &str) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", "--size", size, "--scramble", scramble])
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/moved-centres-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn assert_refused(size: &str, scramble: &str) {
    let (code, out, err) = solve(size, scramble);
    assert_eq!(code, Some(1), "{size}: {err}");
    assert!(out.is_empty(), "{size}: {out}");
    assert!(
        err.lines().any(|l| l
            == "error: the face centers are not on their own faces (a middle-slice turn moved them); this solver needs the centers of each face in place"),
        "{size}: {err}"
    );
    assert!(
        err.lines()
            .any(|l| l == "hint: turn the cube so the centers match, or undo the slice turn"),
        "{size}: {err}"
    );
    assert!(!err.contains("impossible"), "{size}: {err}");
}

#[test]
fn moved_centres_of_the_3x3x3_are_an_error() {
    assert_refused("3", "M");
}

#[test]
fn moved_centres_of_the_5x5x5_are_an_error() {
    // the generator of the pattern centres-cycled
    assert_refused("5", "2-4Uw 2-4Rw' 2-4Uw' 2-4Rw");
}

#[test]
fn moved_centres_of_the_7x7x7_are_an_error() {
    assert_refused("7", "2-6Rw");
}
