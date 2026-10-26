//! The `patterns identify` subcommand: what the structure of a cube says.

use std::process::Command;

fn identify(args: &[&str]) -> (bool, String) {
    run("identify", args)
}

fn cycles(args: &[&str]) -> (bool, String) {
    run("cycles", args)
}

fn run(command: &str, args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("patterns")
        .arg(command)
        .args(args)
        // never touch the real home: the first run writes a global config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn a_solved_cube_has_every_symmetry() {
    let (ok, text) = identify(&["R R'"]);
    assert!(ok);
    assert!(
        text.contains("symmetry: 48 of 48 operations fix it (24 rotations, 24 mirrored)"),
        "{text}"
    );
}

#[test]
fn a_scrambled_cube_has_only_the_identity() {
    let (ok, text) = identify(&["R U R' U'"]);
    assert!(ok);
    assert!(text.contains("symmetry: 1 of 48"), "{text}");
}

#[test]
fn a_checkerboard_keeps_all_symmetries_up_to_colors() {
    let (ok, text) = identify(&["--size", "4", "-s", "R2 L2 U2 D2 F2 B2"]);
    assert!(ok);
    assert!(text.contains("symmetry: 48 of 48"), "{text}");
}

#[test]
fn a_bad_cube_is_an_error() {
    let (ok, _) = identify(&["not a cube"]);
    assert!(!ok);
}

#[test]
fn a_library_pattern_is_named() {
    let (ok, text) = identify(&["U2 D2 R2 L2 F2 B2"]);
    assert!(ok);
    assert!(
        text.contains("pattern: Checkerboard (checkerboard), exactly"),
        "{text}"
    );
    let (ok, text) = identify(&["--size", "5", "2U' 2R 2F2"]);
    assert!(ok);
    assert!(text.contains("pattern: no library match"), "{text}");
}

#[test]
fn a_turned_cube_is_still_the_pattern() {
    let (ok, text) = identify(&["x U2 D2 R2 L2 F2 B2"]);
    assert!(ok);
    assert!(text.contains("(checkerboard)"), "{text}");
}

#[test]
fn the_fingerprint_reads_the_pieces() {
    let (ok, text) = identify(&["U' R2 U' F2 D' R2 U' B F U2 L' B2 R2 F2 D' U F' U2 L' R'"]);
    assert!(ok);
    assert!(text.contains("fingerprint: C0t0 E0f12"), "{text}");
    assert!(
        text.contains("edges: 12 pieces, 0 displaced, 12 flipped"),
        "{text}"
    );
    let (ok, text) = identify(&["--size", "7", "-s", "R U F' 2R 3U Rw' Fw 4L2 3B"]);
    assert!(ok);
    assert!(text.contains("wings (depth 2): 24 pieces"), "{text}");
}

#[test]
fn cycles_of_moves_give_the_order() {
    let (ok, text) = cycles(&["R U"]);
    assert!(ok);
    assert!(
        text.contains("corners: (UBL UBR DBR DFR UFL) 0 +2 +1 +2 0; (UFR) +1; 2 in place"),
        "{text}"
    );
    assert!(text.contains("order: 105"), "{text}");
    let (ok, text) = cycles(&["--size", "4", "R U 2R' Fw"]);
    assert!(ok);
    assert!(text.contains("order without the centers: 60"), "{text}");
}

#[test]
fn cycles_of_colors_need_the_moves_for_an_order_on_big_cubes() {
    let (ok, text) = cycles(&["UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"]);
    assert!(ok);
    assert!(text.contains("order: 1"), "{text}");
    let (ok, text) = cycles(&["--size", "4", "-s", "R U 2R'"]);
    assert!(ok);
    assert!(text.contains("order: "), "{text}");
}

#[test]
fn cycles_of_a_bad_cube_is_an_error() {
    let (ok, _) = cycles(&["not a cube"]);
    assert!(!ok);
}

#[test]
fn subgroups_are_reported_for_a_3x3x3() {
    let (ok, text) = identify(&["U2 D2 R2 L2 F2 B2"]);
    assert!(ok);
    assert!(
        text.contains("in half-turn group <U2,D2,R2,L2,F2,B2>: yes"),
        "{text}"
    );
    assert!(text.contains("in edges only: yes"), "{text}");
    let (ok, text) = identify(&["R U"]);
    assert!(ok);
    assert!(text.contains("in <R,U>: possible"), "{text}");
    assert!(
        text.contains("in G1 domino group, U/D axis: no: a corner is twisted"),
        "{text}"
    );
}

