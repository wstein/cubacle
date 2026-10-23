use std::process::Command;

const LONG: &str = "Uw Lw F Lw2 D R' D B D' B' Rw2 Fw' L Fw Uw' R' Dw Rw D2 L2 B' Rw2 B R D Fw Lw2 F' R Uw F D Fw' L2 B' Lw2 Fw L' B2 D' F Rw' Uw' L2 F2 U' Fw2 D' Rw' F D Lw' Dw' B' R U Bw2 D2 F Lw'";

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

fn moves(args: &[&str]) -> String {
    let mut all = vec!["moves"];
    all.extend_from_slice(args);
    let (ok, out, err) = cubacle(&all);
    assert!(ok, "{args:?}: {err}");
    out
}

#[test]
fn groups_commutators_and_repeats_are_expanded() {
    assert_eq!(moves(&["--size", "7", "(R2 F2 L2)*2"]), "R2 F2 L2 R2 F2 L2");
    assert_eq!(moves(&["[R: U]"]), "R U R'");
    assert_eq!(moves(&["[R, U]"]), "R U R' U'");
    assert_eq!(moves(&["(R", "U)*2"]), "R U R U", "words need no quoting");
}

#[test]
fn slices_and_rotations_are_expanded_into_layer_moves() {
    assert_eq!(moves(&["M"]), "2L");
    assert_eq!(moves(&["x"]), "Rw L'");
    let slice = moves(&["--size", "5", "E", "S"]);
    assert!(!slice.contains('E') && !slice.contains('S'), "{slice}");
}

#[test]
fn sign_ranges_and_both_notations_are_read() {
    assert_eq!(
        moves(&["--size", "5", "2-3r"]),
        moves(&["--size", "5", "2-3Rw"])
    );
    assert_eq!(moves(&["--size", "5", "--notation", "sign", "3Rw"]), "3r");
    assert_eq!(moves(&["--size", "4", "--notation", "sign", "Rw"]), "r");
}

#[test]
fn errors_name_the_problem_and_its_position() {
    let (ok, out, err) = cubacle(&["moves", "R", "3Rw"]);
    assert!(!ok && out.is_empty());
    assert!(
        err.contains("character 2") && err.contains("does not exist"),
        "{err}"
    );
    let (ok, _, err) = cubacle(&["moves", "(R U"]);
    assert!(
        !ok && err.contains("unbalanced") && err.contains("character"),
        "{err}"
    );
    let (ok, _, err) = cubacle(&["moves", "--size", "9", "R"]);
    assert!(!ok && err.contains("size"), "{err}");
    let (ok, _, err) = cubacle(&["moves", "(R)*100001"]);
    assert!(!ok && err.contains("100000"), "{err}");
}

#[test]
fn the_long_sequence_round_trips() {
    for size in ["3", "4", "5", "7"] {
        assert_eq!(moves(&["--size", size, LONG]), LONG);
    }
    let (ok, _, err) = cubacle(&["moves", "--size", "2", LONG]);
    assert!(!ok && err.contains("does not exist"), "{err}");
}

#[test]
fn wca_and_sign_round_trip_on_every_size() {
    for size in ["2", "3", "4", "5", "6", "7"] {
        let (ok, scramble, err) = cubacle(&["scramble", "--size", size, "-n", "60"]);
        assert!(ok, "{err}");
        let sign = moves(&["--size", size, "--notation", "sign", &scramble]);
        let back = moves(&["--size", size, "--notation", "wca", &sign]);
        assert_eq!(back, scramble, "size {size}: {sign}");
        assert_eq!(moves(&["--size", size, &scramble]), scramble);
    }
}
