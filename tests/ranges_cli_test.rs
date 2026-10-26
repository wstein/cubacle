//! `--optimize ranges` on the command line: opt-in, both counts reported.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never the user's real table cache
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

const SCRAMBLE: &str = "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";

#[test]
fn off_by_default_and_on_request() {
    let (ok, plain, error) = cubacle(&["solve", "--size", "4", SCRAMBLE]);
    assert!(ok, "{error}");
    assert!(!error.contains("compressed"), "{error}");

    let (ok, packed, error) = cubacle(&["solve", "--size", "4", "--optimize", "ranges", SCRAMBLE]);
    assert!(ok, "{error}");
    let count = |label: &str| -> usize {
        error
            .lines()
            .find_map(|line| line.strip_prefix(label))
            .and_then(|rest| rest.trim().parse().ok())
            .unwrap_or_else(|| panic!("no {label:?} in {error}"))
    };
    assert_eq!(count("moves "), plain.split_whitespace().count());
    assert_eq!(
        count("compressed moves "),
        packed.split_whitespace().count()
    );
    assert!(count("compressed moves ") <= count("moves "));
}

#[test]
fn only_ranges_is_known() {
    let (ok, _, error) = cubacle(&["solve", "--size", "4", "--optimize", "all", SCRAMBLE]);
    assert!(!ok);
    assert!(error.contains("ranges"), "{error}");
}
