//! The 3x3x3 solved in the fewest moves (half-turn metric), proven.
//!
//! An iterative deepening A* search on the pieces of the cube, bounded below
//! by the largest of several pattern databases (all corners, and two sets of
//! edges). The upper bound, and the answer when the budget ends first, is the
//! solution of an injected solver (the default engine); a shorter solution is
//! only ever put in its place.
//!
//! * `model`: the pieces of a cube and the effect of the 18 moves on them.
//! * `pdb`: the databases, their ranking, build and disk cache.
//! * `search`: the depth-limited search with its move rules.
//! * `vector`: the move step, where a vector version can take over.

mod model;
mod pdb;
mod search;
mod vector;

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use crate::puzzle::{
    Facelets, Failure, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Solver,
};
use crate::reduction::cache::{default_dir, Cache};
use model::{Cube, FACE_NAMES, TURN_NAMES};
use pdb::{Heuristic, Spec, Table};
use search::{Ended, Search};

const CACHE_VERSION: u32 = 1;

/// How large the databases are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tables {
    /// All corners and two disjoint sets of six edges.
    Small,
    /// All corners and two overlapping sets of seven edges.
    Large,
    /// Small and weak, for tests only.
    #[cfg(test)]
    Tiny,
}

impl Tables {
    fn specs(self) -> Vec<Spec> {
        match self {
            Tables::Small => vec![Spec::corners(), Spec::edges(0, 6), Spec::edges(6, 6)],
            Tables::Large => vec![Spec::corners(), Spec::edges(0, 7), Spec::edges(5, 7)],
            #[cfg(test)]
            Tables::Tiny => vec![Spec::edges(0, 5), Spec::edges(5, 5)],
        }
    }

    /// The memory the databases take, in bytes.
    pub fn bytes(self) -> usize {
        self.specs().iter().map(|s| s.states().div_ceil(2)).sum()
    }

    /// The largest tables that fit in `megabytes` (1 MB = 10^6 bytes), if any.
    pub fn fitting(megabytes: usize) -> Option<Tables> {
        [Tables::Large, Tables::Small]
            .into_iter()
            .find(|t| t.bytes() <= megabytes.saturating_mul(1_000_000))
    }
}

/// What ends a search before it proves anything.
#[derive(Default)]
pub struct Limits<'a> {
    /// Wall-clock time for the search (not for building the tables).
    pub time: Option<Duration>,
    /// Nodes to visit; the same on every machine.
    pub nodes: Option<u64>,
    /// Asked every few thousand nodes; true ends the search.
    pub cancel: Option<&'a dyn Fn() -> bool>,
}

/// The answer of an optimal solve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub solution: MoveSequence,
    /// The solution is a shortest one.
    pub proven: bool,
    /// No solution is shorter than this (equal to the length when proven).
    pub lower_bound: u8,
    /// The length of the solution of the injected solver.
    pub upper_bound: u8,
    pub nodes: u64,
    /// The search found the solution, not the injected solver.
    pub improved: bool,
}

/// The optimal solver; `upper` supplies the first solution.
pub struct OptimalSolver {
    upper: Arc<dyn Solver + Send + Sync>,
    tables: Tables,
    threads: usize,
    cache: Cache,
    note: Box<dyn Fn(&str) + Send + Sync>,
    heuristic: OnceLock<Heuristic>,
}

impl OptimalSolver {
    pub fn new(upper: Arc<dyn Solver + Send + Sync>, tables: Tables) -> OptimalSolver {
        OptimalSolver {
            upper,
            tables,
            threads: std::thread::available_parallelism().map_or(1, |n| n.get().min(4)),
            cache: Cache::new(default_dir(), CACHE_VERSION),
            note: Box::new(|_| {}),
            heuristic: OnceLock::new(),
        }
    }

    /// Where progress of the table build is reported.
    pub fn with_notes(mut self, note: impl Fn(&str) + Send + Sync + 'static) -> OptimalSolver {
        self.note = Box::new(note);
        self
    }

    /// Threads used to build the tables (default: up to four).
    pub fn with_threads(mut self, threads: usize) -> OptimalSolver {
        self.threads = threads.max(1);
        self
    }

