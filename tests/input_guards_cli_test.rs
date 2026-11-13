//! Option values that used to hang or panic are refused with a message, and
//! a time limit in any solve mode ends with the documented exit status 3.

use std::process::Command;

fn cubacle(args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .output()
        .expect("cubacle runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn an_icon_size_outside_one_to_1024_is_refused_without_a_panic() {
    for size in ["0", "1025", "99999999"] {
        for command in [
            &["moves", "R U"][..],
            &["solve", "-s", "R U"],
            &["patterns", "list"],
        ] {
            let mut args = command.to_vec();
            args.extend(["--icons", "--icon-size", size]);
            let (code, _, err) = cubacle(&args);
            assert_eq!(code, Some(2), "{args:?}: {err}");
            assert!(err.contains("--icon-size"), "{args:?}: {err}");
            assert!(!err.contains("panicked"), "{args:?}: {err}");
        }
    }
    let (code, _, err) = cubacle(&[
        "moves",
        "R U",
        "--icons",
        "--icon-size",
        "1",
        "--format",
        "text",
    ]);
    assert_eq!(code, Some(0), "{err}");
}

const SCRAMBLE: &str = "R U2 F' D L2 B R' U F2 D' L B2 U' R2 F D2 L' U B' R";

#[test]
fn a_time_limit_ends_the_search_for_all_solutions_with_status_3() {
    for mode in [&["--all"][..], &["--count", "5"]] {
        let mut args = vec!["solve", "-s", SCRAMBLE, "--max-time", "1"];
        args.extend(mode);
        let (code, _, err) = cubacle(&args);
        assert_eq!(code, Some(3), "{args:?}: {err}");
        assert!(err.contains("time budget"), "{args:?}: {err}");
    }
}

#[test]
fn a_time_limit_ends_a_target_with_open_stickers_with_status_3() {
    let target = "UUUUUUUUXXRRRRRRRRFFXFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let (code, _, err) = cubacle(&["solve", "-s", SCRAMBLE, "--to", target, "--max-time", "1"]);
    assert_eq!(code, Some(3), "{err}");
}
