use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, Failure, MoveSequence, PuzzleKind, SolveOptions, Solver};
use std::time::Duration;

fn scrambled(moves: &str) -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves.parse().expect("valid scramble"))
        .expect("moves fit a 3x3")
}

fn assert_replays_to_solved(start: &Facelets, solution: &MoveSequence) {
    let end = start.apply(solution).expect("replayable moves");
    assert!(end.is_solved(), "{end}");
}

#[test]
fn solves_a_scramble_with_replayable_moves() {
    let cube = scrambled("R U R' U' F2 D L2");
    let result = Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default());
    let solution = result.solution().expect("solved");
    assert!(solution.fits(PuzzleKind::Cube3));
    assert_replays_to_solved(&cube, solution);
    assert!(result.metrics.probes.is_some());
}

#[test]
fn a_solved_cube_needs_no_moves() {
    let cube = Facelets::solved(PuzzleKind::Cube3);
    let result = Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default());
    assert!(result.solution().is_some_and(MoveSequence::is_empty));
}

#[test]
fn rejects_the_wrong_cube_size() {
    let cube = Facelets::solved(PuzzleKind::Cube2);
    let result = Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn an_impossible_state_is_invalid_input() {
    // Swapping the two stickers of one edge flips it: well-formed, unreachable.
    let mut stickers: Vec<char> = Facelets::solved(PuzzleKind::Cube3)
        .to_string()
        .chars()
        .collect();
    stickers.swap(5, 10);
    let cube = Facelets::parse(&stickers.into_iter().collect::<String>()).expect("well-formed");
    let result = Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn a_zero_budget_reports_budget_exhausted() {
    let cube = scrambled("R U R' U'");
    let options = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = Min2PhaseSolver::shared().solve(&cube, &options);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn prepare_is_idempotent() {
    let solver = Min2PhaseSolver::shared();
    solver.prepare();
    solver.prepare();
    assert_eq!(solver.kind(), PuzzleKind::Cube3);
}

#[test]
fn a_small_move_limit_gives_up_quickly() {
    // A cube needing 20 moves has no 3-move solution; the search must end at once
    // instead of exploring first phases far deeper than the limit allows.
    let cube = Facelets::parse("FLUBUDDLDBRLFRDDRBRDLBFUDLRLBFUDFBDLUUBLLRURFFFRFBUUBR").unwrap();
    let started = std::time::Instant::now();
    let result = Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default().with_max_moves(3));

    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn the_cube_from_the_first_issue_is_solved() {
    // The cube from the first reported issue.
    let cube = Facelets::parse("UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB").unwrap();
    let result =
        Min2PhaseSolver::shared().solve(&cube, &SolveOptions::default().with_max_moves(20));
    assert_replays_to_solved(&cube, result.solution().expect("solved"));
}

#[test]
fn the_minimum_time_keeps_a_valid_solution_until_it_elapses() {
    let cube = scrambled("R U R' U'");
    let minimum = Duration::from_millis(10);
    let options = SolveOptions::default()
        .with_max_moves(20)
        .with_time_budget(Duration::from_millis(200));
    let started = std::time::Instant::now();
    let result = Min2PhaseSolver::shared().solve_with_minimum_time(&cube, &options, minimum);

    assert!(started.elapsed() >= minimum);
    assert_replays_to_solved(&cube, result.solution().expect("solved"));
}
