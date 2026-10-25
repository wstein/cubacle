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
fn now_icons_shows_should_be_and_is_now_then_the_way_back() {
    let (ok, plain, _) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW]);
    assert!(ok);
    let (ok, out, _) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    assert!(ok);
    assert!(!out.contains('\x1b'), "{out}");
    assert!(out.starts_with(&plain), "the plan text comes first");
    let should = out.find("Should be: ").expect("token line");
    let now = out.find("Is now: ").expect("token line");
    assert!(out.contains("Should be, after move 4"), "{out}");
    let step = out.find("move R1 of 3: D2").expect("step");
    assert!(should < now && now < step, "{out}");
    assert!(out.contains("\nShould be: Enl7Pqxeueao\nIs now: "), "{out}");
    assert!(out.contains("move R3 of 3: D'"), "{out}");
}

#[test]
fn now_icons_draws_the_rollback_then_the_rest_of_the_printed_solution() {
    let (ok, out, _) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    assert!(ok);
    let rollback = out.find("Correction (3 moves)").expect("rollback");
    let r1 = out.find("move R1 of 3: D2").expect("R1");
    let rest = out
        .find("Continue (moves 7-21 of the printed solution)")
        .expect("continue");
    let first = out.find("move 7 of 21: B2").expect("global number");
    let last = out.find("move 21 of 21: R'").expect("last");
    let end = out.find("End state: AAAAAAAAAAAA").expect("end state");
    let moves = out.find("\n\nMoves: ").expect("moves line");
    let order = [rollback, r1, rest, first, last, end, moves];
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{out}");
    assert!(out.contains("\nMoves: D2 B2 D' B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'\n"));
    let differ = out.find("Stickers that differ").expect("differ");
    assert!(differ < rollback);
}

/// The images above the line that counts the stickers that differ.
fn state_images(out: &str) -> usize {
    let end = out.find("Stickers that differ").expect("count line");
    out[..end].matches("a=T").count()
}

#[test]
fn should_be_and_is_now_share_one_image_when_they_fit() {
    let run = |extra: &[&str]| {
        let mut args = vec!["solve", "-s", ORIGINAL, "--now", NOW, "--icons"];
        args.extend_from_slice(&["--graphics", "kitty"]);
        args.extend_from_slice(extra);
        let (ok, out, _) = cubacle(&args);
        assert!(ok);
        out
    };
    let side = run(&[]);
    assert!(
        side.contains("Left: Should be, after move 4"),
        "{}",
        side.len()
    );
    let stacked = run(&["--columns", "1"]);
    assert!(!stacked.contains("Left: "));
    assert_eq!(state_images(&side), 1);
    assert_eq!(state_images(&stacked), 2);
}

#[test]
fn now_icons_of_a_fresh_choice_draws_the_fresh_solution() {
    let solution = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";
    let (ok, out, _) = cubacle(&[
        "solve",
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
fn now_fresh_icons_draws_the_solution_and_the_other_options_apply() {
    let (ok, out, _) = cubacle(&[
        "solve",
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
fn now_picture_options_need_icons() {
    let (ok, _, err) = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--columns", "2"]);
    assert!(!ok);
    assert!(err.contains("--icons"), "{err}");
}
