//! The 4x4x4 engine with the full tables, through the public API.
//!
//! `tests/fixtures/four_transfer.txt` holds the reduction (phases 1 to 3)
//! that the Java solver cs0x7f/TPR-4x4x4-Solver made of 24 cubes. The
//! solution of `FourSolver` must start with those moves, each wide turn
//! possibly swapped for the opposite one (which only turns the whole cube;
//! see `engines/four/swap.rs`), so the head is compared up to a rotation of the
//! cube. The 3x3x3 tail is another solver's.

use std::sync::Arc;

use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Solver};

const TRANSFER: &str = include_str!("fixtures/four_transfer.txt");

fn solver(threads: usize) -> FourSolver {
    FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(threads)
}

/// Whether `b` is `a` turned as a whole, by some product of `x`, `y`, `z`.
fn up_to_rotation(a: &Facelets, b: &Facelets) -> bool {
    let axes = ["Rw Lw'", "Uw Dw'", "Fw Bw'"];
    let mut seen = vec![b.clone()];
    let mut frontier = vec![b.clone()];
    while let Some(cube) = frontier.pop() {
        if cube == *a {
            return true;
        }
        for axis in axes {
            let next = cube.apply(&axis.parse().expect("rotation")).expect("fits");
            if !seen.contains(&next) {
                seen.push(next.clone());
                frontier.push(next);
            }
        }
    }
    false
}

#[test]
fn the_reduction_is_the_one_the_java_solver_makes() {
    let four = solver(2);
    let mut cubes = 0;
    for line in TRANSFER
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (facelets, moves) = line.split_once('\t').expect("facelets TAB moves");
        let cube = Facelets::parse(facelets).expect("facelets");
        let result = four.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("a solution");
        assert!(cube.apply(solution).expect("replays").is_solved());

        let expected: Vec<&str> = moves.split_whitespace().collect();
        let text = solution.to_string();
        let got: Vec<&str> = text.split_whitespace().collect();
        assert!(got.len() >= expected.len(), "{facelets}");
        let head = got[..expected.len()].join(" ");
        let want = cube.apply(&moves.parse().expect("moves")).expect("replays");
        let have = cube.apply(&head.parse().expect("moves")).expect("replays");
        assert!(
            up_to_rotation(&want, &have),
            "reduction differs from the Java one for {facelets}"
        );
        cubes += 1;
    }
    assert_eq!(cubes, 24);
}

#[test]
fn generated_states_are_solved_and_the_answer_replays() {
    let four = solver(2);
    for _ in 0..5 {
        let cube = generate::random_state(PuzzleKind::Cube4).expect("a generator");
        let result = four.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("a solution");
        assert!(cube.apply(solution).expect("replays").is_solved());
        assert!(solution.len() <= 60, "{solution}");
    }
}

#[test]
fn a_cube_of_another_size_is_refused() {
    let result = solver(1).solve(
        &Facelets::solved(PuzzleKind::Cube3),
        &SolveOptions::default(),
    );
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

/// 1, 2 and 5 threads give the same answers; the distribution of lengths is
/// the reference's (mean 44.6 without the final rotation, which adds about 3).
/// Ignored by default: it solves 150 cubes three times over.
/// Run with `cargo test --release -- --ignored four_full`.
#[test]
#[ignore = "solves hundreds of cubes"]
fn four_full_answers_do_not_depend_on_the_threads_and_have_the_reference_length() {
    let (one, two, five) = (solver(1), solver(2), solver(5));
    let mut total = 0;
    let count = 150;
    for _ in 0..count {
        let cube = generate::random_state(PuzzleKind::Cube4).expect("a generator");
        let options = SolveOptions::default();
        let a = one.solve(&cube, &options);
        let solution = a.solution().expect("a solution");
        assert!(cube.apply(solution).expect("replays").is_solved());
        assert_eq!(a.outcome, two.solve(&cube, &options).outcome, "{cube}");
        assert_eq!(a.outcome, five.solve(&cube, &options).outcome, "{cube}");
        total += solution.len();
    }
    let mean = total as f64 / f64::from(count);
    // 44.7 plus the rotation (about 3.1), with room for the sampling error.
    assert!(mean < 49.5, "mean {mean}");
}
