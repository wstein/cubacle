use std::time::{Duration, Instant};

use super::{
    solve_batch, solve_parallel_with_time_limits, solve_pattern, solve_pattern_parallel,
    solve_with_stats, solve_with_time_limits, SolveOutcome, SolveStats,
};
use crate::puzzle::{
    Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};

/// The longest solution the engine searches for when no limit is given.
const DEFAULT_MAX_MOVES: u8 = 21;

/// The 3x3x3 two-phase engine behind the [`Solver`] trait.
#[derive(Debug, Clone, Copy, Default)]
pub struct Min2PhaseSolver;

impl Min2PhaseSolver {
    pub fn new() -> Min2PhaseSolver {
        Min2PhaseSolver
    }

    /// Finds moves that turn `from` into `to`, within `options.max_moves`.
    pub fn solve_pattern(
        &self,
        from: &Facelets,
        to: &Facelets,
        options: &SolveOptions,
    ) -> SolveResult {
        self.pattern(from, to, options, solve_pattern)
    }

    /// Like [`Min2PhaseSolver::solve_pattern`], searching the six URF
    /// orientations concurrently on the process-wide worker pool.
    pub fn solve_pattern_parallel(
        &self,
        from: &Facelets,
        to: &Facelets,
        options: &SolveOptions,
    ) -> SolveResult {
        self.pattern(from, to, options, solve_pattern_parallel)
    }

    /// Solves independent cubes concurrently; results are in input order.
    ///
    /// Intended for throughput. It uses a persistent process-wide worker pool,
    /// whose size is read once from `CUBACLE_THREADS` on first use (otherwise
    /// the available parallelism). Calls made from one of those workers run
    /// serially, to avoid nested-pool oversubscription. A time budget in
    /// `options` is ignored here; every cube runs to completion.
    pub fn solve_batch(&self, cubes: &[Facelets], options: &SolveOptions) -> Vec<SolveResult> {
        let started = Instant::now();
        let texts: Vec<String> = cubes
            .iter()
            .filter(|cube| cube.kind() == PuzzleKind::Cube3)
            .map(Facelets::to_string)
            .collect();
        let mut solved = solve_batch(&texts, max_moves(options)).into_iter();
        cubes
            .iter()
            .map(|cube| match wrong_size(cube, started) {
                Some(rejected) => rejected,
                None => into_result(solved.next().expect("one result per cube"), started),
            })
            .collect()
    }

    /// Searches one cube's six URF orientations concurrently.
    ///
    /// The winner is chosen deterministically. For throughput on many cubes,
    /// prefer [`Min2PhaseSolver::solve_batch`].
    pub fn solve_parallel(&self, cube: &Facelets, options: &SolveOptions) -> SolveResult {
        self.timed(cube, options, |text, max, budget| {
            solve_parallel_with_time_limits(text, max, Duration::ZERO, budget)
        })
    }

    /// Like [`Min2PhaseSolver::solve`], but keeps improving the solution until
    /// `minimum` has elapsed. The minimum only applies together with a time
    /// budget, and is clamped to it.
    pub fn solve_with_minimum_time(
        &self,
        cube: &Facelets,
        options: &SolveOptions,
        minimum: Duration,
    ) -> SolveResult {
        self.timed(cube, options, |text, max, budget| {
            solve_with_time_limits(text, max, minimum, budget)
        })
    }

    /// The parallel counterpart of [`Min2PhaseSolver::solve_with_minimum_time`].
    pub fn solve_parallel_with_minimum_time(
        &self,
        cube: &Facelets,
        options: &SolveOptions,
        minimum: Duration,
    ) -> SolveResult {
        self.timed(cube, options, |text, max, budget| {
            solve_parallel_with_time_limits(text, max, minimum, budget)
        })
    }

    fn timed(
        &self,
        cube: &Facelets,
        options: &SolveOptions,
        run: impl FnOnce(&str, u8, Option<Duration>) -> SolveStats,
    ) -> SolveResult {
        let started = Instant::now();
        if let Some(rejected) = wrong_size(cube, started) {
            return rejected;
        }
        let stats = run(&cube.to_string(), max_moves(options), options.time_budget);
        into_result(stats, started)
    }

