use std::time::{Duration, Instant};

use std::fmt;
use std::sync::OnceLock;

use super::engine::{
    solve_batch, solve_parallel_with_time_limits, solve_pattern, solve_pattern_parallel,
    solve_with_limits, solve_with_time_limits,
};
use super::stats::{SolveOutcome, SolveStats};
use super::tables::{self, SharedTables};
use crate::puzzle::{
    Face, Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult,
    Solver, Target,
};

/// The longest solution the engine searches for when no limit is given.
const DEFAULT_MAX_MOVES: u8 = 21;

/// The 3x3x3 two-phase engine behind the [`Solver`] trait.
///
/// The solver owns its pruning tables, which are built on first use (or by
/// [`Solver::prepare`]) and are immutable afterwards. Clones share them, so
/// build one solver at startup and hand clones to whoever needs one;
/// [`Min2PhaseSolver::new`] starts a table set of its own.
#[derive(Clone)]
pub struct Min2PhaseSolver {
    tables: SharedTables,
}

impl fmt::Debug for Min2PhaseSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Min2PhaseSolver")
            .field("tables_built", &self.tables.get().is_some())
            .finish()
    }
}

impl Default for Min2PhaseSolver {
    /// The same as [`Min2PhaseSolver::new`]: a table set of its own.
    fn default() -> Min2PhaseSolver {
        Min2PhaseSolver::new()
    }
}

impl Min2PhaseSolver {
    /// A solver with a table set of its own, not yet built.
    pub fn new() -> Min2PhaseSolver {
        Min2PhaseSolver {
            tables: tables::fresh(),
        }
    }

    /// A handle on the process-wide default solver, created on first call.
    ///
    /// For code that has no solver to inject, such as a one-off tool. Library
    /// code should take a solver as a parameter instead.
    pub fn shared() -> Min2PhaseSolver {
        static DEFAULT: OnceLock<Min2PhaseSolver> = OnceLock::new();
        DEFAULT.get_or_init(Min2PhaseSolver::new).clone()
    }

    /// Finds moves that turn `from` into `to`, within `options.max_moves`.
    pub fn solve_pattern(
        &self,
        from: &Facelets,
        to: &Facelets,
        options: &SolveOptions,
    ) -> SolveResult {
        self.pattern(from, to, options, |from, to, max| {
            solve_pattern(&self.tables, from, to, max)
        })
    }

    /// Like [`Min2PhaseSolver::solve_pattern`], searching the six URF
    /// orientations concurrently on the process-wide worker pool.
    pub fn solve_pattern_parallel(
        &self,
        from: &Facelets,
        to: &Facelets,
        options: &SolveOptions,
    ) -> SolveResult {
        self.pattern(from, to, options, |from, to, max| {
            solve_pattern_parallel(&self.tables, from, to, max)
        })
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
            .filter(|cube| wrong_size(cube, started).is_none())
            .map(Facelets::to_string)
            .collect();
        let mut solved = solve_batch(&self.tables, &texts, max_moves(options)).into_iter();
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
            solve_parallel_with_time_limits(&self.tables, text, max, Duration::ZERO, budget)
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
            solve_with_time_limits(&self.tables, text, max, minimum, budget)
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
            solve_parallel_with_time_limits(&self.tables, text, max, minimum, budget)
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
        tables::get(&self.tables);
    }

    fn solve_to(&self, start: &Facelets, target: &Target, options: &SolveOptions) -> SolveResult {
        super::partial::solve_to(self, start, target, options)
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let probe_min = options.min_probes.unwrap_or(0);
        self.timed(facelets, options, |text, max, budget| {
            solve_with_limits(&self.tables, text, max, Duration::ZERO, budget, probe_min)
        })
    }
}

fn max_moves(options: &SolveOptions) -> u8 {
    options.max_moves.unwrap_or(DEFAULT_MAX_MOVES)
}

