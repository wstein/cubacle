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
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn recover_icons_shows_should_be_and_is_now_then_the_way_back() {
    let (ok, plain, _) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW]);
    assert!(ok);
    let (ok, out, _) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    assert!(ok);
    assert!(!out.contains('\x1b'), "{out}");
    assert!(out.starts_with(&plain), "the plan text comes first");
    let should = out.find("Should be, after move 4").expect("label");
    let now = out.find("Is now").expect("label");
    let step = out.find("move 1 of 3: D2").expect("step");
    assert!(should < now && now < step, "{out}");
    assert_eq!(out.matches("token: ").count(), 2, "{out}");
    assert!(out.contains("move 3 of 3: D'"), "{out}");
}

#[test]
fn recover_icons_of_a_fresh_choice_draws_the_fresh_solution() {
    let solution = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";
    let (ok, out, _) = cubacle(&[
        "recover",
        "-s",
        ORIGINAL,
        "--now",
        NOW,
        "--solution",
        solution,
        "--depth",
        "0",
        "--one-mistake",
        "--icons",
    ]);
    assert!(ok);
    assert!(out.contains("Recommended: solve fresh"), "{out}");
    assert!(out.contains("move 18 of 18"), "{out}");
    assert!(out.contains("End state: "), "{out}");
}

#[test]
fn recover_fresh_icons_draws_the_solution_and_the_other_options_apply() {
    let (ok, out, _) = cubacle(&[
        "recover",
        "-s",
        ORIGINAL,
        "--now",
        NOW,
        "--fresh",
        "--icons",
        "--icon-size",
        "64",
        "--background",
        "light",
        "--columns",
        "2",
    ]);
    assert!(ok);
    assert!(out.contains("move 18 of 18"), "{out}");
    assert!(!out.contains("Should be"), "{out}");
}

#[test]
fn recover_picture_options_need_icons() {
    let (ok, _, err) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--columns", "2"]);
    assert!(!ok);
    assert!(err.contains("--icons"), "{err}");
}