#[test]
fn center_orbits_are_named_by_their_position() {
    let (ok, text) = identify(&["--size", "5", "-s", "R U 2R' 3Fw"]);
    assert!(ok);
    assert!(
        text.contains("  x-centers (corner side): 24 center stickers"),
        "{text}"
    );
    assert!(
        text.contains("  +-centers (edge side): 24 center stickers"),
        "{text}"
    );
    let code = text
        .lines()
        .find(|l| l.starts_with("fingerprint: "))
        .unwrap();
    assert!(code.contains(" X") && code.contains(" +"), "{code}");
    let (ok, text) = cycles(&["--size", "5", "R U 2R' 3Fw"]);
    assert!(ok);
    assert!(text.contains("x-centers (corner side):"), "{text}");
    assert!(text.contains("+-centers (edge side):"), "{text}");
}

#[test]
fn patternness_is_printed() {
    let (ok, text) = identify(&["R R'"]);
    assert!(ok);
    assert!(text.contains("patternness: 100 %"), "{text}");
    let (ok, text) = identify(&["--size", "5", "-s", "R U 2R' 3Fw 2U 3Dw' F"]);
    assert!(ok);
    let line = text
        .lines()
        .find(|l| l.starts_with("patternness: "))
        .unwrap();
    let value: u32 = line["patternness: ".len()..]
        .trim_end_matches(" %")
        .parse()
        .unwrap();
    assert!(value < 50, "{line}");
}

#[test]
fn the_closest_patterns_and_the_partial_match_are_named() {
    let (ok, text) = identify(&["R"]);
    assert!(ok);
    assert!(text.contains("closest: Solved cube "), "{text}");
    let moves = "F (R U')2 WF U' F R' D L F D2 F2 WD NB MD' TF2 TD2 TF' TL' TD' TR TF' TU MF' (TU TR')2 TF'";
    let (ok, text) = identify(&["--size", "5", moves]);
    assert!(ok);
    let line = text
        .lines()
        .find(|l| l.starts_with("partial: "))
        .expect(&text);
    assert!(
        line.contains("corners, edges, x-centers (corner side) match Solved cube exactly"),
        "{line}"
    );
    assert!(
        line.contains("wings (depth 1)") && line.contains("differ"),
        "{line}"
    );
}

#[test]
fn a_symmetric_cube_gets_its_symmetry_type_lines_appended() {
    let (ok, text) = identify(&["R2 L2"]);
    assert!(ok);
    assert!(text.contains("\nsymmetry: D4h (order 16)\n"), "{text}");
    assert!(
        text.contains("\nantisymmetry: D4h+1' (order 32, 16 symmetries and 16 antisymmetries)\n"),
        "{text}"
    );
    let old = text.find("symmetry: 16 of 48 operations fix it").unwrap();
    assert!(old < text.find("\nsymmetry: D4h").unwrap());
}

#[test]
fn an_asymmetric_cube_gets_no_type_lines() {
    let (ok, text) = identify(&["R U F' L D2 B R2 U'"]);
    assert!(ok);
    assert!(
        text.contains("symmetry: 1 of 48 operations fix it"),
        "{text}"
    );
    assert!(!text.contains("antisymmetry:"), "{text}");
    assert!(
        text.lines().all(|l| !l.starts_with("symmetry: C1")),
        "{text}"
    );
}

#[test]
fn a_big_cube_gets_a_symmetry_type_but_no_antisymmetry() {
    let (ok, text) = identify(&["--size", "5", "R2 L2"]);
    assert!(ok);
    assert!(text.contains("\nsymmetry: D4h (order 16)\n"), "{text}");
    assert!(!text.contains("antisymmetry:"), "{text}");
}

#[test]
fn a_2x2x2_gets_no_antisymmetry_and_no_panic() {
    let (ok, text) = identify(&["--size", "2", "R U"]);
    assert!(ok);
    assert!(!text.contains("antisymmetry:"), "{text}");
    let (ok, text) = identify(&["--size", "2", "R2 L2 U2 D2"]);
    assert!(ok);
    assert!(text.contains("\nsymmetry: Oh (order 48)\n"), "{text}");
    assert!(!text.contains("antisymmetry:"), "{text}");
}
