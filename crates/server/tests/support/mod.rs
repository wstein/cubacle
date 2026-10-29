//! Shared immutable engines, not shared request limits, for server tests.

use std::sync::{Arc, OnceLock};

use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle_server::{AppState, Config};

/// Prepare the uncached tables once per test binary. Engine clones share the
/// tables; each state still owns a fresh config and concurrency semaphore.
pub(crate) fn state(config: Config) -> AppState {
    static ENGINES: OnceLock<(FiveSolver, FourSolver)> = OnceLock::new();
    let (five, four) = ENGINES.get_or_init(|| {
        let three = Arc::new(Min2PhaseSolver::shared());
        (
            FiveSolver::new(three.clone()).with_cache_dir(None),
            FourSolver::new(three).with_threads(1),
        )
    });
    AppState::with_engines(
        config,
        Min2PhaseSolver::shared(),
        five.clone(),
        four.clone(),
    )
}
