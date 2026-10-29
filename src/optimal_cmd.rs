//! `solve --optimal`: the shortest 3x3x3 solution, or the default answer when
//! the budget ends first.

use std::sync::Arc;
use std::time::{Duration, Instant};

use cubacle::engines::optimal::{Limits, OptimalSolver, Tables};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{
    Facelets, PuzzleKind, SolveEvidence, SolveFrame, SolveMetrics, SolveOptions, SolveResult,
    Solver,
};

use crate::cli::SolveArgs;
use crate::ram::{self, Ram};

/// Probes of the default engine's answer, as in a plain `solve`.
const DEFAULT_PROBES: u64 = 3000;

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

/// The tables chosen, and the sentence that says why.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Choice {
    pub tables: Tables,
    pub why: String,
}

fn mb(bytes: u64) -> u64 {
    bytes / 1_000_000
}

/// The largest tables that fit: within `--max-memory` when given, else within
/// a share of the memory of this machine, else the small ones.
pub(crate) fn choose(max_memory: Option<usize>, ram: Option<Ram>) -> Result<Choice, String> {
    let (budget, why) = match (max_memory, ram) {
        (Some(budget), _) => (budget, format!("--max-memory {budget} MB")),
        (None, Some(ram)) => {
            let budget = ram::default_budget_mb(&ram);
            let available = ram
                .available
                .map_or_else(|| "not reported".to_owned(), |a| format!("{} MB", mb(a)));
            (
                budget,
                format!(
                    "this machine has {} MB of memory, available: {available}; budget {budget} MB (half of the available memory, at most 16000)",
                    mb(ram.total)
                ),
            )
        }
        (None, None) => {
            return Ok(Choice {
                tables: Tables::Small,
                why: "the memory of this machine could not be read; using the small tables"
                    .to_owned(),
            })
        }
    };
    let tables = Tables::fitting(budget).ok_or_else(|| {
        format!(
            "--optimal needs at least {} MB of tables ({why})",
            Tables::Small.bytes().div_ceil(1_000_000)
        )
    })?;
    Ok(Choice { tables, why })
}

/// The optimal engine with the tables `choose` picks, wrapped so that it says
/// on stderr whether its answer is proven.
pub(crate) fn solver(
    max_memory: Option<usize>,
    nodes: Option<u64>,
) -> Result<Box<dyn Solver>, String> {
    let Choice { tables, why } = choose(max_memory, ram::detect())?;
    let engine = OptimalSolver::new(Arc::new(Min2PhaseSolver::new()), tables)
        .with_notes(|line| eprintln!("{line}"));
    eprintln!(
        "optimal search with the {} pruning tables ({} MB): {why}",
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
        // The budget is for the search. The default engine answers first, with
        // the counted improvement of a plain `solve` (about 90 ms): its shorter
        // answer is the fallback and ends the search sooner.
        let first = SolveOptions {
            time_budget: None,
            min_probes: Some(DEFAULT_PROBES),
            ..*options
        };
        let (outcome, message, evidence) = match self.engine.solve_report(facelets, &first, &limits)
        {
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
                let evidence = SolveEvidence {
                    frame: SolveFrame::Fixed,
                    proven: report.proven,
                    lower_bound: Some(u32::from(report.lower_bound)),
                };
                (Ok(report.solution), None, evidence)
            }
            Err((failure, message)) => (Err(failure), message, SolveEvidence::default()),
        };
        SolveResult {
            outcome,
            message,
            metrics: SolveMetrics {
                elapsed: start.elapsed(),
                probes: None,
                evidence,
            },
            stages: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1_000_000_000;

    fn ram(total: u64, available: Option<u64>) -> Option<Ram> {
        Some(Ram { total, available })
    }

    #[test]
    fn a_big_host_gets_the_large_tables() {
        let choice = choose(None, ram(96 * GB, Some(80 * GB))).unwrap();
        assert_eq!(choice.tables, Tables::Large);
        assert!(choice.why.contains("96000 MB"), "{}", choice.why);
    }

    #[test]
    fn the_budget_is_half_of_the_available_memory() {
        // 1.2 GB available: 600 MB fits the large set (555 MB).
        assert_eq!(
            choose(None, ram(16 * GB, Some(1_200_000_000)))
                .unwrap()
                .tables,
            Tables::Large
        );
        // 1.0 GB available: 500 MB fits only the small set.
        assert_eq!(
            choose(None, ram(16 * GB, Some(GB))).unwrap().tables,
            Tables::Small
        );
        // Without an available figure the total is used.
        assert_eq!(
            choose(None, ram(2 * GB, None)).unwrap().tables,
            Tables::Large
        );
        // Too little memory is an error that names the budget.
        let error = choose(None, ram(100_000_000, None)).unwrap_err();
        assert!(error.contains("needs at least"), "{error}");
    }

    #[test]
    fn an_explicit_cap_wins_over_the_memory_seen() {
        assert_eq!(
            choose(Some(90), ram(96 * GB, Some(96 * GB)))
                .unwrap()
                .tables,
            Tables::Small
        );
        assert_eq!(
            choose(Some(600), ram(GB, Some(GB))).unwrap().tables,
            Tables::Large
        );
        assert!(choose(Some(10), None).is_err());
    }

    #[test]
    fn unknown_memory_falls_back_to_the_small_tables_and_says_so() {
        let choice = choose(None, None).unwrap();
        assert_eq!(choice.tables, Tables::Small);
        assert!(choice.why.contains("could not be read"));
    }
}
