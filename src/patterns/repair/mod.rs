//! Pattern repair: a cube that is a library pattern plus a few more moves.
//!
//! [`repair`] looks for the shortest block sequence `X` such that the state is
//! the pattern followed by `X` (order `After`) or `X` followed by the pattern
//! (order `Before`), in any of the 24 orientations of the pattern and with the
//! colors renamed freely. The blocks are single layers and wide blocks (see
//! `alphabet.rs`).
//!
//! The search meets in the middle. One table holds every sequence of up to
//! `da` blocks applied to a fixed start (the state itself for `After`, the
//! solved cube for `Before`), keyed by a color-blind hash. Each candidate
//! pattern then walks sequences of `db` blocks from its own start, and a
//! matching hash gives `X = (walked) (stored)^-1`. Levels `db = 0, 1, ...`
//! run over all candidates in turn, so the first level with a hit is the
//! shortest. Every hit is replayed with [`Facelets::apply`] before it is
//! returned.
//!
//! Candidates are ordered by how few stickers differ from the pattern (best
//! orientation and renaming). A fix of `L` blocks displaces at most `L` times
//! the stickers of the biggest block, which prunes candidates only for short
//! fixes; the order and the time limit do the rest, and a result that ran out
//! of time says so.

mod rotation;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use super::Pattern;
use crate::meet::{
    self, hash_of, perm_of, permuted, position_keys, sums_of, Alphabet, Kit, Table, Walker,
    ENTRY_BYTES,
};
use crate::puzzle::{Facelets, MoveSequence};
use rotation::{compose, inverse, mismatches, renamings, rotations, same_up_to_colors};

/// Memory for the stored side unless the caller says otherwise, in megabytes.
pub const DEFAULT_TABLE_MEMORY_MB: usize = 1000;
/// The longest fix the packed sequences can hold.
pub const MAX_BLOCKS: usize = 7;

/// What a repair may spend.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The most blocks in the fix (at most [`MAX_BLOCKS`]).
    pub max_blocks: usize,
    /// Cap for the stored side, in megabytes.
    pub max_memory_mb: usize,
    /// Wall-clock limit for the whole search.
    pub max_time: Duration,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_blocks: 6,
            max_memory_mb: DEFAULT_TABLE_MEMORY_MB,
            max_time: Duration::from_secs(60),
        }
    }
}

/// Whether the extra moves come after or before the pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairOrder {
    /// The pattern, then the extra moves.
    After,
    /// The extra moves, then the pattern.
    Before,
}

/// A verified fix: the state is the pattern and `moves` in `order`.
#[derive(Debug, Clone)]
pub struct Repair {
    pub id: String,
    pub name: String,
    pub catalogue: String,
    pub order: RepairOrder,
    /// The extra moves; for `Before` they are performed first, in the cube's
    /// own orientation.
    pub moves: MoveSequence,
    /// Whole-cube rotations (`x y2`) that put the pattern in the cube's
    /// orientation; empty if none.
    pub frame: String,
    /// The number of blocks searched, a wide block counting once.
    pub blocks: usize,
}

/// What the search did.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub candidates: usize,
    pub pruned: usize,
    pub tried: usize,
    /// States in the stored table and its size in megabytes.
    pub table_states: usize,
    pub table_mb: usize,
    /// Blocks stored / walked per candidate.
    pub stored_depth: usize,
    pub max_blocks: usize,
    pub timed_out: bool,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub found: Option<Repair>,
    pub stats: Stats,
}

struct Candidate<'p> {
    pattern: &'p Pattern,
    state: Vec<u8>,
    /// The generator as a sticker permutation, if the pattern has one.
    generator: Option<Vec<u16>>,
    /// Fewest stickers that differ from the cube over orientations.
    rank: usize,
}

struct Hit {
    candidate: usize,
    order: RepairOrder,
    rotation: usize,
    blocks: Vec<u8>,
}

fn colors(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| f.index() as u8).collect()
}

