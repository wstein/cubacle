use min2phase::{
    apply_moves, from_moves, solve_batch, solve_parallel, solve_parallel_with_budget,
    solve_parallel_with_minimum_time, SolveOutcome,
};
use std::thread;
use std::time::{Duration, Instant};

const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

fn cubes() -> Vec<String> {
    [
        "R U R' U'",
        "F2 D L2",
        "U2 R2 F2 D2",
        "L B L' B'",
        "R2 U F' D L2",
    ]
    .into_iter()
    .map(|moves| from_moves(moves).expect("valid scramble"))
    .collect()
}

#[test]
fn batch_solutions_validate_in_input_order() {
    let cubes = cubes();
    let results = solve_batch(&cubes, 20);

    assert_eq!(results.len(), cubes.len());
    for (cube, result) in cubes.iter().zip(results) {
        assert_eq!(result.outcome, SolveOutcome::Solved, "{result:?}");
        assert_eq!(apply_moves(cube, &result.solution).as_deref(), Some(SOLVED));
    }
}

#[test]
fn batch_pool_is_safe_under_concurrent_callers() {
    let callers: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                let cubes = cubes();
                for result in solve_batch(&cubes, 20) {
                    assert_eq!(result.outcome, SolveOutcome::Solved, "{result:?}");
                }
            })
        })
        .collect();

    for caller in callers {
        caller.join().expect("batch caller panicked");
    }
}

#[test]
fn parallel_solve_returns_a_valid_solution() {
    let cube = from_moves("R U R' U' F2 D L2").expect("valid scramble");
    let result = solve_parallel(&cube, 20);

    assert_eq!(result.outcome, SolveOutcome::Solved, "{result:?}");
    assert_eq!(
        apply_moves(&cube, &result.solution).as_deref(),
        Some(SOLVED)
    );
}

#[test]
fn parallel_solve_is_repeatable() {
    let cube = from_moves("R U R' U' F2 D L2").expect("valid scramble");
    let expected = solve_parallel(&cube, 20).solution;

    for _ in 0..5 {
        assert_eq!(solve_parallel(&cube, 20).solution, expected);
    }
}

#[test]
fn parallel_budget_reports_a_deadline_without_a_solution() {
    let cube = from_moves("R U R' U'").expect("valid scramble");
    let result = solve_parallel_with_budget(&cube, 20, Duration::ZERO);

    assert_eq!(result.solution, "Error 8");
    assert_eq!(result.outcome, SolveOutcome::DeadlineNoSolution);
}

#[test]
fn parallel_minimum_time_retains_a_valid_solution() {
    let cube = from_moves("R U R' U' F2 D L2").expect("valid scramble");
    let minimum = Duration::from_millis(5);
    let started = Instant::now();
    let result = solve_parallel_with_minimum_time(&cube, 20, minimum, Duration::from_millis(200));

    assert!(started.elapsed() >= minimum);
    assert!(matches!(
        result.outcome,
        SolveOutcome::Solved | SolveOutcome::DeadlineWithSolution
    ));
    assert_eq!(
        apply_moves(&cube, &result.solution).as_deref(),
        Some(SOLVED)
    );
}
