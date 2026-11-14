//! `solve --upright-centers`, `--center-twist` and `--twist-target`: a 3x3x3
//! whose face centers carry a picture.
//!
//! Each answer is replayed apart from the engine: the stickers through the
//! facelet model, the twists through the center tracker.

use cubacle::notation::parse_expression;
use cubacle::puzzle::{CenterTwists, Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn moves_of(text: &str) -> MoveSequence {
    parse_expression(text, PuzzleKind::Cube3).expect("moves")
}

fn cube(text: &str) -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves_of(text))
        .unwrap()
}

fn upright() -> CenterTwists {
    CenterTwists::untwisted(PuzzleKind::Cube3).unwrap()
}

const SCRAMBLE: &str = "R U2 F' D L2 B";

#[test]
fn a_picture_cube_is_solved_and_the_answer_is_replayed_for_stickers_and_twists() {
    let (ok, out, err) = cubacle(&["solve", "-s", SCRAMBLE, "--upright-centers"]);
    assert!(ok, "{err}");
    assert!(
        err.contains("center twist at the start: 2,1,3,1,2,1"),
        "{err}"
    );
    assert!(err.contains("shortest: proved"), "{err}");
    let answer = moves_of(out.lines().next().unwrap());
    // the stickers
    assert!(cube(SCRAMBLE).apply(&answer).unwrap().is_solved());
    // the twists, from the twist of the scramble
    let twist = upright().track(&moves_of(SCRAMBLE)).unwrap().twists;
    let end = twist.track(&answer).unwrap();
    assert!(end.frame.is_empty());
    assert_eq!(end.twists, upright());
    assert!(
        err.contains("center twist at the end: 0,0,0,0,0,0"),
        "{err}"
    );
}

#[test]
fn a_cube_given_as_stickers_needs_its_twist() {
    let stickers = cube(SCRAMBLE).to_string();
    let (ok, _, err) = cubacle(&["solve", &stickers, "--upright-centers"]);
    assert!(!ok);
    assert!(err.contains("--center-twist"), "{err}");
    let twist = upright().track(&moves_of(SCRAMBLE)).unwrap().twists;
    let twist = twist.to_string();
    let (ok, from_stickers, err) = cubacle(&[
        "solve",
        &stickers,
        "--upright-centers",
        "--center-twist",
        &twist,
    ]);
    assert!(ok, "{err}");
    let (_, from_scramble, _) = cubacle(&["solve", "-s", SCRAMBLE, "--upright-centers"]);
    assert_eq!(from_stickers, from_scramble);
    // the parity of the twist must be that of the cube
    let (ok, _, err) = cubacle(&[
        "solve",
        &stickers,
        "--upright-centers",
        "--center-twist",
        "1,1,1,0,0,0",
    ]);
    assert!(!ok);
    assert!(err.contains("parity"), "{err}");
    let (ok, _, err) = cubacle(&[
        "solve",
        &stickers,
        "--upright-centers",
        "--center-twist",
        "1,2",
    ]);
    assert!(!ok);
    assert!(err.contains("six numbers"), "{err}");
}

#[test]
fn a_rotation_in_the_scramble_is_a_zero_cost_frame_change() {
    let (ok, out, err) = cubacle(&["solve", "-s", "x R U", "--upright-centers"]);
    assert!(ok, "{err}");
    assert!(err.contains("turned as a whole"), "{err}");
    let answer = moves_of(out.lines().next().unwrap());
    assert!(answer.len() <= 2, "{answer}");
}

#[test]
fn a_twist_target_can_ask_for_one_center_only() {
    let (ok, out, err) = cubacle(&["solve", "-s", "R", "--twist-target", "X,0,X,X,X,X"]);
    assert!(ok, "{err}");
    assert_eq!(out.trim(), "R'");
}

#[test]
fn a_free_orientation_turns_the_twist_condition_with_the_cube() {
    let open = "X".repeat(54);
    let (ok, out, err) = cubacle(&[
        "solve",
        "-s",
        "F",
        "--to",
        &open,
        "--twist-target",
        "X,X,0,X,X,X",
        "--any-orientation",
    ]);
    assert!(ok, "{err}");
    // no move: the rotation that brings an upright picture to F costs nothing
    assert_eq!(out.lines().next().unwrap(), "");
}

