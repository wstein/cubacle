use min2phase::{
    apply_moves, solve, solve_with_budget, solve_with_minimum_time, solve_with_stats, SolveOutcome,
};
use std::time::{Duration, Instant};

const MAX_SOL_LEN: u8 = 20;
const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

#[test]
fn string_slices_are_accepted_by_public_apis() {
    let cube = apply_moves(SOLVED, "R U R' U'").expect("valid move sequence");
    let solution = solve(cube.as_str(), MAX_SOL_LEN);
    let solved = apply_moves(cube.as_str(), solution.as_str()).expect("valid solver result");

    assert_eq!(solved, SOLVED);
}

#[test]
fn solve_stats_include_phase_two_probes() {
    let cube = apply_moves(SOLVED, "R U R' U'").expect("valid move sequence");
    let result = solve_with_stats(cube.as_str(), MAX_SOL_LEN);
    let solved = apply_moves(cube.as_str(), result.solution.as_str()).expect("valid solver result");

    assert_eq!(solved, SOLVED);
    assert_eq!(result.outcome, SolveOutcome::Solved);
    assert!(result.probes > 0, "expected phase-two search probes");
    assert!(result.phase1_nodes > 0, "expected phase-one search nodes");
    assert!(
        result.phase1_children_recurred > 0,
        "expected phase-one recursive children"
    );
    assert_eq!(
        result.phase1_rejected_slice_twist,
        result.phase1_slice_twist_breaks + result.phase1_slice_twist_equals
    );
    assert_eq!(
        result.phase1_rejected_slice_flip,
        result.phase1_slice_flip_breaks + result.phase1_slice_flip_equals
    );
}

#[test]
fn solve_budget_reports_timeout() {
    let cube = apply_moves(SOLVED, "R U R' U'").expect("valid moves");
    let result = solve_with_budget(cube.as_str(), MAX_SOL_LEN, Duration::ZERO);

    assert_eq!(result.solution, "Error 8");
    assert!(result.timed_out);
    assert_eq!(result.outcome, SolveOutcome::DeadlineNoSolution);
}

#[test]
fn solve_minimum_time_retains_a_valid_solution_until_the_minimum_elapses() {
    let cube = apply_moves(SOLVED, "R U R' U'").expect("valid moves");
    let minimum = Duration::from_millis(10);
    let started = Instant::now();
    let result = solve_with_minimum_time(
        cube.as_str(),
        MAX_SOL_LEN,
        minimum,
        Duration::from_millis(30),
    );

    assert!(started.elapsed() >= minimum);
    assert!(!result.timed_out);
    assert!(matches!(
        result.outcome,
        SolveOutcome::Solved | SolveOutcome::DeadlineWithSolution
    ));
    let solved = apply_moves(cube.as_str(), result.solution.as_str()).expect("valid solver result");
    assert_eq!(solved, SOLVED);
}

#[test]
fn test_issue1_solve() {
    let cube = "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB".to_string();
    let solution = solve(&cube, MAX_SOL_LEN);
    let solved_cubie = apply_moves(&cube, &solution);
    assert!(
        solved_cubie == Some(SOLVED.to_string()),
        "unexpected result: solution={solution} solved_cubie={solved_cubie:?}"
    );
}
