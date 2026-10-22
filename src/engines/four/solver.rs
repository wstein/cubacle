//! The 4x4x4 reduction solver behind the [`Solver`] trait.
//!
//! New code (the adapter around the ported phases): input checks, the
//! 3x3x3 hand-off through the injected solver, the final rotation, and the
//! replay of the answer.

use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::control::{Control, Stop};
use super::facelets;
use super::output::{of_face_turn, to_move, to_real};
use super::reduce::{after, reduce};
use super::rotation::Rotations;
use super::swap::Swaps;
use super::tables::Tables;
use crate::puzzle::normalise;
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver,
};
use crate::reduction::par;

/// The longest 3x3x3 hand-off asked for (the reference asks for 21).
const THREE_MAX_MOVES: u8 = 21;
/// Probes the 3x3x3 solver makes before it may answer (the reference: 500).
const HAND_OFF_PROBES: u64 = 500;
/// What the 3x3x3 stage may take when the caller sets no time budget.
const HAND_OFF_BUDGET: Duration = Duration::from_secs(10);
/// What the 3x3x3 stage may take when the caller sets a move limit: the
/// reduction is fixed, so a limit the tail cannot meet would otherwise keep
/// the 3x3x3 solver searching until the budget ends. A tail that can meet
/// the limit is found in a few milliseconds.
const LIMITED_HAND_OFF_BUDGET: Duration = Duration::from_secs(1);

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
    three: Arc<dyn Solver + Send + Sync>,
    tables: Arc<OnceLock<Tables>>,
    rotations: Arc<Rotations>,
    swaps: Arc<Swaps>,
    threads: usize,
    hand_off_probes: u64,
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

    fn tables(&self) -> &Tables {
        self.tables
            .get_or_init(|| Tables::build(par::threads(), self.layers))
    }
}

/// Whether the centres of every face show the face's own colour.
fn centres_home(cube: &Facelets) -> bool {
    let s = cube.stickers();
    Face::ALL
        .iter()
        .enumerate()
        .all(|(f, &face)| [5, 6, 9, 10].iter().all(|&i| s[f * 16 + i] == face))
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
        let reduction = match reduce(t, &cube, self.threads, &ctl) {
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

        let model_moves = reduction.model_moves();
        let model = after(t, &cube, &model_moves);
        let Some(three) = facelets::three_by_three(&model) else {
            return fail(
                Failure::Internal,
                "the reduced cube has no 3x3x3 (the centres are not solved)",
            );
        };
        let (length1, add1, sym) = (
            usize::from(reduction.cand.length1),
            reduction.cand.add1,
            usize::from(reduction.cand.sym),
        );
        let head = to_real(&t.syms, &model_moves, length1, add1, sym);
        let head_moves: Vec<Move> = head.iter().map(|&c| to_move(c)).collect();
        if options.reduce_only {
            // Keep the cube's frame: the big cubes lift these moves.
            let head = normalise(4, &head_moves, false);
            let Some(head) = self
                .swaps
                .finish_with(&self.rotations, facelets, &head, centres_home)
            else {
                return fail(Failure::Internal, "the reduced centres are not home");
            };
            return done(
                Ok(MoveSequence::new(normalise(4, &head, false))),
                None,
                None,
            );
        }
        let Ok(reduced) = facelets.apply(&MoveSequence::new(head_moves.clone())) else {
            return fail(Failure::Internal, "the reduction moves do not fit the cube");
        };
        let Some(rotation) = self.rotations.find(&reduced, centres_home) else {
            return fail(
                Failure::Internal,
                "the reduced cube is not a rotation of the standard colour scheme",
            );
        };

        let limit = options.max_moves.map(usize::from);
        let fixed = head_moves.len() + rotation.len();
        if limit.is_some_and(|max| fixed > max) {
            return fail(
                Failure::Exhausted,
                "the reduction and the final rotation alone are longer than the move limit",
            );
        }
        let remaining = match deadline {
            Some(d) => {
                let now = Instant::now();
                if now >= d {
                    return fail(
                        Failure::BudgetExhausted,
                        "the reduction used up the time budget before the 3x3x3 stage",
                    );
                }
                Some(d - now)
            }
            None => None,
        };
        let tail_limit = limit.map_or(THREE_MAX_MOVES, |max| {
            u8::try_from(max - fixed).map_or(THREE_MAX_MOVES, |n| n.min(THREE_MAX_MOVES))
        });
        // The cap on the 3x3x3 stage, if one applies, and whether it (not the
        // caller's deadline) is what ends the stage.
        let cap = if limit.is_some() {
            LIMITED_HAND_OFF_BUDGET
        } else {
            HAND_OFF_BUDGET
        };
        let stage_budget = match remaining {
            Some(r) => r.max(Duration::from_millis(1)).min(if limit.is_some() {
                cap
            } else {
                Duration::MAX
            }),
            None => cap,
        };
        let capped = remaining.is_none_or(|r| limit.is_some() && r > cap);
        let hand_off = SolveOptions {
            time_budget: Some(stage_budget),
            max_moves: Some(tail_limit),
            min_probes: options.min_probes.or(Some(self.hand_off_probes)),
            slices: false,
            reduce_only: false,
        };
        let finish = self.three.solve(&three, &hand_off);
        let probes = finish.metrics.probes;
        let tail = match finish.outcome {
            Ok(tail) => tail,
            Err(Failure::BudgetExhausted) if capped => {
                return done(
                    Err(Failure::Exhausted),
                    Some(format!(
                        "no 3x3x3 solution within the move limit was found in {} s{}",
                        cap.as_secs(),
                        if limit.is_some() {
                            " (the reduction is fixed; a larger limit may help)"
                        } else {
                            " (set a time budget to allow longer)"
                        }
                    )),
                    probes,
                );
            }
            Err(failure) => {
                let message = finish.message.unwrap_or_else(|| "the 3x3x3 failed".into());
                return done(
                    Err(failure),
                    Some(format!("3x3x3 stage: {message}")),
                    probes,
                );
            }
        };
        let mut all = model_moves;
        for mv in tail.moves() {
            let Some(code) = of_face_turn(mv) else {
                return fail(
                    Failure::Internal,
                    "the 3x3x3 stage returned a move this cube cannot make as a 3x3x3 turn",
                );
            };
            all.push(code);
        }
        let real = to_real(&t.syms, &all, length1, add1, sym);
        let moves: Vec<Move> = real.iter().map(|&c| to_move(c)).collect();
        let moves = normalise(4, &moves, false);
        let Some(moves) = self.swaps.finish(&self.rotations, facelets, &moves) else {
            return fail(
                Failure::Internal,
                "the solution does not solve the cube (the 3x3x3 stage was given an unsolvable state?)",
            );
        };
        let solution = MoveSequence::new(normalise(4, &moves, options.slices));
        if limit.is_some_and(|max| solution.len() > max) {
            return fail(
                Failure::Exhausted,
                "the solution is longer than the move limit",
            );
        }
        if !facelets.apply(&solution).is_ok_and(|end| end.is_solved()) {
            return fail(Failure::Internal, "the solution does not solve the cube");
        }
        done(Ok(solution), None, probes)
    }
}
