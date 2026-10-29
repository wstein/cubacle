//! `solve --to centers` and its `X` text on a 4x4x4 to 7x7x7: only the centers
//! are constrained; edges and corners stay wherever the center moves put them.

use cubacle::notation::{named_subset, parse_expression, parse_partial};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind, Target};
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

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

fn centers(kind: PuzzleKind) -> Target {
    named_subset("centers", kind).unwrap().unwrap()
}

/// Solves `start` to `to` and checks the printed moves against the target.
fn solves(size: usize, scramble: &str, to: &str, target: &Target) -> (MoveSequence, String) {
    let kind = PuzzleKind::of_facelet_count(6 * size * size).unwrap();
    let start = cube(kind, scramble);
    let (ok, out, err) = cubacle(&["solve", "--size", &size.to_string(), scramble, "--to", to]);
    assert!(ok, "{size}: {err}");
    let moves = parse_expression(&out, kind).expect("printed moves");
    assert!(target.reached_by(&start, &moves), "{size}: {out}");
    // edges and corners are not asked for, so the solve is not a full one
    assert!(!start.apply(&moves).unwrap().is_solved(), "{size}");
    (moves, err)
}

#[test]
fn the_4x4_centers_are_solved_in_far_fewer_moves_than_the_cube() {
    let scramble = "Rw U2 F' 2D Lw2 B R' 2U F2 D' Lw B2 U' 2R F Dw2 L' U Fw 2L' B2 R U2 Dw'";
    let (moves, err) = solves(4, scramble, "centers", &centers(PuzzleKind::Cube4));
    assert!(moves.len() <= 20, "{moves}");
    assert!(
        err.contains("best found") || err.contains("shortest"),
        "{err}"
    );
    // the plain solve of the same cube takes more than twice as many
    let (ok, full, _) = cubacle(&["solve", "--size", "4", scramble]);
    assert!(ok);
    let full: MoveSequence = full.parse().unwrap();
    assert!(
        full.len() > 2 * moves.len(),
        "{} vs {}",
        full.len(),
        moves.len()
    );
}

#[test]
fn a_few_moves_away_the_answer_is_proved_shortest() {
    let start = cube(PuzzleKind::Cube4, "Rw 2U");
    let (ok, out, err) = cubacle(&["solve", "--size", "4", "Rw 2U", "--to", "centers"]);
    assert!(ok, "{err}");
    assert!(err.contains("shortest:"), "{err}");
    let moves: MoveSequence = out.parse().unwrap();
    assert!(moves.len() <= 2, "{out}");
    assert!(centers(PuzzleKind::Cube4).reached_by(&start, &moves));
}

#[test]
fn x_text_with_all_centers_given_is_the_same_as_the_name() {
    let kind = PuzzleKind::Cube4;
    let text = centers(kind).to_text().unwrap();
    let scramble = "Rw U2 F' 2D Lw2 B R' 2U";
    let (ok, by_name, _) = cubacle(&["solve", "--size", "4", scramble, "--to", "centers"]);
    assert!(ok);
    for to in [text.clone(), text.replace('X', "?")] {
        let (ok, by_text, err) = cubacle(&["solve", "--size", "4", scramble, "--to", &to]);
        assert!(ok, "{err}");
        assert_eq!(by_text, by_name);
    }
}

#[test]
fn a_rotated_color_assignment_is_reached_on_a_4x4() {
    let kind = PuzzleKind::Cube4;
    let turned = cube(kind, "Rw Lw'");
    let target = Target::masked_by(&turned, &centers(kind).care());
    let to = target.to_text().unwrap();
    assert!(parse_partial(&to, None).unwrap().is_some());
    let scramble = "Rw U2 F' 2D Lw2 B R' 2U F2";
    let start = cube(kind, scramble);
    let (ok, out, err) = cubacle(&["solve", "--size", "4", scramble, "--to", &to]);
    assert!(ok, "{err}");
    let moves = parse_expression(&out, kind).unwrap();
    assert!(target.reached_by(&start, &moves), "{out}");
}

#[test]
fn the_5x5_centers_are_solved() {
    let scramble = "R U2 F' 2D L2 B R' 2U F2 D' 2L B2 U' 2R F D2 L' U 2F 2B' R U2 D";
    let (moves, _) = solves(5, scramble, "centers", &centers(PuzzleKind::Cube5));
    assert!(moves.len() <= 70, "{moves}");
}

#[test]
fn the_6x6_and_7x7_centers_are_solved() {
    for size in [6usize, 7] {
        let kind = PuzzleKind::of_facelet_count(6 * size * size).unwrap();
        let scramble = "R U2 F' 2D L2 B R' 2U F2 D' 2L B2 U' 2R F 3D2 L' U 2F 3B' R U2 D 3R' 2U";
        solves(size, scramble, "centers", &centers(kind));
    }
}

#[test]
fn other_masks_and_sizes_are_refused_naming_the_supported_subset() {
    for (size, to, expect) in [
        (4usize, "cross", "3x3x3 subset"),
        (3, "centers", "4x4x4 to 7x7x7"),
        (2, "centers", "4x4x4 to 7x7x7"),
    ] {
        let (ok, _, err) = cubacle(&["solve", "--size", &size.to_string(), "R U", "--to", to]);
        assert!(!ok, "{size} {to}");
        assert!(err.contains(expect), "{size} {to}: {err}");
    }
    // one face of a 4x4x4 with its edges and corners: not the centers only
    let one_face = format!("{}{}", "U".repeat(16), "X".repeat(80));
    let (ok, _, err) = cubacle(&["solve", "--size", "4", "R U", "--to", &one_face]);
    assert!(!ok);
    assert!(err.contains("`centers`") && err.contains("12"), "{err}");
    // some centers open
    let mut chars: Vec<char> = centers(PuzzleKind::Cube4)
        .to_text()
        .unwrap()
        .chars()
        .collect();
    chars[5] = 'X';
    let partly: String = chars.into_iter().collect();
    let (ok, _, err) = cubacle(&["solve", "--size", "4", "R U", "--to", &partly]);
    assert!(!ok);
    assert!(err.contains("must all be specified"), "{err}");
    assert!(!err.contains("preparing"), "{err}");
}

#[test]
fn centers_that_differ_within_a_face_and_odd_cube_frames_are_refused() {
    let kind = PuzzleKind::Cube4;
    let mixed = Target::masked_by(&cube(kind, "2R"), &centers(kind).care());
    let (ok, _, err) = cubacle(&[
        "solve",
        "--size",
        "4",
        "R U",
        "--to",
        &mixed.to_text().unwrap(),
    ]);
    assert!(!ok);
    assert!(err.contains("different colors"), "{err}");
    // the face centers of a 5x5x5 never move: a target in another orientation is unreachable
    let kind = PuzzleKind::Cube5;
    let moved = Target::masked_by(&cube(kind, "x"), &centers(kind).care());
    let (ok, _, err) = cubacle(&[
        "solve",
        "--size",
        "5",
        "R U",
        "--to",
        &moved.to_text().unwrap(),
    ]);
    assert!(!ok);
    assert!(err.contains("unreachable"), "{err}");
}

#[test]
fn options_that_do_not_combine_are_refused_as_before() {
    let (ok, _, err) = cubacle(&["solve", "--size", "4", "R U", "--to", "centers", "--json"]);
    assert!(!ok);
    assert!(err.contains("--json"), "{err}");
}
