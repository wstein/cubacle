//! The `solve` modes that look for a shorter 3x3x3 solution than the first.

use std::time::Duration;

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, SolveResult, Solver};

use crate::SolveArgs;

/// The budget `--improve` uses when `-b` is absent.
const DEFAULT_BUDGET_MS: u64 = 1000;

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    /// Improve until this many probes were made.
    Probes(u64),
    /// Improve until the budget is spent.
    Time,
    /// As `Time`, on the six orientations at once.
    TimeParallel,
}

pub(crate) fn mode(args: &SolveArgs) -> Option<Mode> {
    match (args.probes, args.improve, args.parallel) {
        (Some(n), _, _) => Some(Mode::Probes(n)),
        (None, true, true) => Some(Mode::TimeParallel),
        (None, true, false) => Some(Mode::Time),
        _ => None,
    }
}

/// Rejects improve flags that do not fit the cube or each other.
pub(crate) fn check(args: &SolveArgs, kind: PuzzleKind) -> Result<(), String> {
    if args.parallel && !args.improve {
        return Err("--parallel only works together with --improve".into());
    }
    if args.parallel && args.budget_ms.is_none() {
        return Err("--parallel needs a time budget (-b): without one it never returns".into());
    }
    if mode(args).is_none() {
        return Ok(());
    }
    match kind {
        PuzzleKind::Cube3 => Ok(()),
        PuzzleKind::Cube2 => {
            Err("--probes, --improve and --parallel do not apply to the 2x2x2: its solution is already optimal".into())
        }
        _ => Err(format!(
            "improve modes are not implemented yet for the {0}x{0}x{0}; they work on the 3x3x3 only",
            kind.size()
        )),
    }
}

/// Solves a 3x3x3 in the given mode. With `verbose`, reports the first
/// solution's length against the final one.
pub(crate) fn solve(
    cube: &Facelets,
    options: SolveOptions,
    mode: Mode,
    verbose: bool,
) -> SolveResult {
    let solver = Min2PhaseSolver::new();
    solver.prepare();
    let first = verbose.then(|| solver.solve(cube, &options));
    let budget = options
        .time_budget
        .unwrap_or(Duration::from_millis(DEFAULT_BUDGET_MS));
    let timed = options.with_time_budget(budget);
    let result = match mode {
        Mode::Probes(n) => solver.solve(cube, &options.with_min_probes(n)),
        Mode::Time => solver.solve_with_minimum_time(cube, &timed, budget),
        Mode::TimeParallel => solver.solve_parallel_with_minimum_time(cube, &timed, budget),
    };
    if let (Some(first), Some(last)) = (first, result.solution()) {
        if let Some(first) = first.solution() {
            eprintln!(
                "improved from {} to {} moves, probes {}",
                first.len(),
                last.len(),
                result.metrics.probes.unwrap_or(0)
            );
        }
    }
    result
}
