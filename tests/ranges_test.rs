//! Compressed real solutions replay to the same state.

use std::sync::Arc;

use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate::scramble;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};
use cubacle::ranges::{compress, expand};

#[test]
fn a_compressed_4x4x4_solution_still_solves_the_cube() {
    let solver = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    for _ in 0..3 {
        let kind = PuzzleKind::Cube4;
        let cube = Facelets::solved(kind).apply(&scramble(kind, 60)).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("solvable");
        let packed = compress(kind, solution);
        assert!(packed.len() <= solution.len());
        let replayed = cube.apply(&expand(&packed)).unwrap();
        assert!(replayed.is_solved());
    }
}