    fn pattern(
        &self,
        from: &Facelets,
        to: &Facelets,
        options: &SolveOptions,
        run: impl FnOnce(&str, &str, u8) -> SolveStats,
    ) -> SolveResult {
        let started = Instant::now();
        for cube in [from, to] {
            if let Some(rejected) = wrong_size(cube, started) {
                return rejected;
            }
        }
        let stats = run(&from.to_string(), &to.to_string(), max_moves(options));
        into_result(stats, started)
    }
}

impl Solver for Min2PhaseSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn prepare(&self) {
        // The engine builds its tables on first use; a trivial solve forces them.
        let _ = solve_with_stats(&Facelets::solved(PuzzleKind::Cube3).to_string(), 1);
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        self.timed(facelets, options, |text, max, budget| {
            solve_with_time_limits(text, max, Duration::ZERO, budget)
        })
    }
}

fn max_moves(options: &SolveOptions) -> u8 {
    options.max_moves.unwrap_or(DEFAULT_MAX_MOVES)
}

fn wrong_size(cube: &Facelets, started: Instant) -> Option<SolveResult> {
    (cube.kind() != PuzzleKind::Cube3).then(|| {
        failed(
            Failure::InvalidInput,
            format!("expected a 3x3x3, got a {0}x{0}x{0}", cube.kind().size()),
            started,
            None,
        )
    })
}

fn into_result(stats: SolveStats, started: Instant) -> SolveResult {
    let probes = Some(stats.probes);
    if !stats.solution.starts_with("Error") {
        return match stats.solution.parse::<MoveSequence>() {
            Ok(solution) => SolveResult {
                outcome: Ok(solution),
                message: None,
                metrics: metrics(started, probes),
            },
            Err(error) => failed(
                Failure::InvalidInput,
                format!("engine produced unreadable moves: {error}"),
                started,
                probes,
            ),
        };
    }

    let failure = match stats.outcome {
        SolveOutcome::DeadlineNoSolution => Failure::BudgetExhausted,
        _ if is_invalid_cube(&stats.solution) => Failure::InvalidInput,
        _ => Failure::Exhausted,
    };
    failed(failure, describe_error(&stats.solution), started, probes)
}

/// Turns an engine `Error <code>` result into a stable sentence.
///
/// The codes come from `Cubie::set_from_facelet` / `Cubie::verify` (1 to 6)
/// and from the search (8). Code 7 is not produced by the engine; anything
/// unrecognised is passed through unchanged.
fn describe_error(error: &str) -> String {
    let code = error
        .strip_prefix("Error ")
        .and_then(|code| code.trim().parse::<u8>().ok());
    let text = match code {
        Some(1) => "the facelets are malformed: each colour must appear exactly nine times",
        Some(2) => "the edges are invalid: not every edge piece appears exactly once",
        Some(3) => "the edges are invalid: one edge is flipped on its own",
        Some(4) => "the corners are invalid: not every corner piece appears exactly once",
        Some(5) => "the corners are invalid: one corner is twisted on its own",
        Some(6) => "the cube is invalid: two pieces are swapped (parity error)",
        Some(8) => "the search ended without a solution (time budget or move limit reached)",
        _ => return error.to_owned(),
    };
    text.to_owned()
}

/// Engine error codes 1 to 6 describe an impossible cube; the rest are limits.
fn is_invalid_cube(error: &str) -> bool {
    error
        .strip_prefix("Error ")
        .and_then(|code| code.trim().parse::<u8>().ok())
        .is_some_and(|code| (1..=6).contains(&code))
}

fn metrics(started: Instant, probes: Option<u64>) -> SolveMetrics {
    SolveMetrics {
        elapsed: started.elapsed(),
        probes,
    }
}

fn failed(failure: Failure, message: String, started: Instant, probes: Option<u64>) -> SolveResult {
    SolveResult {
        outcome: Err(failure),
        message: Some(message),
        metrics: metrics(started, probes),
    }
}

#[cfg(test)]
mod describe_tests {
    use super::describe_error;

    #[test]
    fn every_known_code_has_a_sentence() {
        for code in [1, 2, 3, 4, 5, 6, 8] {
            let text = describe_error(&format!("Error {code}"));
            assert!(!text.starts_with("Error"), "code {code}: {text}");
        }
        assert!(describe_error("Error 2").contains("edge"));
        assert!(describe_error("Error 5").contains("twisted"));
    }

    #[test]
    fn unknown_codes_pass_through() {
        assert_eq!(describe_error("Error 7"), "Error 7");
        assert_eq!(describe_error("Error 99"), "Error 99");
    }
}
