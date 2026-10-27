//! Server configuration, limits and the shared state.

use std::sync::Arc;
use std::time::Duration;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::puzzle::{PuzzleKind, Solver};
use tokio::sync::Semaphore;

/// Longest time budget a solve may be given (10 minutes).
pub const MAX_TIMEOUT_MS: u64 = 600_000;
/// Largest `--max-concurrency`. Stays well below tokio's blocking-thread limit,
/// so a permit holder never waits for a thread and its budget clock starts on time.
pub const MAX_CONCURRENCY: usize = 256;
/// Largest `minProbes` (the budget bounds the time anyway).
pub(crate) const MAX_MIN_PROBES: u64 = 100_000_000;
/// Largest `maxMoves` the 3x3 engine honors (it silently clamps above this).
pub const MAX_MOVES: u8 = 25;
/// Largest `maxMoves` for the 5x5x5 (a byte; its solutions have about 73 moves).
pub const MAX_MOVES_5X5: u8 = u8::MAX;
/// Largest `maxMoves` for the 4x4x4 (a byte; its solutions have about 48 moves).
pub const MAX_MOVES_4X4: u8 = u8::MAX;

/// Runtime limits of the server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Solves allowed to run at the same time.
    pub max_concurrency: usize,
    /// Upper bound for the time one solve may take; also the default budget.
    pub request_timeout: Duration,
    /// Largest accepted request body.
    pub max_body_bytes: usize,
    /// How long a client may take to send the request head (slowloris guard).
    pub header_read_timeout: Duration,
    /// Extra time, beyond `request_timeout`, that in-flight requests get
    /// after a shutdown signal before connections are dropped.
    pub shutdown_grace: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_concurrency: std::thread::available_parallelism().map_or(1, |n| n.get()),
            request_timeout: Duration::from_secs(30),
            max_body_bytes: 16 * 1024,
            header_read_timeout: Duration::from_secs(10),
            shutdown_grace: Duration::from_secs(5),
        }
    }
}

/// Everything the handlers share. Cloning is cheap.
#[derive(Clone)]
pub struct AppState {
    pub(crate) permits: Arc<Semaphore>,
    pub(crate) config: Arc<Config>,
    /// The 3x3x3 engine; clones share its tables.
    pub(crate) three: Min2PhaseSolver,
    /// The 5x5x5 engine, finishing with a clone of `three`.
    pub(crate) five: FiveSolver,
    /// The 4x4x4 engine, finishing with a clone of `three`; one thread per
    /// solve, so concurrent requests do not compete for cores.
    pub(crate) four: FourSolver,
    /// The 6x6x6 engine, built on clones of the others.
    pub(crate) six: Arc<DwaltonSolver>,
    /// The 7x7x7 engine, likewise.
    pub(crate) seven: Arc<DwaltonSolver>,
}

/// The engine for a big cube over clones of the engines it finishes with.
pub(crate) fn big_of(
    kind: PuzzleKind,
    three: &Min2PhaseSolver,
    four: &FourSolver,
    five: &FiveSolver,
) -> Arc<DwaltonSolver> {
    Arc::new(DwaltonSolver::new(
        kind,
        Arc::new(three.clone()),
        Arc::new(four.clone()),
        Arc::new(five.clone()),
    ))
}

impl AppState {
    /// Builds the state and prepares all engines (tables), so a
    /// router made from it is ready to answer the moment it exists. The 3x3x3
    /// engine is the process-wide default instance.
    pub fn new(config: Config) -> AppState {
        AppState::with_solver(config, Min2PhaseSolver::shared())
    }

    /// Like [`AppState::new`], with the 3x3x3 engine injected.
    pub fn with_solver(config: Config, three: Min2PhaseSolver) -> AppState {
        let five = FiveSolver::new(Arc::new(three.clone()));
        let four = FourSolver::new(Arc::new(three.clone())).with_threads(1);
        AppState::with_engines(config, three, five, four)
    }

    /// Like [`AppState::with_solver`], with the 5x5x5 and 4x4x4 engines injected
    /// too (a clone shares its tables, so tests can build them once and keep
    /// the 5x5x5 cache directory private).
    pub fn with_engines(
        config: Config,
        three: Min2PhaseSolver,
        five: FiveSolver,
        four: FourSolver,
    ) -> AppState {
        PocketSolver::new().prepare();
        let state = AppState {
            permits: Arc::new(Semaphore::new(config.max_concurrency.max(1))),
            config: Arc::new(config),
            six: big_of(PuzzleKind::Cube6, &three, &four, &five),
            seven: big_of(PuzzleKind::Cube7, &three, &four, &five),
            four,
            five,
            three,
        };
        state.three.prepare();
        state.five.prepare();
        state.four.prepare();
        state
    }

    /// The state without table construction (tests that do not solve).
    pub fn unprepared(config: Config) -> AppState {
        AppState::assemble(config, Min2PhaseSolver::shared())
    }

    pub(crate) fn assemble(config: Config, three: Min2PhaseSolver) -> AppState {
        let five = FiveSolver::new(Arc::new(three.clone()));
        let four = FourSolver::new(Arc::new(three.clone())).with_threads(1);
        AppState {
            permits: Arc::new(Semaphore::new(config.max_concurrency.max(1))),
            config: Arc::new(config),
            six: big_of(PuzzleKind::Cube6, &three, &four, &five),
            seven: big_of(PuzzleKind::Cube7, &three, &four, &five),
            five,
            four,
            three,
        }
    }

    /// The semaphore guarding the compute pool, exposed so tests can saturate it.
    pub fn permits(&self) -> &Arc<Semaphore> {
        &self.permits
    }
}
