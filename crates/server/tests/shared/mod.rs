//! One 5x5x5 and one 4x4x4 solver per test binary. `AppState::with_engines`
//! prepares the tables of the solvers it is given, so a fresh solver per
//! request would build (or read) the 237 MB 5x5x5 tables again every time.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;

/// Tables read once, from a directory under the build directory (shared with
/// the other test binaries), never from the user's cache.
pub fn five() -> FiveSolver {
    static FIVE: OnceLock<FiveSolver> = OnceLock::new();
    FIVE.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
        FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(dir))
    })
    .clone()
}

/// A single-threaded 4x4x4 solver.
pub fn four() -> FourSolver {
    static FOUR: OnceLock<FourSolver> = OnceLock::new();
    FOUR.get_or_init(|| FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1))
        .clone()
}
