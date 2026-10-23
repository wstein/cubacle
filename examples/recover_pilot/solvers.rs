//! The solvers, built as `cubacle solve` builds them.

use std::sync::Arc;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{PuzzleKind, Solver};

pub fn solver_for(kind: PuzzleKind) -> Box<dyn Solver> {
    let three = Arc::new(Min2PhaseSolver::new());
    match kind {
        PuzzleKind::Cube4 => {
            let four = FourSolver::new(three);
            four.prepare();
            Box::new(four)
        }
        PuzzleKind::Cube5 => {
            let five = FiveSolver::new(three);
            five.prepare();
            Box::new(five)
        }
        PuzzleKind::Cube6 | PuzzleKind::Cube7 => {
            let four = FourSolver::new(three.clone());
            let five = FiveSolver::new(three.clone());
            four.prepare();
            five.prepare();
            Box::new(DwaltonSolver::new(
                kind,
                three,
                Arc::new(four),
                Arc::new(five),
            ))
        }
        _ => Box::new(Min2PhaseSolver::new()),
    }
}

pub fn kind_of(size: usize) -> PuzzleKind {
    match size {
        4 => PuzzleKind::Cube4,
        5 => PuzzleKind::Cube5,
        6 => PuzzleKind::Cube6,
        7 => PuzzleKind::Cube7,
        _ => panic!("the pilot has no {size}x{size}x{size}"),
    }
}
