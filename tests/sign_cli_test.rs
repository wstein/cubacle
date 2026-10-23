//! `--notation sign` and SiGN input on the command line.

use cubacle::notation::parse_moves;
use cubacle::puzzle::{Facelets, PuzzleKind};
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

#[test]
fn facelets_read_sign_and_wca_alike() {
    let wca = cubacle(&["facelets", "--size", "5", "Rw 2R Fw' 3Uw2 3L"]);
    let sign = cubacle(&["facelets", "--size", "5", "r 2R f' 3u2 3L"]);
    assert!(wca.0 && sign.0, "{wca:?} {sign:?}");
    assert_eq!(wca.1, sign.1);
}

#[test]
fn a_bad_sign_token_is_named() {
    let (ok, _, error) = cubacle(&["facelets", "--size", "5", "R 2-3r"]);
    assert!(!ok);
    assert!(error.contains("2-3r") && error.contains("range"), "{error}");
    let (ok, _, error) = cubacle(&["facelets", "--size", "5", "rw"]);
    assert!(!ok);
    assert!(error.contains("rw") && error.contains("wide"), "{error}");
}

#[test]
fn notation_must_be_wca_or_sign() {
    let (ok, _, error) = cubacle(&["scramble", "--notation", "ruwix"]);
    assert!(!ok);
    assert!(error.contains("wca or sign"), "{error}");
}

#[test]
fn scramble_is_written_in_sign() {
    let (ok, text, error) = cubacle(&["scramble", "--size", "7", "-n", "60", "--notation", "sign"]);
    assert!(ok, "{error}");
    assert!(!text.contains('w'), "{text}");
    assert_eq!(parse_moves(&text).unwrap().len(), 60);
}

#[test]
fn a_3x3_solution_is_the_same_in_both_notations() {
    let args = ["solve", "--scramble", "R U F' D2 L B"];
    let (_, wca, _) = cubacle(&args);
    let (ok, sign, error) = cubacle(&[&args[..], &["--notation", "sign"]].concat());
    assert!(ok, "{error}");
    assert_eq!(wca, sign);
}

#[test]
fn a_4x4_solution_in_sign_replays_and_has_no_w() {
    let scramble_sign = "R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l";
    let (ok, wca, error) = cubacle(&["solve", "--size", "4", "-s", scramble_sign]);
    assert!(ok, "{error}");
    let (ok, sign, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "-s",
        scramble_sign,
        "--notation",
        "sign",
    ]);
    assert!(ok, "{error}");
    assert!(!sign.contains('w'), "{sign}");
    assert_eq!(parse_moves(&sign).unwrap(), parse_moves(&wca).unwrap());
    let cube = Facelets::solved(PuzzleKind::Cube4)
        .apply(&parse_moves(scramble_sign).unwrap())
        .unwrap();
    assert!(cube
        .apply(&parse_moves(&sign).unwrap())
        .unwrap()
        .is_solved());
}

#[test]
fn icon_takes_sign_moves_and_keeps_its_labels() {
    let (ok, sign, error) = cubacle(&["icon", "--cube", "4", "--format", "svg", "r"]);
    assert!(ok, "{error}");
    let (ok, wca, _) = cubacle(&["icon", "--cube", "4", "--format", "svg", "Rw"]);
    assert!(ok);
    assert_eq!(sign, wca);
}
