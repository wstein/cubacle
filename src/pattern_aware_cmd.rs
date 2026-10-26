//! `solve --pattern-aware`: the known route of a library pattern, adopted only
//! when it is strictly shorter than the solver's answer.
//!
//! The solver always runs first, so the answer is never longer than without the
//! flag. The start (or, with `--to`, the relative state and its inverse) is looked
//! up in an index of the library's generators under the 48 symmetries; see
//! `cubacle::patterns::RouteIndex`.

use std::time::{Duration, Instant};

use cubacle::patterns::{KnownRoute, RouteIndex};
use cubacle::puzzle::{
    reaches, relative_state, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveResult,
    SolveStage,
};

/// A route that was adopted.
pub struct Adopted {
    pub name: String,
    pub catalog: String,
    pub shorter: Option<usize>,
}

impl Adopted {
    /// The line printed on standard error.
    pub fn line(&self) -> String {
        match self.shorter {
            Some(n) => format!(
                "pattern: {} {} (library route, {n} {} shorter)",
                self.catalog,
                self.name,
                if n == 1 { "move" } else { "moves" }
            ),
            None => format!(
                "pattern: {} {} (library route, no other answer)",
                self.catalog, self.name
            ),
        }
    }
}

/// What `apply` needs to know.
pub struct Request<'a> {
    pub config: Option<&'a std::path::Path>,
    pub start: &'a Facelets,
    pub target: Option<&'a Facelets>,
    /// The cube the solver was given: the start, or the relative state.
    pub cube: &'a Facelets,
    pub max_moves: Option<u8>,
    pub slices: bool,
    pub verbose: bool,
}

/// Whether a move is one the solver of the size would write: face turns on
/// the 3x3x3; from the 4x4x4 up also outer blocks (`Rw`), and single inner
/// layers (`2R`) only with `--slices`.
fn in_alphabet(kind: PuzzleKind, slices: bool, mv: &Move) -> bool {
    if mv.depth == 1 && !mv.wide {
        return true;
    }
    if kind == PuzzleKind::Cube3 {
        return false;
    }
    mv.wide || slices
}

/// The index of the active library for one size, and the time it took.
pub fn index(
    config: Option<&std::path::Path>,
    kind: PuzzleKind,
    slices: bool,
) -> (RouteIndex, Duration) {
    let started = Instant::now();
    let library = match crate::patterns_config::load(config) {
        Ok(config) => {
            let library = config.library();
            for warning in &library.warnings {
                eprintln!("warning: {warning}");
            }
            library
        }
        Err(error) => {
            eprintln!("warning: {error}; using the built-in library");
            crate::patterns_config::Config::default().library()
        }
    };
    let index = RouteIndex::build(kind, library.all(), &|mv| in_alphabet(kind, slices, mv));
    (index, started.elapsed())
}

/// The shorter of the routes for `cube` (and, with a target, for the inverse
/// of the relative state), replayed from the start.
fn best_route(index: &RouteIndex, request: &Request) -> Option<KnownRoute> {
    let mut found: Vec<KnownRoute> = Vec::new();
    found.extend(index.lookup(request.cube));
    if let Some(target) = request.target {
        // R^-1 is the relative state with start and target swapped; the route
        // that builds R^-1 from the solved cube solves R.
        if let Ok(inverse) = relative_state(target, request.start) {
            found.extend(index.lookup(&inverse).map(|hit| KnownRoute {
                solve: hit.build.clone(),
                build: hit.solve.clone(),
                ..hit
            }));
        }
    }
    let wanted = request
        .target
        .cloned()
        .unwrap_or_else(|| Facelets::solved(request.start.kind()));
    found
        .into_iter()
        .filter(|hit| reaches(request.start, &hit.solve, &wanted))
        .min_by_key(|hit| hit.solve.len())
}

/// `result`, or the library route in its place when that has fewer moves.
pub fn apply(request: &Request, result: SolveResult) -> (SolveResult, Option<Adopted>) {
    let kind = request.cube.kind();
    if kind == PuzzleKind::Cube2 || result.outcome == Err(Failure::InvalidInput) {
        return (result, None);
    }
    let (index, built) = index(request.config, kind, request.slices);
    if request.verbose {
        eprintln!(
            "pattern index: {} generators of the library in {built:?}",
            index.len()
        );
    }
    let Some(hit) = best_route(&index, request) else {
        return (result, None);
    };
    let own = result.solution().map(MoveSequence::len);
    let longest = request.max_moves.map_or(usize::MAX, usize::from);
    let route = hit.solve.len();
    if own.is_some_and(|own| route >= own) || route > longest {
        return (result, None);
    }
    let adopted = Adopted {
        name: hit.name,
        catalog: hit.catalog,
        shorter: own.map(|own| own - route),
    };
    let result = SolveResult {
        outcome: Ok(hit.solve),
        message: None,
        stages: vec![SolveStage {
            name: "known patterns",
            end: route,
        }],
        ..result
    };
    (result, Some(adopted))
}
