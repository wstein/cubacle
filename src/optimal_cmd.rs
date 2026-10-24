//! `solve --optimal`: the shortest 3x3x3 solution, or the default answer when
//! the budget ends first.

use std::sync::Arc;
use std::time::{Duration, Instant};

use cubacle::engines::optimal::{Limits, OptimalSolver, Tables};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver};

use crate::SolveArgs;

/// The search time when `-b` is absent.
const DEFAULT_BUDGET_MS: u64 = 60_000;

/// Rejects `--optimal` for other sizes.
pub(crate) fn check(args: &SolveArgs, kind: PuzzleKind) -> Result<(), String> {
    if args.optimal && kind != PuzzleKind::Cube3 {
        return Err(format!(
            "--optimal works on the 3x3x3 only, not the {0}x{0}x{0}",
            kind.size()
        ));
    }
    Ok(())
}

/// The optimal engine with the largest tables `max_memory_mb` allows, wrapped
/// so that it says on stderr whether its answer is proven.
pub(crate) fn solver(max_memory_mb: usize, nodes: Option<u64>) -> Result<Box<dyn Solver>, String> {
    let tables = Tables::fitting(max_memory_mb).ok_or_else(|| {
        format!(
            "--optimal needs at least {} MB of tables (--max-memory is {max_memory_mb})",
            Tables::Small.bytes().div_ceil(1_000_000)
        )
    })?;
    let engine = OptimalSolver::new(Arc::new(Min2PhaseSolver::new()), tables)
        .with_notes(|line| eprintln!("{line}"));
    eprintln!(
        "optimal search with the {} pruning tables ({} MB)",
        if tables == Tables::Large {
            "large"
        } else {
            "small"
        },
        tables.bytes() / 1_000_000
    );
    engine.prepare();
    Ok(Box::new(Optimal { engine, nodes }))
}

struct Optimal {
    engine: OptimalSolver,
    nodes: Option<u64>,
}

impl Solver for Optimal {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let start = Instant::now();
        let budget = options
            .time_budget
            .unwrap_or(Duration::from_millis(DEFAULT_BUDGET_MS));
        let limits = Limits {
            time: Some(budget),
            nodes: self.nodes,
            cancel: None,
        };
        // The budget is for the search; the default engine answers first.
        let first = SolveOptions {
            time_budget: None,
            ..*options
        };
        let (outcome, message) = match self.engine.solve_report(facelets, &first, &limits) {
            Ok(report) => {
                let n = report.solution.len();
                if report.proven {
                    eprintln!("optimal: no solution is shorter than {n} moves");
                } else {
                    let reason = if self.nodes.is_some_and(|max| report.nodes >= max) {
                        format!("{} nodes", report.nodes)
                    } else {
                        format!("{} ms", budget.as_millis())
                    };
                    eprintln!(
                        "not proven shortest: the default engine's {n} moves; no solution is shorter than {} (stopped after {reason})",
                        report.lower_bound
                    );
                }
                (Ok(report.solution), None)
            }
            Err((failure, message)) => (Err(failure), message),
        };
        SolveResult {
            outcome,
            message,
            metrics: SolveMetrics {
                elapsed: start.elapsed(),
                probes: None,
            },
            stages: Vec::new(),
        }
    }
}