fn facelets(state: &[u8]) -> Facelets {
    Facelets::from_stickers(
        state
            .iter()
            .map(|&c| crate::puzzle::Face::ALL[c as usize])
            .collect(),
    )
    .expect("a recolored cube keeps its color counts")
}

/// The shortest fix of `cube` over `library`, if one is found within `limits`.
pub fn repair(cube: &Facelets, library: &[&Pattern], limits: &Limits) -> Outcome {
    let started = Instant::now();
    let deadline = started + limits.max_time;
    let mut search = Search::new(cube, library, limits, deadline);
    let mut stats = Stats {
        candidates: search.candidates.len(),
        max_blocks: limits.max_blocks.min(MAX_BLOCKS),
        ..Stats::default()
    };
    let mut best: Option<Hit> = None;
    let mut limit = stats.max_blocks;
    for order in [RepairOrder::After, RepairOrder::Before] {
        if limit == 0 {
            break;
        }
        if let Some(hit) = search.run(order, limit, &mut stats) {
            limit = hit.blocks.len() - 1;
            best = Some(hit);
        }
        if search.timed_out {
            break;
        }
    }
    stats.timed_out = search.timed_out;
    stats.tried = search.tried.iter().filter(|t| **t).count();
    stats.elapsed = started.elapsed();
    let found = best.map(|hit| search.report(hit));
    Outcome { found, stats }
}

struct Search<'p> {
    cube: Vec<u8>,
    facelets: Facelets,
    al: Alphabet,
    keys: Vec<u64>,
    rotations: Vec<rotation::Rotation>,
    candidates: Vec<Candidate<'p>>,
    tried: Vec<bool>,
    limits: Limits,
    deadline: Instant,
    timed_out: bool,
}

impl<'p> Search<'p> {
    fn new(
        cube: &Facelets,
        library: &[&'p Pattern],
        limits: &Limits,
        deadline: Instant,
    ) -> Search<'p> {
        let kind = cube.kind();
        let al = Alphabet::new(kind, Kit::Blocks);
        let rotations = rotations(kind);
        let renames = renamings();
        let state = colors(cube);
        let mut candidates: Vec<Candidate<'p>> = library
            .iter()
            .filter_map(|&pattern| {
                let p = pattern.state(kind)?;
                if p.is_solved() {
                    return None;
                }
                let p = colors(&p);
                let rank = rotations
                    .iter()
                    .map(|r| mismatches(&state, &permuted(&p, &r.perm), &renames))
                    .min()?;
                let generator = pattern
                    .generator(kind.size(), None)
                    .and_then(|g| g.sequence().ok())
                    .map(|s| perm_of(kind, s.moves()));
                Some(Candidate {
                    pattern,
                    state: p,
                    generator,
                    rank,
                })
            })
            .collect();
        candidates.sort_by_key(|c| c.rank);
        Search {
            cube: state,
            facelets: cube.clone(),
            al,
            keys: position_keys(kind.facelet_count()),
            rotations,
            tried: vec![false; candidates.len()],
            candidates,
            limits: *limits,
            deadline,
            timed_out: false,
        }
    }

    /// The deepest stored side that fits the memory cap.
    fn stored_depth(&self, blocks: usize) -> usize {
        meet::stored_depth(&self.al, self.limits.max_memory_mb, blocks)
    }

    /// Whether a candidate can still fit a fix of `blocks` blocks.
    fn possible(&self, candidate: &Candidate<'_>, blocks: usize) -> bool {
        candidate.rank <= blocks.saturating_mul(self.al.max_moved)
    }