/// Why this engine cannot take `cube`, if it cannot.
///
/// Besides the size, the centers must be in their standard places: wide and
/// inner-layer moves can move them, and the engine neither solves such a cube
/// nor would its solution (layer turns only) put them back.
fn wrong_size(cube: &Facelets, started: Instant) -> Option<SolveResult> {
    if cube.kind() != PuzzleKind::Cube3 {
        return Some(failed(
            Failure::InvalidInput,
            format!("expected a 3x3x3, got a {0}x{0}x{0}", cube.kind().size()),
            started,
            None,
        ));
    }
    let centers_home = Face::ALL
        .into_iter()
        .all(|face| cube.stickers()[face as usize * 9 + 4] == face);
    (!centers_home).then(|| {
        let (failure, why) = crate::puzzle::centers_off_face(cube);
        failed(failure, why, started, None)
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
                stages: Vec::new(),
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
/// unrecognized is passed through unchanged.
fn describe_error(error: &str) -> String {
    let code = error
        .strip_prefix("Error ")
        .and_then(|code| code.trim().parse::<u8>().ok());
    let text = match code {
        Some(1) => "the facelets are malformed: each color must appear exactly nine times",
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
        ..SolveMetrics::default()
    }
}

fn failed(failure: Failure, message: String, started: Instant, probes: Option<u64>) -> SolveResult {
    SolveResult {
        outcome: Err(failure),
        message: Some(message),
        metrics: metrics(started, probes),
        stages: Vec::new(),
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

#[cfg(test)]
mod ownership_tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn clones_share_tables_and_new_instances_do_not() {
        let solver = Min2PhaseSolver::new();
        assert!(Arc::ptr_eq(&solver.tables, &solver.clone().tables));
        assert!(!Arc::ptr_eq(&solver.tables, &Min2PhaseSolver::new().tables));
        assert!(Arc::ptr_eq(
            &Min2PhaseSolver::shared().tables,
            &Min2PhaseSolver::shared().tables
        ));
    }

    #[test]
    fn tables_are_built_lazily_and_by_prepare() {
        let solver = Min2PhaseSolver::new();
        assert!(solver.tables.get().is_none());
        let invalid = Facelets::solved(PuzzleKind::Cube2);
        let _ = solver.solve(&invalid, &SolveOptions::default());
        assert!(
            solver.tables.get().is_none(),
            "a rejected input builds nothing"
        );
        solver.prepare();
        assert!(solver.tables.get().is_some());
    }

    #[test]
    fn independent_solvers_agree() {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R U F' D2 L B".parse().unwrap())
            .unwrap();
        let options = SolveOptions::default();
        let first = Min2PhaseSolver::new().solve(&cube, &options).outcome;
        let second = Min2PhaseSolver::new().solve(&cube, &options).outcome;
        assert!(first.is_ok());
        assert_eq!(first, second);
    }
}

#[cfg(test)]
mod center_tests {
    use super::*;

    fn scrambled(moves: &str) -> Facelets {
        Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves.parse().unwrap())
            .unwrap()
    }

    #[test]
    fn a_wide_move_moves_the_centers_and_is_rejected() {
        let solver = Min2PhaseSolver::new();
        for moves in ["Rw U", "2R", "2Rw' F"] {
            let result = solver.solve(&scrambled(moves), &SolveOptions::default());
            assert_eq!(result.outcome, Err(Failure::CentersOffFace), "{moves}");
            assert!(result.message.unwrap().contains("centers"), "{moves}");
        }
        let cube = scrambled("Rw U");
        let results = solver.solve_batch(std::slice::from_ref(&cube), &SolveOptions::default());
        assert_eq!(results[0].outcome, Err(Failure::CentersOffFace));
        let solved = Facelets::solved(PuzzleKind::Cube3);
        let pattern = solver.solve_pattern(&cube, &solved, &SolveOptions::default());
        assert_eq!(pattern.outcome, Err(Failure::CentersOffFace));
    }

    #[test]
    fn outer_face_moves_are_unaffected() {
        let cube = scrambled("R U R' U' F2 D L2 B");
        let result = Min2PhaseSolver::new().solve(&cube, &SolveOptions::default());
        assert!(cube.apply(result.solution().unwrap()).unwrap().is_solved());
    }
}

#[cfg(test)]
mod probe_tests {
    use super::*;

    const SCRAMBLE: &str = "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'";

    fn run(min_probes: Option<u64>) -> SolveResult {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&SCRAMBLE.parse().unwrap())
            .unwrap();
        let options = SolveOptions {
            min_probes,
            ..SolveOptions::default()
        };
        let result = Min2PhaseSolver::shared().solve(&cube, &options);
        assert!(cube.apply(result.solution().unwrap()).unwrap().is_solved());
        result
    }

    #[test]
    fn a_probe_minimum_keeps_improving_and_counts_work_not_time() {
        let first = run(None);
        let improved = run(Some(500));
        assert!(first.metrics.probes < Some(500));
        assert!(improved.metrics.probes >= Some(500));
        assert!(improved.solution().unwrap().len() <= first.solution().unwrap().len());
        // The same count gives the same answer, however long it takes.
        assert_eq!(run(Some(500)).outcome, improved.outcome);
        assert_eq!(run(Some(500)).metrics.probes, improved.metrics.probes);
    }
}
