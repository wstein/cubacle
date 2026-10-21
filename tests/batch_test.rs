use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions};
use std::thread;
use std::time::{Duration, Instant};

fn scrambled(moves: &str) -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves.parse().expect("valid scramble"))
        .expect("moves fit a 3x3")
}

fn cubes() -> Vec<Facelets> {
    [
        "R U R' U'",
        "F2 D L2",
        "U2 R2 F2 D2",
        "L B L' B'",
        "R2 U F' D L2",
    ]
    .into_iter()
    .map(scrambled)
    .collect()
}

fn options() -> SolveOptions {
    SolveOptions::default().with_max_moves(20)
}

#[test]
fn batch_solutions_validate_in_input_order() {
    let cubes = cubes();
    let results = Min2PhaseSolver::new().solve_batch(&cubes, &options());

    assert_eq!(results.len(), cubes.len());
    for (cube, result) in cubes.iter().zip(results) {
        let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
        assert!(cube.apply(solution).unwrap().is_solved());
    }
}

#[test]
fn batch_rejects_the_wrong_size_without_losing_the_order() {
    let mut cubes = cubes();
    cubes.insert(2, Facelets::solved(PuzzleKind::Cube2));
    let results = Min2PhaseSolver::new().solve_batch(&cubes, &options());

    assert_eq!(results.len(), cubes.len());
    assert_eq!(results[2].outcome, Err(Failure::InvalidInput));
    for (index, (cube, result)) in cubes.iter().zip(results).enumerate() {
        if index != 2 {
            assert!(cube.apply(result.solution().unwrap()).unwrap().is_solved());
        }
    }
}

#[test]
fn batch_pool_is_safe_under_concurrent_callers() {
    let callers: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                let cubes = cubes();
                for result in Min2PhaseSolver::new().solve_batch(&cubes, &options()) {
                    assert!(result.is_solved(), "{result:?}");
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
    let cube = scrambled("R U R' U' F2 D L2");
    let result = Min2PhaseSolver::new().solve_parallel(&cube, &options());

    let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
    assert!(cube.apply(solution).unwrap().is_solved());
}

#[test]
fn parallel_solve_is_repeatable() {
    let cube = scrambled("R U R' U' F2 D L2");
    let solver = Min2PhaseSolver::new();
    let expected = solver.solve_parallel(&cube, &options()).outcome;

    for _ in 0..5 {
        assert_eq!(solver.solve_parallel(&cube, &options()).outcome, expected);
    }
}

#[test]
fn parallel_budget_reports_a_deadline_without_a_solution() {
    let cube = scrambled("R U R' U'");
    let options = options().with_time_budget(Duration::ZERO);
    let result = Min2PhaseSolver::new().solve_parallel(&cube, &options);

    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn parallel_minimum_time_retains_a_valid_solution() {
    let cube = scrambled("R U R' U' F2 D L2");
    let minimum = Duration::from_millis(5);
    let options = options().with_time_budget(Duration::from_millis(200));
    let started = Instant::now();
    let result = Min2PhaseSolver::new().solve_parallel_with_minimum_time(&cube, &options, minimum);

    assert!(started.elapsed() >= minimum);
    let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
    assert!(cube.apply(solution).unwrap().is_solved());
}
