//! The pattern shortcut: known patterns before the reduction.
//!
//! Cubes that a person built from classic patterns (checkerboard, six spot,
//! cube in a cube, ...) are far from a random state in the number of turns that
//! undo them, but a reduction engine does not see that: it solves them as it
//! solves any cube. [`PatternShortcut`] wraps such an engine. It lets the
//! engine solve as usual, then searches for the cube as a short product of
//! pattern macros (see `library`), and answers with whichever solution has
//! fewer moves. A cube that is no such product gets the engine's answer
//! unchanged.
//!
//! Files: `library` (the macros), `search` (meet in the middle over their
//! products), and for the generator search of `examples/pattern_search`:
//! `geometry` (sticker tables), `steps` (the move sets) and `bidir` (the
//! bidirectional search over them).

pub mod bidir;
pub mod geometry;
mod library;
mod search;
pub mod steps;

use std::sync::OnceLock;
use std::time::Instant;

use crate::puzzle::{
    Facelets, Failure, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, SolveStage, Solver,
};
use library::Library;
use search::Table;

/// What the shortcut searches with, built once per solver.
struct Search {
    library: Library,
    table: Table,
}

/// A solver for the 4x4x4 and bigger that tries known patterns as well.
pub struct PatternShortcut {
    inner: Box<dyn Solver>,
    search: OnceLock<Search>,
}

impl std::fmt::Debug for PatternShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternShortcut")
            .field("kind", &self.inner.kind())
            .finish_non_exhaustive()
    }
}

impl PatternShortcut {
    /// Wraps `inner`, which answers when no pattern is shorter. Cubes smaller
    /// than the 4x4x4 have no use for it; the wrapper then only passes on.
    pub fn new(inner: Box<dyn Solver>) -> PatternShortcut {
        PatternShortcut {
            inner,
            search: OnceLock::new(),
        }
    }

    fn search(&self) -> &Search {
        self.search.get_or_init(|| {
            let library = Library::build(self.inner.kind());
            let table = Table::build(&library);
            Search { library, table }
        })
    }

    fn applies(&self) -> bool {
        self.inner.kind().size() >= 4
    }

    /// `answer` with the pattern solution in place of its own when that has
    /// fewer moves (or when it has none).
    fn shorten(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        started: Instant,
        mut answer: SolveResult,
    ) -> SolveResult {
        if !self.applies()
            || options.reduce_only
            || answer.outcome == Err(Failure::InvalidInput)
            || facelets.is_solved()
        {
            return answer;
        }
        let deadline = options.time_budget.map(|budget| started + budget);
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return answer;
        }
        let search = self.search();
        let Some(found) = search.table.solve(&search.library, facelets, deadline) else {
            return answer;
        };
        let longest = options.max_moves.map_or(usize::MAX, usize::from);
        let own = answer.solution().map_or(usize::MAX, |s| s.len());
        if found.len() >= own || found.len() > longest {
            return answer;
        }
        answer.message = Some(format!(
            "known patterns: {} moves instead of {}",
            found.len(),
            if own == usize::MAX {
                "none".to_string()
            } else {
                own.to_string()
            }
        ));
        answer.stages = vec![SolveStage {
            name: "known patterns",
            end: found.len(),
        }];
        answer.outcome = Ok(found);
        answer.metrics = SolveMetrics {
            elapsed: started.elapsed(),
            ..answer.metrics
        };
        answer
    }
}

impl Solver for PatternShortcut {
    fn kind(&self) -> PuzzleKind {
        self.inner.kind()
    }

    fn prepare(&self) {
        self.inner.prepare();
        if self.applies() {
            self.search();
        }
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let answer = self.inner.solve(facelets, options);
        self.shorten(facelets, options, started, answer)
    }

    fn solve_improving(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let answer = self.inner.solve_improving(facelets, options);
        self.shorten(facelets, options, started, answer)
    }
}

#[cfg(test)]
mod tests;
