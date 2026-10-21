use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Solver};
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
    let results = Min2PhaseSolver::shared().solve_batch(&cubes, &options());

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
    let results = Min2PhaseSolver::shared().solve_batch(&cubes, &options());

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
                for result in Min2PhaseSolver::shared().solve_batch(&cubes, &options()) {
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
    let result = Min2PhaseSolver::shared().solve_parallel(&cube, &options());

    let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
    assert!(cube.apply(solution).unwrap().is_solved());
}

#[test]
fn parallel_solve_is_valid_on_every_run() {
    // The orientation searches share a best-length bound, so which valid
    // solution wins can vary between runs. What is guaranteed is that every
    // run solves the cube within the move limit.
    let cube = scrambled("R U R' U' F2 D L2");
    let solver = Min2PhaseSolver::shared();
    let options = options();
    let limit = usize::from(options.max_moves.unwrap());

    for _ in 0..6 {
        let result = solver.solve_parallel(&cube, &options);
        let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
        assert!(solution.len() <= limit, "{solution}");
        assert!(cube.apply(solution).unwrap().is_solved(), "{solution}");
    }
}

#[test]
fn parallel_budget_reports_a_deadline_without_a_solution() {
    let cube = scrambled("R U R' U'");
    let options = options().with_time_budget(Duration::ZERO);
    let result = Min2PhaseSolver::shared().solve_parallel(&cube, &options);

    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn parallel_minimum_time_retains_a_valid_solution() {
    // The minimum bounds how long the search keeps improving; a search that has
    // run out of shorter solutions returns early, so only the budget is asserted.
    let cube = scrambled("R U R' U' F2 D L2");
    let budget = Duration::from_millis(200);
    let options = options().with_time_budget(budget);
    let solver = Min2PhaseSolver::shared();
    solver.prepare();
    let started = Instant::now();
    let result = solver.solve_parallel_with_minimum_time(&cube, &options, Duration::from_millis(5));

    assert!(started.elapsed() < budget + Duration::from_millis(100));
    let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
    assert!(cube.apply(solution).unwrap().is_solved());
}
