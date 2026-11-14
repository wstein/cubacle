//! Tests of the center twist as a search coordinate (picture cubes).
//!
//! Every answer is replayed twice, apart from the engine: the stickers through
//! the facelet model and the twists through the center tracker.

use super::super::solver::Min2PhaseSolver;
use super::goal::{color_masks, twist_ok, Goal};
use super::model::{pack, twist_of, twist_step, Turns, TURNS};
use super::twist::TwistTable;
use crate::puzzle::{
    center_rotations, parse_twist_masks, rotation_maps, CenterTwists, Face, Facelets, Failure,
    MoveSequence, PuzzleKind, SolveOptions, SolveResult, Solver, Target, FREE_TWIST,
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
    Target::complete(&solved()).with_twist([1; 6]).unwrap()
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
    let target = Target::complete(&solved()).with_twist(masks).unwrap();
    let (start, twist, result) = run("R", &target, SolveOptions::default());
    let moves = result.outcome.clone().unwrap();
    assert_eq!(moves.to_string(), "R'");
    assert_eq!(twist.track(&moves).unwrap().twists.twist(Face::R), 0);
    assert!(start.apply(&moves).unwrap().is_solved());
    // two twists are accepted for the R picture, 0 or 2: the one move R2 does
    let masks = parse_twist_masks("X,0|2,X,X,X,X").unwrap();
    let target = Target::complete(&solved()).with_twist(masks).unwrap();
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
        .with_twist(parse_twist_masks("X,X,0,X,X,X").unwrap())
        .unwrap();
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
        .with_twist(parse_twist_masks("X,1,X,X,X,X").unwrap())
        .unwrap();
    let result = Min2PhaseSolver::new().solve_to(&solved(), &target, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap().to_string(), "R");
    // with the condition met at the start, nothing is asked
    let target = target
        .with_twist(parse_twist_masks("X,0,X,X,X,X").unwrap())
        .unwrap();
    let result = Min2PhaseSolver::new().solve_to(&solved(), &target, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap().len(), 0);
}

fn open_target(masks: &str) -> Target {
    Target::from_sets(vec![0b11_1111; 54])
        .unwrap()
        .with_twist(parse_twist_masks(masks).unwrap())
        .unwrap()
}

fn twists_of(t: [u8; 6]) -> CenterTwists {
    CenterTwists::new(PuzzleKind::Cube3, t).unwrap()
}

/// The shortest answer length for `target` from `start` with the twists `twist`.
fn length(start: &Facelets, twist: [u8; 6], target: &Target, free: bool) -> usize {
    let options = SolveOptions::default()
        .with_start_twist(twists_of(twist))
        .with_free_orientation(free);
    Min2PhaseSolver::new()
        .solve_to(start, target, &options)
        .outcome
        .expect("an answer")
        .len()
}

#[test]
fn the_rotation_table_of_the_goal_agrees_with_the_sticker_maps_and_the_tracker() {
    let rotations = center_rotations();
    let maps = rotation_maps(PuzzleKind::Cube3);
    assert_eq!(rotations.len(), maps.len());
    let start = [1u8, 2, 0, 3, 1, 1];
    for (rotation, map) in rotations.iter().zip(&maps) {
        let seen = twists_of(start).rotated(&rotation.word).unwrap();
        for g in 0..6 {
            // where the center goes: the sticker map says the same
            assert_eq!(rotation.dest[g], map[g * 9 + 4] / 9, "{}", rotation.word);
            // how its picture turns: the tracker, which follows the geometry
            assert_eq!(
                seen.twist(Face::ALL[rotation.dest[g]]),
                (start[g] + rotation.add[g]) & 3,
                "{}",
                rotation.word
            );
        }
    }
}

#[test]
fn a_free_orientation_uses_the_quarters_a_rotation_adds() {
    // no sticker matters; the F picture must show a quarter turn. A move F does
    // it, but the rotation z keeps the center on F and turns its picture a
    // quarter: no move at all, only because `add` is applied.
    let target = open_target("X,X,1,X,X,X");
    assert_eq!(length(&solved(), [0; 6], &target, false), 1);
    assert_eq!(length(&solved(), [0; 6], &target, true), 0);
}

#[test]
fn a_free_orientation_reads_the_center_that_lands_on_the_face() {
    // the F and B pictures are turned a quarter; U must show a quarter. A move U
    // does it, but x brings the F center to U with its picture as it is.
    let target = open_target("1,X,X,X,X,X");
    let twist = [0, 0, 1, 0, 0, 1];
    assert_eq!(length(&solved(), twist, &target, false), 1);
    assert_eq!(length(&solved(), twist, &target, true), 0);
}

#[test]
fn a_condition_on_the_b_center_is_checked() {
    let target = open_target("X,X,X,X,X,1");
    let result = Min2PhaseSolver::new().solve_to(
        &solved(),
        &target,
        &SolveOptions::default().with_start_twist(upright()),
    );
    assert_eq!(result.outcome.unwrap().to_string(), "B");
}

#[test]
fn a_twist_the_corners_rule_out_is_unreachable_at_once() {
    // solved stickers have an even corner permutation; one quarter in all is odd
    let target = Target::complete(&solved())
        .with_twist(parse_twist_masks("0,0,0,0,0,1").unwrap())
        .unwrap();
    let options = SolveOptions::default()
        .with_start_twist(upright())
        .with_time_budget(std::time::Duration::from_secs(5));
    let began = std::time::Instant::now();
    let result = Min2PhaseSolver::new().solve_to(&solved(), &target, &options);
    assert_eq!(result.outcome.unwrap_err(), Failure::Exhausted);
    assert!(result.message.unwrap().contains("unreachable"));
    assert!(began.elapsed() < std::time::Duration::from_secs(4));
}

#[test]
fn the_rotated_goal_accepts_exactly_what_the_tracker_says_after_some_rotation() {
    // A twist vector t meets the free-orientation goal when, after some whole-cube
    // rotation r, the twists the tracker reads on the faces meet the condition.
    let target = Target::from_sets(vec![0b11_1111; 54]).unwrap();
    let masks = color_masks(target.sets());
    let rotations: Vec<_> = rotation_maps(PuzzleKind::Cube3)
        .into_iter()
        .zip(center_rotations())
        .collect();
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..60 {
        // a random condition: each center accepts 1 to 3 twists, or any
        let condition: [u8; 6] = std::array::from_fn(|_| {
            if next() % 3 == 0 {
                0b1111
            } else {
                let keep = 1 + (next() % 4) as u8;
                (0..keep).fold(0u8, |acc, _| acc | 1 << (next() % 4))
            }
        });
        let goal = Goal::rotated_twisted(&masks, &condition, &rotations);
        let wanted = target.clone().with_twist(condition).unwrap();
        for _ in 0..40 {
            let t: [u8; 6] = std::array::from_fn(|_| (next() % 4) as u8);
            let tw = pack(t);
            let by_goal = goal.views.iter().any(|v| twist_ok(&v.twist, tw));
            let by_tracker = rotations.iter().any(|(_, r)| {
                let seen = twists_of(t).rotated(&r.word).unwrap();
                wanted.twist_met(&seen)
            });
            assert_eq!(by_goal, by_tracker, "{condition:?} {t:?}");
        }
    }
}
