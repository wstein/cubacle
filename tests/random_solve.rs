use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};
use std::time::Instant;

const MAX_SOL_LEN: u8 = 20;
const N_TEST: u32 = 1000;

fn assert_solves(solver: &Min2PhaseSolver, cube: &Facelets) {
    let options = SolveOptions::default().with_max_moves(MAX_SOL_LEN);
    let result = solver.solve(cube, &options);
    let solution = result.solution().expect("solved");
    let end = cube.apply(solution).expect("solver returned valid moves");
    assert!(end.is_solved(), "{end}");
}

#[test]
fn random_move_solve() {
    let solver = Min2PhaseSolver::new();
    let now = Instant::now();
    let n_moves = 100;
    for _ in 0..N_TEST {
        let scramble = generate::scramble(PuzzleKind::Cube3, n_moves);
        assert_eq!(scramble.len(), n_moves);
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&scramble)
            .expect("moves fit a 3x3");
        assert_solves(&solver, &cube);
    }
    let elapsed = now.elapsed() / N_TEST;
    println!("Random move solve avg: {:.3?}", elapsed);
}

#[test]
fn random_state_solve() {
    let solver = Min2PhaseSolver::new();
    let now = Instant::now();
    for _ in 0..N_TEST {
        let cube = generate::random_state(PuzzleKind::Cube3).expect("generator");
        assert_solves(&solver, &cube);
    }
    let elapsed = now.elapsed() / N_TEST;
    println!("Random state solve avg: {:.3?}", elapsed);
}
