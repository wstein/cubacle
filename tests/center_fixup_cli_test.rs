//! `solve --to` with a target whose centers show different colors on one
//! face (slice moves, wide turns with slices): the center fix-up, and every
//! option that goes with a target.

use cubacle::notation::parse_expression;
use cubacle::puzzle::{reaches, Facelets, MoveSequence, PuzzleKind};
use std::process::Command;
use std::sync::Mutex;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    // CLI budgets include cold table preparation. Run children one at a time
    // so parallel tests do not turn the 1.5 s functional case into a CPU
    // contention benchmark. Budget and expiry assertions remain unchanged.
    static CHILD: Mutex<()> = Mutex::new(());
    let _child = CHILD.lock().expect("CLI test lock");
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

fn cube(size: usize, moves: &str) -> Facelets {
    let kind = PuzzleKind::of_size(size).unwrap();
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

const START: &str = "R U F' D2 L B 2R' U2";
const TARGET: &str = "2R 2U' 2F 2L2 2D";

/// Replays an answer written in WCA notation.
fn replays(size: usize, answer: &str) {
    let moves: MoveSequence = answer.parse().expect("the answer is move notation");
    assert!(
        reaches(&cube(size, START), &moves, &cube(size, TARGET)),
        "{answer}"
    );
}

fn solve4(extra: &[&str]) -> (bool, String, String) {
    let mut args = vec!["solve", "--size", "4", START, "--to", TARGET];
    args.extend_from_slice(extra);
    cubacle(&args)
}

#[test]
fn moves_reach_the_target_and_the_fix_up_is_the_last_phase() {
    let (ok, answer, error) = solve4(&["-v"]);
    assert!(ok, "{error}");
    replays(4, &answer);
    let phase = error
        .lines()
        .rfind(|line| line.starts_with("phase "))
        .expect("phase lines");
    assert!(phase.contains("center fix-up"), "{error}");
    assert!(error.contains("center fix-up: "), "{error}");
    assert!(error.contains("metrics: "), "{error}");
}

#[test]
fn facelets_reach_the_same_target() {
    let (ok, facelets, _) = cubacle(&["facelets", "--size", "4", TARGET]);
    assert!(ok);
    let (ok, answer, error) = solve4(&[]);
    assert!(ok, "{error}");
    let by_facelets = cubacle(&["solve", "--size", "4", START, "--to", &facelets]);
    assert!(by_facelets.0, "{}", by_facelets.2);
    replays(4, &by_facelets.1);
    replays(4, &answer);
}

#[test]
fn notations_write_the_same_route() {
    let (_, wca, _) = solve4(&[]);
    for name in ["sign", "sse", "wca"] {
        let (ok, answer, error) = solve4(&["--notation", name]);
        assert!(ok, "{name}: {error}");
        let moves = parse_expression(&answer, PuzzleKind::Cube4).expect(name);
        assert!(reaches(&cube(4, START), &moves, &cube(4, TARGET)));
        let again = parse_expression(&wca, PuzzleKind::Cube4).unwrap();
        assert_eq!(moves.len(), again.len(), "{name}");
    }
}

#[test]
fn groups_ranges_and_metrics_work_with_the_fix_up() {
    let (ok, answer, error) = solve4(&["--group", "phases,axis"]);
    assert!(ok, "{error}");
    assert!(answer.contains(" · "), "{answer}");
    let (ok, answer, error) = solve4(&["--optimize", "ranges"]);
    assert!(ok, "{error}");
    assert!(error.contains("compressed moves"), "{error}");
    let moves = parse_expression(&answer, PuzzleKind::Cube4).expect("ranges");
    assert!(reaches(&cube(4, START), &moves, &cube(4, TARGET)));
    assert!(error.contains("metrics: "), "{error}");
}

#[test]
fn icons_show_the_steps_to_the_target() {
    let (ok, out, error) = solve4(&["--icons", "--icon-size", "120"]);
    assert!(ok, "{error}");
    assert!(out.contains("Target state"), "{out}");
    assert!(out.contains("move 1 of "), "{out}");
}

#[test]
fn the_move_limit_covers_the_fix_up() {
    let (ok, _, error) = solve4(&["-m", "6"]);
    assert!(!ok);
    assert!(
        error.contains("no solution within the move limit"),
        "{error}"
    );
    let (ok, answer, error) = solve4(&["-m", "40"]);
    assert!(ok, "{error}");
    assert!(answer.split_whitespace().count() <= 40);
}

#[test]
fn a_time_limit_that_planning_uses_up_is_reported() {
    let (ok, _, error) = solve4(&["-b", "1"]);
    assert!(!ok);
    assert!(error.contains("time limit"), "{error}");
    let (ok, answer, error) = solve4(&["-b", "30000"]);
    assert!(ok, "{error}");
    replays(4, &answer);
}

#[test]
fn probes_and_improve_apply_to_the_solve_and_parallel_is_refused() {
    for extra in [&["--probes", "50"][..], &["--improve", "-b", "1500"][..]] {
        let (ok, answer, error) = solve4(extra);
        assert!(ok, "{extra:?}: {error}");
        replays(4, &answer);
    }
    let (ok, _, error) = solve4(&["--improve", "--parallel", "-b", "1500"]);
    assert!(!ok);
    assert!(
        error.contains("--parallel works on the 3x3x3 only"),
        "{error}"
    );
}

#[test]
fn a_move_set_cannot_be_combined_with_a_fix_up() {
    let (ok, _, error) = solve4(&["--moves", "htm"]);
    assert!(ok, "htm restricts nothing: {error}");
    let (ok, _, error) = solve4(&["--moves", "<R,U>"]);
    assert!(!ok);
    assert!(!error.is_empty());
}

#[test]
fn a_5x5_reaches_a_slice_target() {
    let (ok, answer, error) = cubacle(&["solve", "--size", "5", "R U F2", "--to", "2R 4U' Fw 2L2"]);
    assert!(ok, "{error}");
    let moves: MoveSequence = answer.parse().unwrap();
    assert!(reaches(
        &cube(5, "R U F2"),
        &moves,
        &cube(5, "2R 4U' Fw 2L2")
    ));
}

#[test]
fn a_target_with_the_wrong_center_colors_is_still_refused() {
    // two center stickers of different orbits swapped: no cube looks so
    let mut stickers = cube(5, "").stickers().to_vec();
    stickers.swap(6, 32);
    let facelets = Facelets::from_stickers(stickers).unwrap().to_string();
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "R", "--to", &facelets]);
    assert!(!ok);
    assert!(error.contains("impossible"), "{error}");
}

#[test]
fn a_5x5_target_built_with_the_middle_slice_is_reached_from_moves_and_from_facelets() {
    // the middle slice turns the face centers; the engine keeps them, so the
    // fix-up turns the middle slice too
    let target = "2R 2U' 3F";
    let (ok, facelets, _) = cubacle(&["facelets", "--size", "5", target]);
    assert!(ok);
    for to in [target, facelets.as_str()] {
        let (ok, answer, error) = cubacle(&["solve", "--size", "5", "R U", "--to", to]);
        assert!(ok, "{to}: {error}");
        let moves: MoveSequence = answer.parse().unwrap();
        assert!(reaches(&cube(5, "R U"), &moves, &cube(5, target)), "{to}");
    }
}
