//! A solve that reaches its time limit says so, prints nothing on standard
//! output and exits with status 3, on every size that has no quick answer.

use std::process::Command;

fn solve(size: &str, scramble: &str, extra: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", "--size", size, "--scramble", scramble])
        .args(extra)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

const SCRAMBLES: [(&str, &str); 4] = [
    ("4", "R U2 Fw' L2 Uw B' D Rw2 F U' Bw L' D2 R2 Uw F2 B"),
    ("5", "R U2 Fw' L2 Uw B' D Rw2 F U' 2Bw L' D2 R2 Uw F2 B"),
    ("6", "R U2 Fw' L2 Uw B' D Rw2 F U' 3Bw L' D2 R2 3Uw F2 B"),
    (
        "7",
        "R U2 Fw' L2 Uw B' D Rw2 F U' 3Bw L' D2 R2 3Uw F2 3Rw B",
    ),
];

#[test]
fn a_tiny_limit_ends_the_solve_with_status_3() {
    for (size, scramble) in SCRAMBLES {
        let (code, out, err) = solve(size, scramble, &["--max-time", "1"]);
        assert_eq!(code, Some(3), "{size}: {err}");
        assert!(out.is_empty(), "{size}: {out}");
        let want = format!(
            "error: time limit of 0.001 s reached on the {size}x{size}x{size} (raise it with --max-time MS)"
        );
        assert!(err.lines().any(|l| l.starts_with(&want)), "{size}: {err}");
    }
}

#[test]
fn the_budget_option_ends_the_solve_the_same_way() {
    let (size, scramble) = SCRAMBLES[3];
    let (code, out, err) = solve(size, scramble, &["-b", "1"]);
    assert_eq!(code, Some(3), "{err}");
    assert!(out.is_empty(), "{out}");
    assert!(err.contains("time limit of 0.001 s"), "{err}");
}

#[test]
fn a_normal_solve_is_not_limited_and_no_limit_gives_the_same_answer() {
    for (size, scramble) in SCRAMBLES {
        let (code, out, err) = solve(size, scramble, &[]);
        assert_eq!(code, Some(0), "{size}: {err}");
        assert!(!out.is_empty(), "{size}");
        let (code, with, _) = solve(size, scramble, &["--max-time", "0"]);
        assert_eq!(code, Some(0), "{size}");
        assert_eq!(out, with, "{size}");
    }
}

#[test]
fn a_limit_cannot_be_combined_with_a_budget() {
    let (code, _, err) = solve("5", "R U", &["--max-time", "5", "-b", "5"]);
    assert_eq!(code, Some(2), "{err}");
}
