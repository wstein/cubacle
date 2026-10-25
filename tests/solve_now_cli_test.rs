use std::process::Command;

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
/// The original cube after `B L' U2 B2 U2 B2 D2`: move 5 (`D'`) was skipped.
const NOW: &str = "FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn now_prints_what_recover_prints() {
    for extra in [
        &[][..],
        &["--fresh"][..],
        &["--at-move", "7", "--sure-until", "3"][..],
        &["--icons", "--graphics", "plain"][..],
    ] {
        let mut recover = vec!["recover", "-s", ORIGINAL, "--now", NOW];
        let mut solve = vec!["solve", "-s", ORIGINAL, "--now", NOW];
        recover.extend_from_slice(extra);
        solve.extend_from_slice(extra);
        let (ok_r, out_r, err_r) = cubacle(&recover);
        let (ok_s, out_s, err_s) = cubacle(&solve);
        assert!(ok_r && ok_s, "{extra:?}");
        assert_eq!(out_r, out_s, "{extra:?}");
        assert_eq!(err_r, err_s, "{extra:?}");
    }
}

#[test]
fn the_recover_options_need_now() {
    for option in [
        &["--fresh"][..],
        &["--solution", "R"][..],
        &["--at-move", "3"][..],
    ] {
        let mut args = vec!["solve", "-s", ORIGINAL];
        args.extend_from_slice(option);
        let (ok, _, err) = cubacle(&args);
        assert!(!ok, "{option:?}");
        assert!(err.contains("--now"), "{err}");
    }
}

#[test]
fn now_refuses_the_options_of_a_plain_solve() {
    let (ok, _, err) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--fast"]);
    assert!(!ok);
    assert!(err.contains("cannot be used with '--fast'"), "{err}");
}

#[test]
fn depth_is_zero_to_five_with_now() {
    let (ok, _, err) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--depth", "9"]);
    assert!(!ok);
    assert!(
        err.contains("--depth must be 0 to 5 with --now, not 9"),
        "{err}"
    );
}
