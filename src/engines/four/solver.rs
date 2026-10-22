//! The 4x4x4 reduction solver behind the [`Solver`] trait.
//!
//! New code (the adapter around the ported phases): input checks, the
//! 3x3x3 hand-off through the injected solver, the final rotation, and the
//! replay of the answer.

use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use super::control::{Control, Stop};
use super::facelets;
use super::reduce::reduce;
use super::reduce::reduce_many;
use super::rotation::Rotations;
use super::swap::Swaps;
use super::tables::Tables;
use crate::puzzle::{
    Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};
use crate::reduction::improve::{self, Plan};
use crate::reduction::par;

/// Probes the 3x3x3 solver makes before it may answer (the reference: 500).
const HAND_OFF_PROBES: u64 = 500;
/// The 4x4x4 reduction solver, a port of Tsai's three-phase solver
/// cs0x7f/TPR-4x4x4-Solver.
///
/// Three searches bring the centres and wings into place (about 25 moves),
/// then the solver it was built with solves the 3x3x3 that is left (about
/// 20). The 3x3x3 solver is injected through the [`Solver`] trait. Without
/// fixed centres a cube can be solved in any of 24 orientations: the answer
/// ends with the whole-cube rotation to the standard one, written as at most
/// four wide turns (`Rw Lw'` is `x`), so that it solves the cube exactly.
///
/// The tables (about 12 MB) belong to the instance and are shared by its
/// clones; they are built on first use or by [`Solver::prepare`]. The number
/// of threads is `CUBACLE_THREADS` or the available parallelism; without a
/// time budget the solution does not depend on it.
#[derive(Clone)]
pub struct FourSolver {
    pub(super) three: Arc<dyn Solver + Send + Sync>,
    tables: Arc<OnceLock<Tables>>,
    pub(super) rotations: Arc<Rotations>,
    pub(super) swaps: Arc<Swaps>,
    threads: usize,
    pub(super) hand_off_probes: u64,
    layers: u32,
}

impl fmt::Debug for FourSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FourSolver")
            .field("tables_ready", &self.tables_ready())
            .field("threads", &self.threads)
            .field("hand_off_probes", &self.hand_off_probes)
            .finish()
    }
}

impl FourSolver {
    /// A solver that finishes with `three`, which must handle the 3x3x3.
    pub fn new(three: Arc<dyn Solver + Send + Sync>) -> FourSolver {
        let rotations = Arc::new(Rotations::new());
        FourSolver {
            three,
            tables: Arc::new(OnceLock::new()),
            rotations: Arc::clone(&rotations),
            swaps: Arc::new(Swaps::new(&rotations)),
            threads: par::threads(),
            hand_off_probes: HAND_OFF_PROBES,
            layers: super::edge3_prune::FULL_LAYERS,
        }
    }

    /// Searches with `threads` workers instead of the configured number. The
    /// tables are always built with the configured number.
    pub fn with_threads(mut self, threads: usize) -> FourSolver {
        self.threads = threads.max(1);
        self
    }

    /// Asks the 3x3x3 solver for this many probes before it may answer
    /// (default 500: shorter hand-offs, a little more time). 0 takes its
    /// first solution.
    pub fn with_hand_off_probes(mut self, probes: u64) -> FourSolver {
        self.hand_off_probes = probes;
        self
    }

    /// Smaller wing table for tests.
    #[cfg(test)]
    pub(super) fn with_layers(mut self, layers: u32) -> FourSolver {
        self.layers = layers;
        self.tables = Arc::new(OnceLock::new());
        self
    }

    /// Whether the tables are in memory, so that a solve starts at once.
    pub fn tables_ready(&self) -> bool {
        self.tables.get().is_some()
    }

    /// Bytes held by the big tables, once they are in memory.
    pub fn table_bytes(&self) -> Option<usize> {
        self.tables.get().map(Tables::bytes)
    }

    /// The number of worker threads.
    pub fn threads(&self) -> usize {
        self.threads
    }

    pub(super) fn tables(&self) -> &Tables {
        self.tables
            .get_or_init(|| Tables::build(par::threads(), self.layers))
    }
}

impl Solver for FourSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube4
    }

    fn prepare(&self) {
        self.tables();
        self.three.prepare();
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        self.run(facelets, options, None)
    }

    fn solve_improving(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let budget = options.time_budget.unwrap_or(improve::DEFAULT_BUDGET);
        let options = options.with_time_budget(budget);
        self.run(facelets, &options, Some(improve::plan(budget)))
    }
}

impl FourSolver {
    /// One solve; with a `plan`, the best of several reductions.
    pub(super) fn run(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        plan: Option<Plan>,
    ) -> SolveResult {
        let started = Instant::now();
        let done = |outcome: Result<MoveSequence, Failure>,
                    message: Option<String>,
                    probes: Option<u64>| SolveResult {
            outcome,
            message,
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes,
            },
        };
        let fail =
            |failure: Failure, message: &str| done(Err(failure), Some(message.to_owned()), None);

        if facelets.kind() != PuzzleKind::Cube4 {
            let size = facelets.kind().size();
            return fail(
                Failure::InvalidInput,
                &format!("expected a 4x4x4, got a {size}x{size}x{size}"),
            );
        }
        if self.three.kind() != PuzzleKind::Cube3 {
            return fail(
                Failure::Internal,
                "the solver for the 3x3x3 stage does not solve 3x3x3 cubes",
            );
        }
        let cube = match facelets::read(facelets) {
            Ok(cube) => cube,
            Err(why) => return fail(Failure::InvalidInput, why),
        };

        let deadline = options.time_budget.map(|budget| started + budget);
        let cold = !self.tables_ready();
        let t = self.tables();
        let ctl = Control::new(deadline);
        if ctl.check() {
            return fail(
                Failure::BudgetExhausted,
                if cold {
                    "building the 4x4x4 tables (first use; prepare() builds them ahead) used up the time budget"
                } else {
                    "the time budget was used up before the search started"
                },
            );
        }
        let reductions = match plan {
            None => reduce(t, &cube, self.threads, &ctl).map(|r| vec![r]),
            Some(plan) => {
                let early = Control::new(deadline.map(|d| d.min(started + plan.collect)));
                reduce_many(t, &cube, self.threads, (&ctl, &early), plan.candidates)
            }
        };
        let reductions = match reductions {
            Ok(r) => r,
            Err(Stop::Timeout) => {
                return fail(
                    Failure::BudgetExhausted,
                    "the reduction ran out of time before the 3x3x3 stage",
                )
            }
            Err(Stop::Exhausted(why)) => return fail(Failure::Exhausted, why),
            Err(Stop::Internal(why)) => return fail(Failure::Internal, why),
        };

        let outcome = match plan {
            None => self.finish(t, &cube, facelets, &reductions[0], options, deadline),
            Some(plan) => {
                self.finish_best(t, &cube, facelets, &reductions, options, deadline, plan)
            }
        };
        match outcome {
            Ok((solution, probes)) => done(Ok(solution), None, probes),
            Err(miss) => done(Err(miss.failure), Some(miss.message), miss.probes),
        }
    }
}
