use cubacle::api::{
    Facelets, Failure, Min2PhaseSolver, MoveSequence, PuzzleKind, SolveOptions, Solver,
};
use cubacle::{apply_moves, from_moves};
use std::time::Duration;

fn scrambled(moves: &str) -> Facelets {
    Facelets::parse(&from_moves(moves).expect("valid scramble")).expect("valid facelets")
}

fn assert_replays_to_solved(start: &Facelets, solution: &MoveSequence) {
    let end = apply_moves(&start.to_string(), &solution.to_string()).expect("replayable moves");
    assert_eq!(end, Facelets::solved(PuzzleKind::Cube3).to_string());
}

#[test]
fn solves_a_scramble_with_replayable_moves() {
    let cube = scrambled("R U R' U' F2 D L2");
    let result = Min2PhaseSolver::new().solve(&cube, &SolveOptions::default());
    let solution = result.solution().expect("solved");
    assert!(solution.fits(PuzzleKind::Cube3));
    assert_replays_to_solved(&cube, solution);
    assert!(result.metrics.probes.is_some());
}

#[test]
fn a_solved_cube_needs_no_moves() {
    let cube = Facelets::solved(PuzzleKind::Cube3);
    let result = Min2PhaseSolver::new().solve(&cube, &SolveOptions::default());
    assert!(result.solution().is_some_and(MoveSequence::is_empty));
}

#[test]
fn rejects_the_wrong_cube_size() {
    let cube = Facelets::solved(PuzzleKind::Cube2);
    let result = Min2PhaseSolver::new().solve(&cube, &SolveOptions::default());
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
    let result = Min2PhaseSolver::new().solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn a_zero_budget_reports_budget_exhausted() {
    let cube = scrambled("R U R' U'");
    let options = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = Min2PhaseSolver::new().solve(&cube, &options);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn prepare_is_idempotent() {
    let solver = Min2PhaseSolver::new();
    solver.prepare();
    solver.prepare();
    assert_eq!(solver.kind(), PuzzleKind::Cube3);
}
