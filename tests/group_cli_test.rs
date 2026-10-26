//! `solve --group phases` writes a dot between solver phases; the answer reads back
//! to the same moves. `moves --icons` draws groups.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
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

const SCRAMBLE: &str = "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";

#[test]
fn none_is_todays_output_and_phases_add_dots_only() {
    let (ok, plain, error) = cubacle(&["solve", "--size", "4", SCRAMBLE]);
    assert!(ok, "{error}");
    assert!(!plain.contains('·'));
    let (ok, same, _) = cubacle(&["solve", "--size", "4", "--group", "none", SCRAMBLE]);
    assert!(ok);
    assert_eq!(same, plain);
    let (ok, dotted, error) = cubacle(&["solve", "--size", "4", "--group", "phases", SCRAMBLE]);
    assert!(ok, "{error}");
    assert!(dotted.contains(" · "), "{dotted}");
    assert_eq!(dotted.replace(" · ", " "), plain);
}

#[test]
fn the_dotted_answer_reads_back_to_the_same_moves() {
    for notation in ["wca", "sign", "sse"] {
        let (_, plain, _) = cubacle(&["solve", "--size", "4", "--notation", notation, SCRAMBLE]);
        let (ok, dotted, error) = cubacle(&[
            "solve",
            "--size",
            "4",
            "--notation",
            notation,
            "--group",
            "phases",
            SCRAMBLE,
        ]);
        assert!(ok, "{error}");
        let (ok, flat, error) = cubacle(&["moves", "--size", "4", &dotted]);
        assert!(ok, "{dotted}: {error}");
        let (ok, want, error) = cubacle(&["moves", "--size", "4", &plain]);
        assert!(ok, "{error}");
        assert_eq!(flat, want, "{notation}");
    }
}

#[test]
fn ranges_never_merge_across_a_dot() {
    let (ok, packed, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "--optimize",
        "ranges",
        "--group",
        "phases",
        SCRAMBLE,
    ]);
    assert!(ok, "{error}");
    let (_, flat, _) = cubacle(&["moves", "--size", "4", &packed]);
    let (_, plain, _) = cubacle(&["solve", "--size", "4", SCRAMBLE]);
    let (_, want, _) = cubacle(&["moves", "--size", "4", &plain]);
    assert_eq!(flat, want);
}

#[test]
fn moves_icons_groups_text_fallback() {
    let (ok, out, error) = cubacle(&[
        "moves",
        "--icons",
        "--graphics",
        "plain",
        "R U · F' (D · L)",
    ]);
    assert!(ok, "{error}");
    assert!(out.starts_with("R U · F' · (D · L)"), "{out}");
    assert!(out.contains("group 1 of 4: R U"), "{out}");
    assert!(out.contains("group 3 of 4: (D"), "{out}");
    let (ok, out, _) = cubacle(&[
        "moves",
        "--icons",
        "--graphics",
        "plain",
        "--group",
        "none",
        "R U · F'",
    ]);
    assert!(ok);
    assert!(out.contains("move 1 of 3: R"), "{out}");
    let (ok, out, _) = cubacle(&["moves", "--icons", "--graphics", "plain", "R U"]);
    assert!(ok);
    assert!(out.contains("move 2 of 2: U"), "{out}");
}
