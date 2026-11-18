//! A list of distinct solutions of one cube: [`Solver::solve_many`].
//!
//! New code. Phase 5 of the reduction keeps the first `n` results in the serial
//! order (the first is the reduction of a plain solve), each gets its own 3x3x3
//! tail under the options of a plain solve, and solutions that are equal after
//! normalizing are dropped, asking phase 5 for more until there are `n` or no
//! more.

use std::time::Instant;

use super::finish::{Finished, Miss};
use super::reduction::{reduce_listing, Stuck};
use super::solver::{result_of, FiveSolver};
use crate::puzzle::{Facelets, Failure, SolveOptions, SolveResult};

impl FiveSolver {
    /// Up to `n` distinct solutions, the plain solution first.
    pub(super) fn list(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        n: usize,
    ) -> Vec<SolveResult> {
        let started = Instant::now();
        let (cube, tables, deadline) = match self.ready(facelets, options, started) {
            Ok(ready) => ready,
            Err(failed) => return vec![failed],
        };
        let mut finished: Vec<Finished> = Vec::new();
        let mut tried = 0;
        let mut first_miss: Option<Miss> = None;
        let listed = reduce_listing(
            tables,
            &cube,
            self.threads(),
            deadline,
            self.keep(),
            n,
            |reductions| {
                while tried < reductions.len() && finished.len() < n {
                    let reduction = &reductions[tried];
                    tried += 1;
                    match self.finish(facelets, reduction, options, deadline) {
                        Ok(done) => {
                            if finished.iter().all(|g| g.solution != done.solution) {
                                finished.push(done);
                            }
                        }
                        Err(miss) if finished.is_empty() => {
                            first_miss = Some(miss);
                            return true;
                        }
                        // the clock is spent: keep what there is
                        Err(miss) if miss.failure == Failure::BudgetExhausted => return true,
                        // a tail over the move limit or the like: the next one
                        Err(_) => {}
                    }
                }
                finished.len() >= n
            },
        );
        let failed = |failure, message: &str| {
            vec![result_of(
                started,
                Err(failure),
                Some(message.to_owned()),
                None,
            )]
        };
        match listed {
            Err(Stuck::TimedOut) if finished.is_empty() => {
                return failed(
                    Failure::BudgetExhausted,
                    "the reduction ran out of time before the 3x3x3 stage",
                )
            }
            Err(Stuck::NoWay) if finished.is_empty() => {
                return failed(Failure::Internal, "the reduction found no solution")
            }
            _ => {}
        }
        if finished.is_empty() {
            return match first_miss {
                Some(miss) => vec![result_of(
                    started,
                    Err(miss.failure),
                    Some(miss.message),
                    None,
                )],
                None => failed(Failure::Internal, "no reduction to finish"),
            };
        }
        finished
            .into_iter()
            .map(|done| SolveResult {
                stages: done.stages,
                ..result_of(started, Ok(done.solution), None, done.probes)
            })
            .collect()
    }
}
