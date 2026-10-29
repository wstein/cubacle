//! `solve --picture`, `--center-twist`, `--twist-target` and `--certificate`,
//! and `verify`: a 3x3x3 whose face centers carry a picture.
//!
//! Each answer is replayed apart from the engine: the stickers through the
//! facelet model, the twists through the center tracker.

use cubacle::notation::parse_expression;
use cubacle::puzzle::{track_moves, CenterTwist, Facelets, MoveSequence, PuzzleKind};
use std::io::Write;
use std::process::{Command, Stdio};

fn cubacle_in(args: &[&str], stdin: &str) -> (bool, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cubacle runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn cubacle(args: &[&str]) -> (bool, String, String) {
    cubacle_in(args, "")
}

fn moves_of(text: &str) -> MoveSequence {
    parse_expression(text, PuzzleKind::Cube3).expect("moves")
}

fn cube(text: &str) -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves_of(text))
        .unwrap()
}

/// The `key: value` lines of a certificate (the part of stdout after the moves line).
fn fields(stdout: &str) -> Vec<(String, String)> {
    stdout
        .lines()
        .skip(1)
        .filter_map(|line| line.split_once(": "))
        .map(|(k, v)| (k.to_owned(), v.trim().to_owned()))
        .collect()
}

fn get<'a>(fields: &'a [(String, String)], key: &str) -> &'a str {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .unwrap_or_else(|| panic!("no {key} line"))
}

/// The certificate text of a stdout (everything after the moves line).
fn certificate(stdout: &str) -> String {
    stdout
        .split_once('\n')
        .map(|(_, rest)| rest)
        .unwrap()
        .to_owned()
}

const SCRAMBLE: &str = "R U2 F' D L2 B";

#[test]
fn a_picture_cube_is_solved_and_the_answer_is_replayed_for_stickers_and_twists() {
    let (ok, out, err) = cubacle(&["solve", "-s", SCRAMBLE, "--picture", "--certificate"]);
    assert!(ok, "{err}");
    assert!(
        err.contains("center twist at the start: 2,1,3,1,2,1"),
        "{err}"
    );
    assert!(err.contains("shortest: proved"), "{err}");
    let answer = moves_of(out.lines().next().unwrap());
    // the stickers
    let start = cube(SCRAMBLE);
    assert!(start.apply(&answer).unwrap().is_solved());
    // the twists, from the twist of the scramble
    let twist = track_moves(CenterTwist::UPRIGHT, &moves_of(SCRAMBLE))
        .unwrap()
        .twist;
    let end = track_moves(twist, &answer).unwrap();
    assert!(end.frame.is_empty());
    assert_eq!(end.twist, CenterTwist::UPRIGHT);
    // no shorter answer is in the certificate's accounting, and it checks
    let f = fields(&out);
    assert_eq!(get(&f, "length"), answer.len().to_string());
    assert_eq!(get(&f, "start"), start.to_string());
    assert_eq!(get(&f, "start-twist"), twist.to_string());
    assert!(get(&f, "convention").starts_with("center-twist-1"));
    let (ok, report, err) = cubacle_in(&["verify", "-"], &certificate(&out));
    assert!(ok, "{err}");
    assert!(report.starts_with("valid:"), "{report}");
}

#[test]
fn a_changed_certificate_is_refused() {
    let (_, out, _) = cubacle(&["solve", "-s", SCRAMBLE, "--picture", "--certificate"]);
    let good = certificate(&out);
    let first = out.lines().next().unwrap().split(' ').next().unwrap();
    let other = if first == "U" { "D" } else { "U" };
    let changed_move = good.replacen(&format!("moves: {first}"), &format!("moves: {other}"), 1);
    let (ok, _, err) = cubacle_in(&["verify", "-"], &changed_move);
    assert!(!ok);
    assert!(err.contains("end"), "{err}");
    let changed_twist = good.replacen("start-twist: 2,1,3,1,2,1", "start-twist: 2,1,3,1,2,3", 1);
    let (ok, _, err) = cubacle_in(&["verify", "-"], &changed_twist);
    assert!(!ok);
    assert!(err.contains("end-twist"), "{err}");
    let (ok, _, err) = cubacle_in(&["verify", "-"], "hello");
    assert!(!ok);
    assert!(err.contains("not a certificate"), "{err}");
}

