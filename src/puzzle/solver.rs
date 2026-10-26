use std::time::Duration;

use super::{Facelets, MoveSequence, PuzzleKind, SolveStage, Target};

/// Limits for one solve. The defaults impose none beyond the engine's own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SolveOptions {
    /// A hard wall-clock limit.
    pub time_budget: Option<Duration>,
    /// The longest solution to accept, in moves.
    pub max_moves: Option<u8>,
    /// Keep improving a solution until this many probes have been made, for
    /// engines that count probes (the 3x3x3 engine does). It counts work,
    /// not time, so the answer is the same on every machine. Other engines
    /// ignore it.
    pub min_probes: Option<u64>,
    /// Allow single inner layers in the answer (`2R`) where they make it
    /// shorter. The reduction engines (4x4x4, 5x5x5) otherwise write wide
    /// turns and outer layers only (5x5x5: also second layers). Engines for
    /// other sizes ignore it.
    pub slices: bool,
    /// Stop when the cube is reduced to a 3x3x3 and answer with the moves
    /// that reduce it (centers solved, wings paired), not with a solution.
    /// For engines that build on another reduction engine; the 4x4x4 and
    /// 5x5x5 honor it, the others ignore it.
    pub reduce_only: bool,
}

impl SolveOptions {
    pub fn with_time_budget(mut self, budget: Duration) -> SolveOptions {
        self.time_budget = Some(budget);
        self
    }

    pub fn with_max_moves(mut self, moves: u8) -> SolveOptions {
        self.max_moves = Some(moves);
        self
    }

    pub fn with_min_probes(mut self, probes: u64) -> SolveOptions {
        self.min_probes = Some(probes);
        self
    }

    pub fn with_slices(mut self, slices: bool) -> SolveOptions {
        self.slices = slices;
        self
    }

    pub fn with_reduce_only(mut self, reduce_only: bool) -> SolveOptions {
        self.reduce_only = reduce_only;
        self
    }
}

/// What the search measured about itself.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SolveMetrics {
    pub elapsed: Duration,
    /// Final-phase probes, for engines that count them.
    pub probes: Option<u64>,
}

/// Why a solve ended without a solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// The state cannot exist, or does not match the solver's cube size.
    InvalidInput,
    /// A reachable state whose face centers are not on their own faces (a
    /// middle-slice turn moved them), for a solver that needs them in place.
    CentersOffFace,
    /// The time budget ran out before a solution was found.
    BudgetExhausted,
    /// The space was searched out within the move limit and holds no solution.
    Exhausted,
    /// The engine failed itself, not the input: a bug, or a solver that was
    /// wired up wrongly (for example a 5x5x5 engine given a solver that does
    /// not handle the 3x3x3).
    Internal,
}

/// The outcome of one solve.
///
/// A solved result always means the cube is fully solved by `solution`, which
/// contains only layer turns, never whole-cube rotations. The exception is a
/// solve with [`SolveOptions::reduce_only`], whose moves only reduce the cube.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveResult {
    pub outcome: Result<MoveSequence, Failure>,
    pub message: Option<String>,
    pub metrics: SolveMetrics,
    /// The engine's own stages, in order, covering the solution exactly (see
    /// [`SolveStage`]); empty when the engine reports none. Their ends are
    /// positions in the final `solution`, after the engine's normalizing.
    pub stages: Vec<SolveStage>,
}

impl SolveResult {
    pub fn is_solved(&self) -> bool {
        self.outcome.is_ok()
    }

    pub fn solution(&self) -> Option<&MoveSequence> {
        self.outcome.as_ref().ok()
    }
}

/// An engine for one cube size.
///
/// Instances may keep mutable per-search state, so one solver serves one
/// caller at a time. The expensive part of an engine, its pruning tables, is
/// immutable once built and shared between instances.
pub trait Solver {
    /// The cube size this solver handles.
    fn kind(&self) -> PuzzleKind;

    /// Builds whatever the solver needs before its first solve, typically its
    /// pruning tables. Optional: an unprepared solver prepares itself on first
    /// use. Idempotent and safe to call concurrently.
    fn prepare(&self) {}

    /// Solves a cube. A failure is reported in the result, never by panicking.
    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult;

    /// Like [`Solver::solve`], but spends the time budget (default one
    /// second) looking for a shorter answer than the first. Engines without
    /// such a mode answer as `solve` does. The reduction engines (4x4x4,
    /// 5x5x5) keep several reductions and finish each; what is counted
    /// rather than timed is described on the engine.
    fn solve_improving(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        self.solve(facelets, options)
    }

    /// Finds moves that take `start` to any state of a target that may leave
    /// stickers open. The result's message says whether the answer is proven
    /// shortest ("shortest: ...") or the best found ("best found: ...").
    ///
    /// The default fails with [`Failure::InvalidInput`] and a message that the
    /// engine cannot solve partial targets; engines override it as they gain
    /// support. A complete target is solved by [`Solver::solve`] on the
    /// relative state, never by this method.
    fn solve_to(&self, start: &Facelets, target: &Target, options: &SolveOptions) -> SolveResult {
        let _ = (start, options);
        let size = target.kind().size();
        SolveResult {
            outcome: Err(Failure::InvalidInput),
            message: Some(format!(
                "unsupported: the {size}x{size}x{size} engine cannot solve targets with open stickers yet"
            )),
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Stub;

    impl Solver for Stub {
        fn kind(&self) -> PuzzleKind {
            PuzzleKind::Cube3
        }

        fn solve(&self, _: &Facelets, _: &SolveOptions) -> SolveResult {
            unreachable!("not called")
        }
    }

    #[test]
    fn the_default_solve_to_says_it_is_unsupported() {
        let start = Facelets::solved(PuzzleKind::Cube3);
        let target = Target::masked(&start, |position| position < 9);
        let result = Stub.solve_to(&start, &target, &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
        let message = result.message.unwrap();
        assert!(message.contains("unsupported") && message.contains("3x3x3"));
    }
}
