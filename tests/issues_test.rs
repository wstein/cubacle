use min2phase::{apply_moves, solve, solve_with_budget, solve_with_stats};
use std::time::Duration;

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
    assert!(result.probes > 0, "expected phase-two search probes");
}

#[test]
fn solve_budget_reports_timeout() {
    let cube = apply_moves(SOLVED, "R U R' U' F2 D B2 L2 U2 R2 F' D2").expect("valid moves");
    let result = solve_with_budget(cube.as_str(), MAX_SOL_LEN, Duration::ZERO);

    assert_eq!(result.solution, "Error 8");
    assert!(result.timed_out);
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