#[test]
fn a_cube_given_as_stickers_needs_its_twist() {
    let stickers = cube(SCRAMBLE).to_string();
    let (ok, _, err) = cubacle(&["solve", &stickers, "--picture"]);
    assert!(!ok);
    assert!(err.contains("--center-twist"), "{err}");
    let twist = track_moves(CenterTwist::UPRIGHT, &moves_of(SCRAMBLE))
        .unwrap()
        .twist
        .to_string();
    let (ok, from_stickers, err) =
        cubacle(&["solve", &stickers, "--picture", "--center-twist", &twist]);
    assert!(ok, "{err}");
    let (_, from_scramble, _) = cubacle(&["solve", "-s", SCRAMBLE, "--picture"]);
    assert_eq!(from_stickers, from_scramble);
    // the parity of the twist must be that of the cube
    let (ok, _, err) = cubacle(&[
        "solve",
        &stickers,
        "--picture",
        "--center-twist",
        "1,1,1,0,0,0",
    ]);
    assert!(!ok);
    assert!(err.contains("parity"), "{err}");
    let (ok, _, err) = cubacle(&["solve", &stickers, "--picture", "--center-twist", "1,2"]);
    assert!(!ok);
    assert!(err.contains("six numbers"), "{err}");
}

#[test]
fn a_rotation_in_the_scramble_is_a_zero_cost_frame_change() {
    let (ok, out, err) = cubacle(&["solve", "-s", "x R U", "--picture", "--certificate"]);
    assert!(ok, "{err}");
    assert!(err.contains("turned as a whole"), "{err}");
    let f = fields(&out);
    assert!(!get(&f, "frame").is_empty());
    // the answer needs no rotation and leaves the cube solved in the new frame
    let answer = moves_of(out.lines().next().unwrap());
    let start = Facelets::parse(get(&f, "start")).unwrap();
    assert!(start.apply(&answer).unwrap().is_solved());
    assert!(answer.len() <= 2);
    let (ok, _, err) = cubacle_in(&["verify", "-"], &certificate(&out));
    assert!(ok, "{err}");
}

#[test]
fn a_twist_target_can_ask_for_one_center_only() {
    let (ok, out, err) = cubacle(&["solve", "-s", "R", "--twist-target", "X,0,X,X,X,X"]);
    assert!(ok, "{err}");
    assert_eq!(out.trim(), "R'");
}

#[test]
fn a_free_orientation_turns_the_twist_condition_and_the_rotation_is_in_the_certificate() {
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
        "--certificate",
    ]);
    assert!(ok, "{err}");
    // no move: the rotation that brings an upright picture to F costs nothing
    assert_eq!(out.lines().next().unwrap(), "");
    let f = fields(&out);
    assert!(!get(&f, "final-rotation").is_empty());
    assert_eq!(get(&f, "length"), "0");
    let (ok, _, err) = cubacle_in(&["verify", "-"], &certificate(&out));
    assert!(ok, "{err}");
}

#[test]
fn a_long_scramble_is_not_proven_within_the_budget_and_the_certificate_says_so() {
    let long = "D R' U' F' U L D R' U2 L U2 F D2 L2 R' U2";
    let (ok, out, err) = cubacle(&[
        "solve",
        "-s",
        long,
        "--picture",
        "--certificate",
        "-b",
        "500",
    ]);
    assert!(!ok);
    assert!(
        err.contains("not proven: no solution has fewer than"),
        "{err}"
    );
    assert!(out.starts_with("cubacle-certificate 1"), "{out}");
    assert!(out.contains("result: not proven"), "{out}");
    assert!(out.contains("lower-bound:"), "{out}");
    let (ok, report, _) = cubacle_in(&["verify", "-"], &out);
    assert!(ok);
    assert!(report.contains("without an answer"), "{report}");
}

#[test]
fn the_options_are_checked() {
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--picture", "--to", &"X".repeat(54)]);
    assert!(!ok);
    assert!(err.contains("cannot be used"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--size", "2", "--picture"]);
    assert!(!ok);
    assert!(err.contains("3x3x3 only"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--twist-target", "0,1,2"]);
    assert!(!ok);
    assert!(err.contains("six items"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--center-twist", "1,0,0,0,0,0"]);
    assert!(!ok);
    assert!(err.contains("needs --picture or --twist-target"), "{err}");
    // a certificate needs an exact search
    let (ok, _, err) = cubacle(&["solve", "-s", "R", "--certificate"]);
    assert!(!ok);
    assert!(err.contains("exact search"), "{err}");
}

#[test]
fn without_the_options_nothing_changes() {
    let (ok, out, _) = cubacle(&["solve", "-s", SCRAMBLE]);
    assert!(ok);
    assert!(!out.contains("certificate"));
}