    /// Keeps the tables in this directory instead of the default cache.
    pub fn with_cache_dir(mut self, dir: Option<std::path::PathBuf>) -> OptimalSolver {
        self.cache = Cache::new(dir, CACHE_VERSION);
        self
    }

    /// Loads or builds the tables; returns the seconds spent building, 0 when
    /// they were all cached (or already in memory).
    pub fn prepare_timed(&self) -> f64 {
        let start = Instant::now();
        let mut built = false;
        self.heuristic.get_or_init(|| {
            let tables = self
                .tables
                .specs()
                .into_iter()
                .map(|spec| {
                    let (table, was_built) =
                        Table::load_or_build(spec, &self.cache, self.threads, &*self.note);
                    built |= was_built;
                    table
                })
                .collect();
            Heuristic { tables }
        });
        if built {
            start.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }

    /// The memory of the tables in MiB (builds them if needed).
    pub fn table_mib(&self) -> f64 {
        self.prepare_timed();
        self.heuristic.get().map_or(0.0, Heuristic::mib)
    }

    #[cfg(test)]
    fn heuristic_bound(&self, cube: &Cube) -> u8 {
        self.prepare_timed();
        self.heuristic.get().map_or(0, |h| h.bound(cube))
    }

    /// The tables' lower bound for a cube (builds the tables if needed).
    pub fn lower_bound(&self, facelets: &Facelets) -> Option<u8> {
        self.prepare_timed();
        let cube = Cube::from_facelets(facelets)?;
        self.heuristic.get().map(|h| h.bound(&cube))
    }

    /// Solves with the injected solver, then searches for a shorter solution
    /// until the limits end it or none is shorter.
    pub fn solve_report(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        limits: &Limits,
    ) -> Result<Report, (Failure, Option<String>)> {
        let first = self.upper.solve(facelets, options);
        let upper = match first.outcome {
            Ok(solution) => solution,
            Err(failure) => return Err((failure, first.message)),
        };
        let cube = Cube::from_facelets(facelets).ok_or((Failure::InvalidInput, None))?;
        let upper_bound = upper.len() as u8;
        self.prepare_timed();
        let heuristic = self.heuristic.get().ok_or((Failure::Internal, None))?;
        let start = Instant::now();
        let stop = || {
            limits.time.is_some_and(|t| start.elapsed() >= t)
                || limits.cancel.is_some_and(|cancel| cancel())
        };
        let mut search = Search::new(heuristic, limits.nodes.unwrap_or(u64::MAX), &stop);
        let first_limit = heuristic.bound(&cube);
        let mut limit = first_limit;
        let report = |solution, proven, lower_bound, nodes, improved| Report {
            solution,
            proven,
            lower_bound,
            upper_bound,
            nodes,
            improved,
        };
        while limit < upper_bound {
            match search.run(&cube, limit) {
                (Ended::Found, moves) => {
                    let text: Vec<String> = moves
                        .iter()
                        .map(|&m| {
                            format!(
                                "{}{}",
                                FACE_NAMES[usize::from(m) / 3],
                                TURN_NAMES[usize::from(m) % 3]
                            )
                        })
                        .collect();
                    let solution: MoveSequence = text
                        .join(" ")
                        .parse()
                        .map_err(|_| (Failure::Internal, None))?;
                    return Ok(report(solution, true, limit, search.nodes, true));
                }
                (Ended::Exhausted, _) => limit += 1,
                (Ended::Stopped, _) => {
                    return Ok(report(upper, false, limit, search.nodes, false));
                }
            }
        }
        Ok(report(upper, true, upper_bound, search.nodes, false))
    }
}

impl Solver for OptimalSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn prepare(&self) {
        self.prepare_timed();
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let start = Instant::now();
        let limits = Limits {
            time: options.time_budget,
            ..Limits::default()
        };
        let (outcome, message, evidence) = match self.solve_report(facelets, options, &limits) {
            Ok(report) => {
                let evidence = crate::puzzle::SolveEvidence {
                    frame: crate::puzzle::SolveFrame::Fixed,
                    proven: report.proven,
                    lower_bound: Some(u32::from(report.lower_bound)),
                };
                (Ok(report.solution), None, evidence)
            }
            Err((failure, message)) => (Err(failure), message, Default::default()),
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
mod tests;
