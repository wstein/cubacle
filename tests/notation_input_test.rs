//! WCA and SiGN input in every command that reads moves: layer ranges,
//! groups, commutators and conjugates.

use std::process::Command;

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
const NOW: &str = "FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL";
const SOLUTION: &str = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn facelets(size: &str, text: &str) -> String {
    let (ok, out, err) = cubacle(&["facelets", "--size", size, text]);
    assert!(ok, "{text}: {err}");
    out
}

#[test]
fn facelets_and_convert_read_ranges_groups_and_commutators() {
    for (size, short, plain) in [
        ("5", "2-3r", "3Rw R'"),
        ("5", "2-4l2", "4Lw2 L2"),
        ("6", "3-5u'", "5Uw' 2Uw"),
        ("5", "2-3Fw", "3Fw F'"),
        ("4", "[r: U]", "Rw U Rw'"),
        ("3", "[R, U]", "R U R' U'"),
        ("3", "(R U)2'", "U' R' U' R'"),
        ("7", "[2-3r U: f]2", "2-3r U f U' 3Rw R' 2-3r U f U' 3Rw R'"),
    ] {
        let plain = if size == "7" {
            // the same thing spelled out
            "3Rw R' U Fw U' R 3Rw' 3Rw R' U Fw U' R 3Rw'"
        } else {
            plain
        };
        assert_eq!(facelets(size, short), facelets(size, plain), "{short}");
        let (ok, out, err) = cubacle(&["convert", "--size", size, "--scramble", short]);
        assert!(ok, "{err}");
        assert_eq!(out, facelets(size, plain));
    }
}

#[test]
fn errors_point_at_the_character() {
    let (ok, _, err) = cubacle(&["facelets", "--size", "5", "R [U, F, D]"]);
    assert!(!ok);
    assert!(err.contains("character 7"), "{err}");
    let (ok, _, err) = cubacle(&["facelets", "--size", "3", "R 2-3r"]);
    assert!(!ok);
    assert!(
        err.contains("character 2") && err.contains("does not exist"),
        "{err}"
    );
}

#[test]
fn solve_reads_a_scramble_expression_and_writes_either_notation() {
    for notation in ["wca", "sign"] {
        let (ok, out, err) = cubacle(&[
            "solve",
            "-s",
            "[R: U2] (F D)2 [L, B]",
            "--notation",
            notation,
        ]);
        assert!(ok, "{err}");
        assert!(!out.is_empty());
    }
}

#[test]
fn recover_reads_an_expression_solution_and_takes_a_notation() {
    let args = |solution: &str, notation: &str| {
        cubacle(&[
            "recover",
            "-s",
            ORIGINAL,
            "--now",
            NOW,
            "--solution",
            solution,
            "--notation",
            notation,
            "--one-mistake",
            "--depth",
            "0",
        ])
    };
    let (ok, plain, err) = args(SOLUTION, "wca");
    assert!(ok, "{err}");
    let (ok, with_group, err) = args(
        "(B L' U2 B2) D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'",
        "wca",
    );
    assert!(ok, "{err}");
    assert_eq!(plain, with_group);
    let (ok, sign, err) = args(SOLUTION, "sign");
    assert!(ok, "{err}");
    assert_eq!(plain, sign, "3x3x3 moves read the same in both");
    let (ok, _, err) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--notation", "x"]);
    assert!(!ok && err.contains("wca, sign or sse"), "{err}");
    let (ok, _, err) = args("B L' (U2", "wca");
    assert!(
        !ok && err.contains("--solution") && err.contains("unbalanced"),
        "{err}"
    );
}

#[test]
fn recover_writes_wide_moves_in_sign_on_request() {
    // The solution of "Uw2 R" is "R' Uw2"; the cube is after its first step.
    let (ok, now, err) = cubacle(&["facelets", "--size", "4", "u2"]);
    assert!(ok, "{err}");
    let run = |notation: &str| {
        cubacle(&[
            "recover",
            "--size",
            "4",
            "-s",
            "Uw2 R",
            "--now",
            &now,
            "--solution",
            "R' u2",
            "--notation",
            notation,
        ])
    };
    let (ok, wca, err) = run("wca");
    assert!(ok, "{err}");
    assert!(wca.contains("Continue with move 2 (Uw2)"), "{wca}");
    let (ok, sign, err) = run("sign");
    assert!(ok, "{err}");
    assert!(sign.contains("Continue with move 2 (u2)"), "{sign}");
}

#[test]
fn moves_icons_read_ranges_and_expressions() {
    let (ok, one, err) = cubacle(&["moves", "--icons", "--size", "5", "--format", "text", "3Rw"]);
    assert!(ok, "{err}");
    let (ok, range, err) = cubacle(&[
        "moves", "--icons", "--size", "5", "--format", "text", "2-3r",
    ]);
    assert!(ok, "{err}");
    assert!(range.len() > one.len());
    let (ok, from, err) = cubacle(&[
        "moves", "--icons", "--size", "5", "--format", "text", "--from", "[r: U]", "R",
    ]);
    assert!(ok, "{err}");
    assert!(!from.is_empty());
    let (ok, _, err) = cubacle(&[
        "moves", "--icons", "--size", "3", "--format", "text", "2-3r",
    ]);
    assert!(!ok && err.contains("does not exist"), "{err}");
}