#[test]
fn a_long_scramble_is_not_proven_within_the_budget() {
    let long = "D R' U' F' U L D R' U2 L U2 F D2 L2 R' U2";
    let (ok, out, err) = cubacle(&["solve", "-s", long, "--upright-centers", "-b", "500"]);
    assert!(!ok);
    assert!(
        err.contains("not proven: no solution has fewer than"),
        "{err}"
    );
    assert!(out.is_empty(), "{out}");
}

#[test]
fn the_options_are_checked() {
    let (ok, _, err) = cubacle(&[
        "solve",
        "-s",
        "R",
        "--upright-centers",
        "--to",
        &"X".repeat(54),
    ]);
    assert!(!ok);
    assert!(err.contains("cannot be used"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--size", "2", "--upright-centers"]);
    assert!(!ok);
    assert!(err.contains("3x3x3 only"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--twist-target", "0,1,2"]);
    assert!(!ok);
    assert!(err.contains("six items"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--center-twist", "1,0,0,0,0,0"]);
    assert!(!ok);
    assert!(
        err.contains("needs --upright-centers or --twist-target"),
        "{err}"
    );
    // the old name is gone, without an alias
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--picture"]);
    assert!(!ok);
    assert!(err.contains("--picture"), "{err}");
}

#[test]
fn without_the_options_nothing_changes() {
    let (ok, out, err) = cubacle(&["solve", "-s", SCRAMBLE]);
    assert!(ok, "{err}");
    assert!(!err.contains("center twist"), "{err}");
    assert!(!out.is_empty());
}

#[test]
fn every_center_is_pinned_by_the_upright_target() {
    // solved stickers with one picture turned half: only the pictures can say so,
    // and the answer is long (12 moves), never empty
    let stickers = Facelets::solved(PuzzleKind::Cube3).to_string();
    for face in 0..6 {
        let mut twist = [0u8; 6];
        twist[face] = 2;
        let text = twist.map(|t| t.to_string()).join(",");
        let (ok, out, err) = cubacle(&[
            "solve",
            &stickers,
            "--upright-centers",
            "--center-twist",
            &text,
        ]);
        assert!(ok, "{text}: {err}");
        let answer = moves_of(out.lines().next().unwrap());
        assert!(answer.len() > 1, "{text}: {answer}");
        // the stickers end solved and every picture upright again
        assert!(Facelets::solved(PuzzleKind::Cube3)
            .apply(&answer)
            .unwrap()
            .is_solved());
        let start = CenterTwists::new(PuzzleKind::Cube3, twist).unwrap();
        let end = start.track(&answer).unwrap();
        assert!(end.frame.is_empty());
        assert_eq!(end.twists, upright(), "{text}: {answer}");
    }
}

#[test]
fn a_given_twist_and_a_scramble_exclude_each_other() {
    let (ok, _, err) = cubacle(&[
        "solve",
        "-s",
        "R",
        "--upright-centers",
        "--center-twist",
        "0,0,0,0,0,0",
    ]);
    assert!(!ok);
    assert!(err.contains("describes a cube given as stickers"), "{err}");
    assert!(!err.contains("center twist at the start"), "{err}");
}

#[test]
fn a_twist_target_that_asks_for_nothing_says_so() {
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--twist-target", "X,X,X,X,X,X"]);
    assert!(!ok);
    assert!(err.contains("--twist-target: every center is X"), "{err}");
    assert!(!err.contains("--center-twist needs"), "{err}");
}

#[test]
fn options_that_do_not_combine_are_refused_before_anything_else() {
    for option in ["--json", "--icons", "--improve"] {
        let (ok, _, err) = cubacle(&["solve", "-s", "R", "--upright-centers", option]);
        assert!(!ok, "{option}");
        assert!(
            err.contains(&format!("{option} does not work with --upright-centers")),
            "{option}: {err}"
        );
        assert!(!err.contains("open stickers"), "{option}: {err}");
        assert!(
            !err.contains("center twist at the start"),
            "{option}: {err}"
        );
    }
}

#[test]
fn a_center_twist_is_refused_with_now() {
    let (ok, _, err) = cubacle(&[
        "solve",
        "-s",
        "R",
        "--now",
        "R U",
        "--center-twist",
        "0,0,0,0,0,0",
    ]);
    assert!(!ok);
    assert!(err.contains("cannot be used"), "{err}");
}

#[test]
fn a_cube_that_is_not_a_3x3x3_is_refused_with_the_3x3x3_message() {
    let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--size", "2", "--upright-centers"]);
    assert!(!ok);
    assert!(
        err.contains("--upright-centers, --twist-target and --center-twist work on the 3x3x3 only, not the 2x2x2"),
        "{err}"
    );
}