    fn run(&mut self, order: RepairOrder, blocks: usize, stats: &mut Stats) -> Option<Hit> {
        let depth = self.stored_depth(blocks);
        let kind = self.al.kind;
        let start = match order {
            RepairOrder::After => self.cube.clone(),
            RepairOrder::Before => colors(&Facelets::solved(kind)),
        };
        let expected = self.al.count_up_to(depth) as usize;
        let mut walker = Walker::new(&self.al, &self.keys, self.deadline);
        let Some(table) = Table::build(&mut walker, &start, depth, expected) else {
            self.timed_out = true;
            return None;
        };
        stats.stored_depth = stats.stored_depth.max(depth);
        stats.table_states = stats.table_states.max(table.len());
        stats.table_mb = stats.table_mb.max((table.len() * ENTRY_BYTES) >> 20);
        stats.pruned = (0..self.candidates.len())
            .filter(|&i| !self.possible(&self.candidates[i], blocks))
            .count();
        for walked in 0..=(blocks - depth) {
            let mut best: Option<Hit> = None;
            for index in 0..self.candidates.len() {
                if !self.possible(&self.candidates[index], blocks) {
                    continue;
                }
                self.tried[index] = true;
                let bound = best.as_ref().map_or(blocks + 1, |h| h.blocks.len());
                let hit = self.stream(index, order, walked, &table, bound);
                if self.timed_out {
                    return best;
                }
                if hit.is_some() {
                    best = hit;
                    if walked > 0 {
                        break;
                    }
                }
            }
            if best.is_some() {
                return best;
            }
        }
        None
    }

    /// Walks `walked` blocks from every orientation of one candidate.
    fn stream(
        &mut self,
        index: usize,
        order: RepairOrder,
        walked: usize,
        table: &Table,
        bound: usize,
    ) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        let mut bound = bound;
        let mut seen: Vec<u64> = Vec::new();
        for r in 0..self.rotations.len() {
            let perm = &self.rotations[r].perm;
            let candidate = &self.candidates[index];
            let start = match order {
                RepairOrder::After => {
                    let start = permuted(&candidate.state, perm);
                    let h = hash_of(&sums_of(&start, &self.keys), false);
                    if seen.contains(&h) {
                        continue;
                    }
                    seen.push(h);
                    start
                }
                RepairOrder::Before => {
                    let Some(g) = &candidate.generator else {
                        return best;
                    };
                    let g_r = compose(&compose(&inverse(perm), g), perm);
                    permuted(&self.cube, &inverse(&g_r))
                }
            };
            let mut walker = Walker::new(&self.al, &self.keys, self.deadline);
            let Some(found) = meet::hits(
                &mut walker,
                table,
                &start,
                walked,
                bound,
                order == RepairOrder::After,
            ) else {
                self.timed_out = true;
                return best;
            };
            for blocks in found {
                if blocks.len() < bound && !blocks.is_empty() {
                    let hit = Hit {
                        candidate: index,
                        order,
                        rotation: r,
                        blocks,
                    };
                    if self.verify(&hit) {
                        bound = hit.blocks.len();
                        best = Some(hit);
                    }
                }
            }
        }
        best
    }

    /// Replays the fix with the plain move applier.
    fn verify(&self, hit: &Hit) -> bool {
        let kind = self.al.kind;
        let candidate = &self.candidates[hit.candidate];
        let moves = self.al.sequence(&hit.blocks);
        let rotation = &self.rotations[hit.rotation].perm;
        let result = match hit.order {
            RepairOrder::After => facelets(&permuted(&candidate.state, rotation)).apply(&moves),
            RepairOrder::Before => {
                let Some(g) = &candidate.generator else {
                    return false;
                };
                let g_r = compose(&compose(&inverse(rotation), g), rotation);
                Facelets::solved(kind)
                    .apply(&moves)
                    .map(|c| facelets(&permuted(&colors(&c), &g_r)))
            }
        };
        result.is_ok_and(|c| same_up_to_colors(&colors(&c), &self.cube))
    }

    fn report(&self, hit: Hit) -> Repair {
        let candidate = &self.candidates[hit.candidate];
        let pattern = candidate.pattern;
        debug_assert!(self.verify(&hit), "{}", self.facelets);
        Repair {
            id: pattern.id.clone(),
            name: pattern.name.clone(),
            catalogue: pattern.catalogue.clone(),
            order: hit.order,
            moves: self.al.sequence(&hit.blocks),
            frame: self.rotations[hit.rotation].name.clone(),
            blocks: hit.blocks.len(),
        }
    }
}
