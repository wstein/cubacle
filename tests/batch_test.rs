use min2phase::{apply_moves, from_moves, solve_batch, SolveOutcome};
use std::thread;

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
