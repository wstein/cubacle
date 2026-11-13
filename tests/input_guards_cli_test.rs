//! Option values that used to panic or hang are refused with a message.

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
