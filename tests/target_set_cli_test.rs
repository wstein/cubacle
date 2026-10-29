//! `solve --optimal --to` with open stickers, and `--any-orientation`: a proven
//! shortest answer to the whole set of states a target allows.

use cubacle::notation::{parse_expression, parse_partial};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

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

fn cube(moves: &str) -> Facelets {
    let moves = parse_expression(moves, PuzzleKind::Cube3).expect("moves");
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves)
        .expect("moves fit")
}

/// The two bottom layers: the D face and the lower two rows of the side faces.
const F2L: &str = "XXXXXXXXXXXXRRRRRRXXXFFFFFFDDDDDDDDDXXXLLLLLLXXXBBBBBB";
/// The ring of eight stickers around the U center, the center open.
const U_RING: &str = "UUUUXUUUUXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
/// The whole F face, as a face target.
const F_FACE: &str = "XXXXXXXXXXXXXXXXXXFFFFFFFFFXXXXXXXXXXXXXXXXXXXXXXXXXXX";
const SCRAMBLE: &str = "R U2 F' D L2 B R' U F2 L D2";

#[test]
fn a_face_is_reached_in_the_fewest_moves_and_the_answer_is_replayed() {
    let start = cube(SCRAMBLE);
    let (ok, out, err) = cubacle(&["solve", &start.to_string(), "--optimal", "--to", F_FACE]);
    assert!(ok, "{err}");
    assert!(err.contains("shortest: proved"), "{err}");
    let moves: MoveSequence = out.parse().unwrap();
    let target = parse_partial(F_FACE, None).unwrap().unwrap();
    assert!(target.reached_by(&start, &moves));
    // nothing shorter reaches it: the answer minus its last move does not
    let shorter: MoveSequence = MoveSequence::new(moves.moves()[..moves.len() - 1].to_vec());
    assert!(!target.reached_by(&start, &shorter));
}

#[test]
fn f2l_is_a_target_set_with_many_completions_and_is_proved_shortest() {
    let start = cube(SCRAMBLE);
    let (ok, out, err) = cubacle(&["solve", &start.to_string(), "--optimal", "--to", F2L]);
    assert!(ok, "{err}");
    let moves: MoveSequence = out.parse().unwrap();
    assert!(parse_partial(F2L, None)
        .unwrap()
        .unwrap()
        .reached_by(&start, &moves));
    // without --optimal the same target is still refused, as before
    let (ok, _, err) = cubacle(&["solve", &start.to_string(), "--to", F2L]);
    assert!(!ok);
    assert!(err.contains("unsupported"), "{err}");
}

#[test]
fn any_orientation_never_costs_more_and_the_answer_holds_in_some_orientation() {
    let start = cube(SCRAMBLE);
    let fixed = cubacle(&["solve", &start.to_string(), "--optimal", "--to", U_RING]);
    let free = cubacle(&[
        "solve",
        &start.to_string(),
        "--any-orientation",
        "--to",
        U_RING,
    ]);
    assert!(fixed.0 && free.0, "{} {}", fixed.2, free.2);
    let (fixed, free): (MoveSequence, MoveSequence) =
        (fixed.1.parse().unwrap(), free.1.parse().unwrap());
    assert!(free.len() <= fixed.len());
    let target = parse_partial(U_RING, None).unwrap().unwrap();
    assert!(target.reached_by_any_rotation(&start, &free));
}

#[test]
fn a_limit_ends_in_an_error_that_names_the_proven_depth_never_a_longer_answer() {
    let start = cube("R U2 F' D L2 B R' U F2 L D2 B' R2 U' F L' D B2 R U");
    let (ok, out, err) = cubacle(&[
        "solve",
        &start.to_string(),
        "--optimal",
        "-b",
        "50",
        "--max-memory",
        "0",
        "--to",
        F2L,
    ]);
    assert!(!ok && out.is_empty(), "{out}");
    assert!(
        err.contains("not proven: no solution has fewer than"),
        "{err}"
    );
    let (ok, out, err) = cubacle(&[
        "solve",
        &start.to_string(),
        "--optimal",
        "-m",
        "6",
        "--max-memory",
        "100",
        "--to",
        F2L,
    ]);
    assert!(!ok && out.is_empty(), "{out}");
    assert!(err.contains("no solution within 6 moves"), "{err}");
}

#[test]
fn the_options_are_refused_where_they_make_no_sense() {
    let start = cube("R U").to_string();
    let (ok, _, err) = cubacle(&["solve", &start, "--optimal", "--to", "R U2"]);
    assert!(!ok);
    assert!(err.contains("open stickers"), "{err}");
    let (ok, _, err) = cubacle(&["solve", "--size", "4", "--optimal", "--to", "centers"]);
    assert!(!ok);
    assert!(err.contains("3x3x3 only"), "{err}");
    let (ok, _, err) = cubacle(&["solve", &start, "--any-orientation"]);
    assert!(!ok);
    assert!(err.contains("--to"), "{err}");
}
