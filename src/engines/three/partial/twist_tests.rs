//! Tests of the center twist as a search coordinate (picture cubes).
//!
//! Every answer is replayed twice, apart from the engine: the stickers through
//! the facelet model and the twists through the center tracker.

use super::super::solver::Min2PhaseSolver;
use super::goal::{color_masks, Goal};
use super::model::{pack, twist_of, twist_step, Turns, TURNS};
use super::twist::TwistTable;
use crate::puzzle::{
    parse_twist_masks, CenterTwists, Face, Facelets, Failure, MoveSequence, PuzzleKind,
    SolveOptions, SolveResult, Solver, Target, FREE_TWIST,
};

fn seq(text: &str) -> MoveSequence {
    text.parse().unwrap()
}

fn upright() -> CenterTwists {
    CenterTwists::untwisted(PuzzleKind::Cube3).unwrap()
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
) -> (Facelets, CenterTwists, SolveResult) {
    let moves = seq(scramble);
    let start = solved().apply(&moves).unwrap();
    let twist = upright().track(&moves).unwrap().twists;
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
        let end = twist.track(&moves).unwrap();
        assert!(end.frame.is_empty());
        assert_eq!(end.twists, upright(), "{scramble}: {moves}");
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
    assert_eq!(twist.track(&moves).unwrap().twists.twist(Face::R), 0);
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
    let options = SolveOptions::default()
        .with_start_twist(CenterTwists::new(PuzzleKind::Cube3, [1, 0, 0, 0, 0, 0]).unwrap());
    let result = Min2PhaseSolver::new().solve_to(&start, &picture_target(), &options);
    assert_eq!(result.outcome.unwrap_err(), Failure::InvalidInput);
    assert!(result.message.unwrap().contains("parity"));
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
    let twist = upright().track(&moves).unwrap().twists;
    let fixed = SolveOptions::default().with_start_twist(twist);
    let engine = Min2PhaseSolver::new();
    let fixed_result = engine.solve_to(&start, &target, &fixed);
    assert_eq!(fixed_result.outcome.unwrap().len(), 1);
    let free = fixed.with_free_orientation(true);
    let free_result = engine.solve_to(&start, &target, &free);
    assert_eq!(free_result.outcome.unwrap().len(), 0);
}

#[test]
fn the_search_step_of_the_twists_equals_the_tracker_for_all_18_turns() {
    let turns = Turns::new();
    let start = [1u8, 2, 0, 3, 1, 2];
    for m in 0..TURNS {
        let moves = MoveSequence::new(vec![turns.moves[m]]);
        let tracked = CenterTwists::new(PuzzleKind::Cube3, start)
            .unwrap()
            .track(&moves)
            .unwrap();
        let stepped = twist_step(pack(start), m);
        for (g, face) in Face::ALL.into_iter().enumerate() {
            assert_eq!(twist_of(stepped, g), tracked.twists.twist(face), "{moves}");
        }
    }
}

#[test]
fn the_twist_table_counts_the_faces_that_need_a_turn_at_least() {
    let target = Target::complete(&solved());
    let masks = color_masks(target.sets());
    let goal = Goal::with_twists(&[(masks, parse_twist_masks("0,0,0,0,0,0").unwrap())]);
    let table = TwistTable::build(&goal).expect("a condition");
    assert_eq!(table.bound(pack([0; 6])), 0);
    // one face off by any amount is one move, three faces are three
    assert_eq!(table.bound(pack([1, 0, 0, 0, 0, 0])), 1);
    assert_eq!(table.bound(pack([0, 2, 0, 0, 0, 0])), 1);
    assert_eq!(table.bound(pack([3, 0, 1, 0, 2, 0])), 3);
    // a goal without a twist condition needs no table
    let free = Goal::with_twists(&[(masks, FREE_TWIST)]);
    assert!(TwistTable::build(&free).is_none());
}

#[test]
fn a_twist_condition_is_refused_for_a_start_with_moved_centers() {
    // the free route would read the start by its centers and drop the twists
    let start = solved().apply(&seq("2R R")).unwrap();
    let options = SolveOptions::default().with_free_orientation(true);
    let result = Min2PhaseSolver::new().solve_to(&start, &picture_target(), &options);
    assert_eq!(result.outcome.unwrap_err(), Failure::InvalidInput);
    assert!(result.message.unwrap().contains("centers"));
}

#[test]
fn an_open_target_with_a_twist_condition_asks_for_the_turn() {
    // no sticker matters, only the R picture must show a quarter turn
    let target = Target::from_sets(vec![0b11_1111; 54])
        .unwrap()
        .with_twist(parse_twist_masks("X,1,X,X,X,X").unwrap());
    let result = Min2PhaseSolver::new().solve_to(&solved(), &target, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap().to_string(), "R");
    // with the condition met at the start, nothing is asked
    let target = target.with_twist(parse_twist_masks("X,0,X,X,X,X").unwrap());
    let result = Min2PhaseSolver::new().solve_to(&solved(), &target, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap().len(), 0);
}
