//! Tests of the center twist as a search coordinate (picture cubes).
//!
//! Every answer is replayed twice, apart from the engine: the stickers through
//! the facelet model and the twists through the center tracker.

use super::super::solver::Min2PhaseSolver;
use crate::puzzle::{
    parse_twist_masks, track_moves, CenterTwist, Face, Facelets, Failure, MoveSequence, PuzzleKind,
    SolveOptions, SolveResult, Solver, Target,
};

fn seq(text: &str) -> MoveSequence {
    text.parse().unwrap()
}

fn solved() -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
}

/// The picture cube: solved stickers with all six centers upright.
fn picture_target() -> Target {
    Target::complete(&solved()).with_twist([1; 6])
}

fn run(
    scramble: &str,
    target: &Target,
    options: SolveOptions,
) -> (Facelets, CenterTwist, SolveResult) {
    let moves = seq(scramble);
    let start = solved().apply(&moves).unwrap();
    let twist = track_moves(CenterTwist::UPRIGHT, &moves).unwrap().twist;
    let result = Min2PhaseSolver::new().solve_to(&start, target, &options.with_start_twist(twist));
    (start, twist, result)
}

#[test]
fn a_short_scramble_is_undone_with_the_pictures_upright_again() {
    for scramble in ["R", "R U", "R U2 F'", "F2 D' L B2 U", "U R U' R' F2 D L'"] {
        let (start, twist, result) = run(scramble, &picture_target(), SolveOptions::default());
        let moves = result.outcome.clone().expect(scramble);
        assert!(result.message.unwrap().starts_with("shortest:"));
        assert!(moves.len() <= seq(scramble).len(), "{scramble}: {moves}");
        // stickers through the facelet model
        assert!(start.apply(&moves).unwrap().is_solved(), "{scramble}");
        // twists through the tracker, from the start twist
        let end = track_moves(twist, &moves).unwrap();
        assert!(end.frame.is_empty());
        assert_eq!(end.twist, CenterTwist::UPRIGHT, "{scramble}: {moves}");
    }
}

#[test]
fn a_condition_on_one_center_leaves_the_others_free() {
    // R turns the R picture a quarter; only the R center is asked to be upright
    let masks = parse_twist_masks("X,0,X,X,X,X").unwrap();
    let target = Target::complete(&solved()).with_twist(masks);
    let (start, twist, result) = run("R", &target, SolveOptions::default());
    let moves = result.outcome.clone().unwrap();
    assert_eq!(moves.to_string(), "R'");
    assert_eq!(track_moves(twist, &moves).unwrap().twist.get(Face::R), 0);
    assert!(start.apply(&moves).unwrap().is_solved());
    // two twists are accepted for the R picture, 0 or 2: the one move R2 does
    let masks = parse_twist_masks("X,0|2,X,X,X,X").unwrap();
    let target = Target::complete(&solved()).with_twist(masks);
    let (_, _, result) = run("R2", &target, SolveOptions::default());
    assert_eq!(result.outcome.unwrap().to_string(), "R2");
}

#[test]
fn a_twist_that_does_not_belong_to_the_cube_is_refused() {
    // the solved cube has an even corner permutation: an odd sum cannot be
    let start = solved();
    let options = SolveOptions::default().with_start_twist(CenterTwist::new([1, 0, 0, 0, 0, 0]));
    let result = Min2PhaseSolver::new().solve_to(&start, &picture_target(), &options);
    assert_eq!(result.outcome.unwrap_err(), Failure::InvalidInput);
    assert!(result.message.unwrap().contains("parity"));
}

#[test]
fn the_search_record_accounts_for_every_depth_below_the_answer() {
    let moves = seq("R U2 F' D L2 B");
    let start = solved().apply(&moves).unwrap();
    let twist = track_moves(CenterTwist::UPRIGHT, &moves).unwrap().twist;
    let options = SolveOptions::default().with_start_twist(twist);
    let engine = Min2PhaseSolver::new();
    let (result, record) = engine.solve_to_recorded(&start, &picture_target(), &options);
    let answer = result.outcome.unwrap();
    let record = record.expect("the exact route records");
    let (found, _) = record.found.expect("found");
    assert_eq!(found, answer.len());
    let depths: Vec<usize> = record.completed.iter().map(|&(d, _)| d).collect();
    assert_eq!(
        depths,
        (record.start_bound..answer.len()).collect::<Vec<_>>()
    );
    assert!(record
        .tables
        .iter()
        .any(|t| t.name == "twist" && t.cells == 4096));
    // the digests are of the contents: a second run gives the same ones
    let (_, again) = engine.solve_to_recorded(&start, &picture_target(), &options);
    assert_eq!(again.unwrap().tables, record.tables);
}

#[test]
fn a_free_orientation_turns_the_twist_condition_with_the_cube() {
    // every sticker open, only the picture on the F center must be upright:
    // after F the F picture is turned, but in the orientation that brings the
    // R center to F (side pictures are upright there) no move is needed
    let target = Target::from_sets(vec![0b11_1111; 54])
        .unwrap()
        .with_twist(parse_twist_masks("X,X,0,X,X,X").unwrap());
    let moves = seq("F");
    let start = solved().apply(&moves).unwrap();
    let twist = track_moves(CenterTwist::UPRIGHT, &moves).unwrap().twist;
    let fixed = SolveOptions::default().with_start_twist(twist);
    let engine = Min2PhaseSolver::new();
    let (fixed_result, _) = engine.solve_to_recorded(&start, &target, &fixed);
    assert_eq!(fixed_result.outcome.unwrap().len(), 1);
    let free = fixed.with_free_orientation(true);
    let (free_result, _) = engine.solve_to_recorded(&start, &target, &free);
    assert_eq!(free_result.outcome.unwrap().len(), 0);
}
