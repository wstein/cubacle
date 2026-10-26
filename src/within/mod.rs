//! `solve --within K`: the shortest route of at most K moves between two
//! states, if there is one.
//!
//! The search meets in the middle on color-wise hashes. A table holds every
//! sequence of up to `d` moves applied to the *target* (the same moves, so the
//! route runs through their inverses), the walk applies sequences of `w`
//! moves to the *start*, and a matching hash gives the route
//! `walked (stored)^-1`. Levels `w = 0, 1, ...` run in turn, so the first
//! level with a verified hit is the shortest route: a hit at level `w` has
//! `w` plus the stored distance moves, the stored distance is at most `d`, and
//! a shorter route would have been found at a lower level.
//!
//! Nothing is relative: the colors of the start are compared with the colors
//! of the target position by position, so every state is a valid target (also
//! one with mixed center colors) and no equal pieces have to be paired. Every
//! route is replayed with [`Facelets::apply`] before it is returned.
//!
//! Before any search, [`lower_bound`] answers "more than K moves" from the
//! number of differing stickers per orbit; see `meet/alphabet.rs` for the proof.

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crate::meet::{self, position_keys, Alphabet, Table, Walker, ENTRY_BYTES};
use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

pub use crate::meet::Kit;

/// The most moves a route may have (the packed sequences hold seven).
pub const MAX_MOVES: usize = 7;
/// Memory for the stored side unless the caller says otherwise, in megabytes.
pub const DEFAULT_MEMORY_MB: usize = 1000;
/// Time limit unless the caller says otherwise.
pub const DEFAULT_TIME: Duration = Duration::from_secs(60);

/// What a search may spend.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The most moves in the route (at most [`MAX_MOVES`]).
    pub max_moves: usize,
    /// Cap for the stored side, in megabytes.
    pub max_memory_mb: usize,
    /// Wall-clock limit for the whole search.
    pub max_time: Duration,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_moves: 6,
            max_memory_mb: DEFAULT_MEMORY_MB,
            max_time: DEFAULT_TIME,
        }
    }
}

/// Which limit ended a search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    Time,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A shortest route in the alphabet, replayed and checked.
    Found(MoveSequence),
    /// The lower bound exceeds the limit: no search was made.
    Excluded,
    /// The search finished and there is no route of at most K moves.
    NotFound,
    /// The search was stopped; there may or may not be a route.
    Limit(Limit),
}

/// What the search did.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// The lower bound on the moves; `None` when the target cannot be reached
    /// in this alphabet at all.
    pub bound: Option<usize>,
    pub stored_depth: usize,
    pub table_states: usize,
    pub table_mb: usize,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub outcome: Outcome,
    pub stats: Stats,
}

/// How an alphabet is called and what its length is counted in, for messages.
///
/// The outer blocks all touch the outside, so their count is HTM (OBTM from
/// the 4x4x4, the same rule). A single inner layer is one move here but two in
/// HTM, so with slices the count is BTM, one per block. The layers 2-3 are
/// written as two plain moves (`3Rw R'`) and count once, which no metric of
/// the plain moves does, so that alphabet counts blocks.
pub fn describe(kit: Kit, kind: PuzzleKind) -> (&'static str, &'static str) {
    let big = kind.size() >= 4;
    match kit {
        Kit::Outer if big => ("outer-block", "obtm"),
        Kit::Outer => ("face-turn", "htm"),
        Kit::Slices if big => ("outer-block and slice", "btm"),
        Kit::Slices => ("face-turn and slice", "btm"),
        Kit::Blocks => ("all-block", "blocks, layers 2-3 counting once"),
    }
}

fn colors(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| f.index() as u8).collect()
}

/// The fewest moves any route from `start` to `target` has in the alphabet,
/// as a bound: `None` when no route exists whatever its length.
pub fn lower_bound(start: &Facelets, target: &Facelets, kit: Kit) -> Option<usize> {
    Alphabet::new(start.kind(), kit).lower_bound(&colors(start), &colors(target))
}

/// The shortest route of at most `limits.max_moves` moves from `start` to
/// `target`, both of one size, in the alphabet `kit`.
pub fn search(start: &Facelets, target: &Facelets, kit: Kit, limits: &Limits) -> Report {
    run(start, target, kit, limits, true)
}

fn run(start: &Facelets, target: &Facelets, kit: Kit, limits: &Limits, use_bound: bool) -> Report {
    let started = Instant::now();
    let mut stats = Stats::default();
    let outcome = find(start, target, kit, limits, use_bound, started, &mut stats);
    stats.elapsed = started.elapsed();
    Report { outcome, stats }
}

fn find(
    start: &Facelets,
    target: &Facelets,
    kit: Kit,
    limits: &Limits,
    use_bound: bool,
    started: Instant,
    stats: &mut Stats,
) -> Outcome {
    let k = limits.max_moves.min(MAX_MOVES);
    if start.kind() != target.kind() {
        return Outcome::NotFound;
    }
    let al = Alphabet::new(start.kind(), kit);
    let (a, b) = (colors(start), colors(target));
    stats.bound = al.lower_bound(&a, &b);
    if use_bound && stats.bound.is_none_or(|bound| bound > k) {
        return Outcome::Excluded;
    }
    if a == b {
        return Outcome::Found(MoveSequence::default());
    }
    if k == 0 {
        return Outcome::NotFound;
    }
    let keys = position_keys(a.len());
    let depth = meet::stored_depth(&al, limits.max_memory_mb, k.div_ceil(2));
    if depth == 0 {
        return Outcome::Limit(Limit::Memory);
    }
    let deadline = started + limits.max_time;
    let mut walker = Walker::new(&al, &keys, deadline).exact();
    let expected = al.count_up_to(depth) as usize;
    let Some(table) = Table::build(&mut walker, &b, depth, expected) else {
        return Outcome::Limit(Limit::Time);
    };
    stats.stored_depth = depth;
    stats.table_states = table.len();
    stats.table_mb = (table.len() * ENTRY_BYTES) >> 20;
    for walked in 0..=(k - depth) {
        let Some(hits) = meet::hits(&mut walker, &table, &a, walked, k + 1, true) else {
            return Outcome::Limit(Limit::Time);
        };
        let best = hits
            .iter()
            .filter(|blocks| verified(&al, start, target, blocks))
            .min_by_key(|blocks| blocks.len());
        if let Some(blocks) = best {
            return Outcome::Found(al.sequence(blocks));
        }
    }
    Outcome::NotFound
}

/// Replays a route with the plain move applier.
fn verified(al: &Alphabet, start: &Facelets, target: &Facelets, blocks: &[u8]) -> bool {
    start
        .apply(&al.sequence(blocks))
        .is_ok_and(|end| end == *target)
}
