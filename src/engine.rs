//! The engine for a cube size, and the default limits.

use std::sync::Arc;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::pattern::PatternShortcut;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::puzzle::{PuzzleKind, SolveOptions, Solver};

/// The engine for a cube size, prepared; progress goes to standard error. From
/// the 4x4x4 up it also tries known patterns and answers with the shorter solution.
pub(crate) fn solver_for(kind: PuzzleKind) -> Box<dyn Solver> {
    let engine = engine_for(kind);
    if kind.size() >= 4 {
        Box::new(PatternShortcut::new(engine))
    } else {
        engine
    }
}

pub(crate) fn engine_for(kind: PuzzleKind) -> Box<dyn Solver> {
    match kind {
        PuzzleKind::Cube2 => Box::new(PocketSolver::new()),
        PuzzleKind::Cube3 => Box::new(Min2PhaseSolver::new()),
        PuzzleKind::Cube4 => {
            let four = FourSolver::new(Arc::new(Min2PhaseSolver::new()));
            eprintln!("preparing the 4x4x4 tables (built now, about 12 MB)");
            four.prepare();
            Box::new(four)
        }
        PuzzleKind::Cube5 => {
            let five = FiveSolver::new(Arc::new(Min2PhaseSolver::new()));
            match five.cache_dir() {
                Some(dir) => eprintln!(
                    "preparing the 5x5x5 tables (cache directory {})",
                    dir.display()
                ),
                None => eprintln!("preparing the 5x5x5 tables (no cache directory: built now)"),
            }
            five.prepare();
            if five.cache_write_failed() == Some(true) {
                eprintln!("warning: the tables could not be saved; they are built again next time");
            }
            Box::new(five)
        }
        PuzzleKind::Cube6 | PuzzleKind::Cube7 => {
            let three = Arc::new(Min2PhaseSolver::new());
            let four = FourSolver::new(three.clone());
            let five = FiveSolver::new(three.clone());
            eprintln!(
                "preparing the 4x4x4 and 5x5x5 tables for the wings (the center tables are built on first use)"
            );
            four.prepare();
            five.prepare();
            Box::new(DwaltonSolver::new(
                kind,
                three,
                Arc::new(four),
                Arc::new(five),
            ))
        }
    }
}

/// The limits `solve` applies without options: 21 moves for the 2x2x2 and 3x3x3.
pub(crate) fn default_options(kind: PuzzleKind) -> SolveOptions {
    match kind {
        PuzzleKind::Cube2 | PuzzleKind::Cube3 => SolveOptions::default().with_max_moves(21),
        _ => SolveOptions::default(),
    }
}
