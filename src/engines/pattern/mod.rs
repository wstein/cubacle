//! The pattern engine: macros, the searches over them, and the shortcut that
//! tries known patterns before a reduction.
//!
//! Cubes that a person built from classic patterns (checkerboard, six spot,
//! cube in a cube, ...) are far from a random state in the number of turns that
//! undo them, but a reduction engine does not see that. This module finds such
//! cubes as short products of pattern macros (see `library`), for two users:
//!
//! * [`PatternShortcut`] wraps a reduction engine. It lets the engine solve as
//!   usual, then searches for the cube as a product of macros and answers with
//!   whichever solution has fewer moves. A cube that is no such product gets
//!   the engine's answer unchanged.
//! * [`find_generator`] (the `patterns find` command) looks for the shortest
//!   generator of a state, with an exact search for shorter ones.
//!
//! Every search takes a [`Cap`] on its memory. A search that reaches the cap
//! stops and says so; it never claims that nothing exists.
//!
//! Files: `library` (the macros, built from `crate::patterns::bases`),
//! `search` (meet in the middle over their products), `find` (the generator
//! search), and for the generator search of `examples/pattern_search`:
//! `geometry` (sticker tables), `steps` (the move sets) and `bidir` (the
//! bidirectional search over them).

pub mod bidir;
mod find;
pub mod geometry;
mod library;
mod search;
pub mod steps;

use std::sync::OnceLock;
use std::time::Instant;

use crate::puzzle::{
    Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions, SolveResult,
    SolveStage, Solver,
};
pub use find::{find_generator, Generator, NotFound};
use library::Library;
use search::Table;

/// The memory cap in megabytes when none is given.
pub const DEFAULT_MAX_MEMORY_MB: usize = 1500;

/// A hard limit on the memory a search may use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cap {
    pub max_bytes: usize,
}

impl Cap {
    pub fn megabytes(mb: usize) -> Cap {
        Cap {
            max_bytes: mb.saturating_mul(1 << 20),
        }
    }
}

impl Default for Cap {
    fn default() -> Cap {
        Cap::megabytes(DEFAULT_MAX_MEMORY_MB)
    }
}

/// What a probe of the macro search found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    /// Moves that solve the cube.
    Found(MoveSequence),
    /// No product of macros solves it. `complete` is false when the cap or
    /// the deadline cut the search short, so that a product may exist.
    Missing { complete: bool },
}

/// The macro search for one cube size: the macros and their table.
pub struct PatternSearch {
    library: Library,
    table: Table,
    merge: bool,
}

impl PatternSearch {
    /// Builds the macros of `kind` and the table of their products within
    /// `cap`. With `allow`, only macros whose moves all pass it are used (and
    /// the answer keeps exactly those moves, no merging into ranges); an
    /// error says when no macro is left.
    pub fn new(
        kind: PuzzleKind,
        cap: Cap,
        allow: Option<&dyn Fn(&Move) -> bool>,
    ) -> Result<PatternSearch, String> {
        let library = Library::build(kind, allow);
        if library.macros.is_empty() {
            return Err(
                "no pattern macro is made only of the allowed moves; the set cannot be searched as pattern macros"
                    .to_owned(),
            );
        }
        let table = Table::build(&library, cap);
        Ok(PatternSearch {
            library,
            table,
            merge: allow.is_none(),
        })
    }

    pub fn kind(&self) -> PuzzleKind {
        self.library.kind
    }

    pub fn macro_count(&self) -> usize {
        self.library.macros.len()
    }

    /// Whether the cap stopped the table before it was complete.
    pub fn capped(&self) -> bool {
        self.table.capped()
    }

    /// A sentence on what cut the search short, for messages.
    pub(crate) fn limit_text(&self, cap: Cap) -> String {
        if self.capped() {
            format!("memory cap of {} MB reached", cap.max_bytes >> 20)
        } else {
            "time budget ran out".to_owned()
        }
    }

    /// The shortest solution that is a product of macros, if one is within
    /// reach before `deadline`.
    pub fn solve(&self, state: &Facelets, deadline: Option<Instant>) -> Probe {
        match self.table.solve(&self.library, state, deadline, self.merge) {
            Some(found) => Probe::Found(found),
            None => Probe::Missing {
                complete: !self.table.capped() && deadline.is_none_or(|d| Instant::now() < d),
            },
        }
    }
}

/// A solver for the 4x4x4 and bigger that tries known patterns as well.
pub struct PatternShortcut {
    inner: Box<dyn Solver>,
    search: OnceLock<Option<PatternSearch>>,
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

    fn search(&self) -> Option<&PatternSearch> {
        self.search
            .get_or_init(|| PatternSearch::new(self.inner.kind(), Cap::default(), None).ok())
            .as_ref()
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
        let Some(search) = self.search() else {
            return answer;
        };
        let Probe::Found(found) = search.solve(facelets, deadline) else {
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
