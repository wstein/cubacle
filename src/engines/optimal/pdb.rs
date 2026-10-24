//! Pattern databases: for a few pieces of the cube, the fewest moves that put
//! those pieces right (the others ignored), kept at 4 bits per state.
//!
//! Such a count never exceeds the true distance, so the largest over several
//! databases is a lower bound. A database is built by a breadth first search
//! from the solved cube, level by level, and cached on disk by `Cache`.

#![allow(clippy::needless_range_loop)]

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use super::model::{tables, Cube, MOVES};
use crate::reduction::cache::Cache;

const UNSET: u32 = 0xF;
const EMPTY_WORD: u32 = 0xFFFF_FFFF;

/// Which pieces a database looks at.
#[derive(Clone, Debug)]
pub(crate) struct Spec {
    /// File name of the cache entry.
    pub id: String,
    /// All eight corners, or the listed edges.
    pub corners: bool,
    pub edges: Vec<u8>,
}

impl Spec {
    pub(crate) fn corners() -> Spec {
        Spec {
            id: "corners8".into(),
            corners: true,
            edges: Vec::new(),
        }
    }

    pub(crate) fn edges(first: u8, count: u8) -> Spec {
        Spec {
            id: format!("edges{count}from{first}"),
            corners: false,
            edges: (first..first + count).collect(),
        }
    }

    /// The number of states.
    pub(crate) fn states(&self) -> usize {
        if self.corners {
            40_320 * 2_187
        } else {
            let k = self.edges.len();
            (0..k).map(|i| 12 - i).product::<usize>() << k
        }
    }

    fn words(&self) -> usize {
        self.states().div_ceil(8)
    }

    /// The index of the state of the cube.
    #[inline]
    pub(crate) fn index(&self, cube: &Cube) -> usize {
        self.index_of(&cube.edges, &cube.corners)
    }

    /// The index of the state of the pieces `edges` and `corners`.
    #[inline]
    pub(crate) fn index_of(&self, edges: &[u8; 16], corners: &[u8; 16]) -> usize {
        if self.corners {
            let mut rank = 0usize;
            let mut mask = 0xFFu32;
            let mut twist = 0usize;
            for i in 0..8 {
                let v = usize::from(corners[i]);
                let (place, t) = (v / 3, v % 3);
                rank = rank * (8 - i) + (mask & ((1 << place) - 1)).count_ones() as usize;
                mask &= !(1 << place);
                if i < 7 {
                    twist = twist * 3 + t;
                }
            }
            rank * 2_187 + twist
        } else {
            let mut rank = 0usize;
            let mut mask = 0xFFFu32;
            let mut flips = 0usize;
            for (i, &piece) in self.edges.iter().enumerate() {
                let v = usize::from(edges[usize::from(piece)]);
                let place = v >> 1;
                rank = rank * (12 - i) + (mask & ((1 << place) - 1)).count_ones() as usize;
                mask &= !(1 << place);
                flips = flips << 1 | (v & 1);
            }
            rank << self.edges.len() | flips
        }
    }

    /// The piece bytes of state `index` (inverse of `index`).
    fn unrank(&self, index: usize) -> [u8; 12] {
        let mut out = [0u8; 12];
        if self.corners {
            let (mut rank, mut twist) = (index / 2_187, index % 2_187);
            let mut t = [0usize; 8];
            let mut sum = 0;
            for i in (0..7).rev() {
                t[i] = twist % 3;
                twist /= 3;
                sum += t[i];
            }
            t[7] = (3 - sum % 3) % 3;
            let mut digits = [0usize; 8];
            for i in (0..8).rev() {
                digits[i] = rank % (8 - i);
                rank /= 8 - i;
            }
            let mut mask = 0xFFu32;
            for i in 0..8 {
                let place = nth_set(mask, digits[i]);
                mask &= !(1 << place);
                out[i] = (3 * place + t[i]) as u8;
            }
        } else {
            let k = self.edges.len();
            let (mut rank, flips) = (index >> k, index & ((1 << k) - 1));
            let mut digits = [0usize; 12];
            for i in (0..k).rev() {
                digits[i] = rank % (12 - i);
                rank /= 12 - i;
            }
            let mut mask = 0xFFFu32;
            for i in 0..k {
                let place = nth_set(mask, digits[i]);
                mask &= !(1 << place);
                out[i] = (2 * place + ((flips >> (k - 1 - i)) & 1)) as u8;
            }
        }
        out
    }

    /// The index of the state after move `m`, from the unranked bytes.
    fn moved(&self, bytes: &[u8; 12], m: usize) -> usize {
        let t = tables();
        let mut cube = Cube::SOLVED;
        if self.corners {
            for i in 0..8 {
                cube.corners[i] = t.corners[m][usize::from(bytes[i])];
            }
        } else {
            for (i, &piece) in self.edges.iter().enumerate() {
                cube.edges[usize::from(piece)] = t.edges[m][usize::from(bytes[i])];
            }
        }
        self.index(&cube)
    }
}

/// The `n`-th (from 0) lowest set bit of `mask`.
fn nth_set(mut mask: u32, n: usize) -> usize {
    for _ in 0..n {
        mask &= mask - 1;
    }
    mask.trailing_zeros() as usize
}

/// A finished database.
pub(crate) struct Table {
    pub spec: Spec,
    words: Vec<u32>,
}

impl Table {
    #[inline]
    pub(crate) fn get(&self, index: usize) -> u8 {
        ((self.words[index >> 3] >> ((index & 7) * 4)) & 0xF) as u8
    }

    #[inline]
    pub(crate) fn bound(&self, cube: &Cube) -> u8 {
        self.get(self.spec.index(cube))
    }

    /// The count for the inverse cube, which is as far from solved.
    #[inline]
    pub(crate) fn inverse_bound(&self, cube: &Cube) -> u8 {
        self.get(self.spec.index_of(&cube.inv_edges, &cube.inv_corners))
    }

    pub(crate) fn mib(&self) -> f64 {
        self.words.len() as f64 * 4.0 / 1_048_576.0
    }

    /// Loads the table from the cache, or builds it (and stores it).
    /// `note` is told what happens. Returns the table and whether it was built.
    pub(crate) fn load_or_build(
        spec: Spec,
        cache: &Cache,
        threads: usize,
        note: &dyn Fn(&str),
    ) -> (Table, bool) {
        let id = format!("optimal-{}", spec.id);
        if let Some(words) = cache.load(&id, spec.words()) {
            return (Table { spec, words }, false);
        }
        note(&format!(
            "building the pruning table {} ({:.0} MiB, {} states)",
            spec.id,
            spec.words() as f64 * 4.0 / 1_048_576.0,
            spec.states()
        ));
        let table = Table::build(spec, threads, note);
        if !cache.store(&id, &table.words) {
            note("warning: the table could not be saved; it is built again next time");
        }
        (table, true)
    }

    fn build(spec: Spec, threads: usize, note: &dyn Fn(&str)) -> Table {
        let words: Vec<AtomicU32> = (0..spec.words())
            .map(|_| AtomicU32::new(EMPTY_WORD))
            .collect();
        let set = |index: usize, level: u32| {
            let shift = (index & 7) * 4;
            // An unset nibble is 1111; and-ing the level in sets it.
            words[index >> 3].fetch_and(!(0xF << shift) | (level << shift), Ordering::Relaxed);
        };
        let get =
            |index: usize| (words[index >> 3].load(Ordering::Relaxed) >> ((index & 7) * 4)) & 0xF;
        set(spec.index(&Cube::SOLVED), 0);
        let mut level = 0u32;
        loop {
            let next = AtomicUsize::new(0);
            let found = AtomicUsize::new(0);
            const CHUNK: usize = 1 << 14;
            std::thread::scope(|scope| {
                for _ in 0..threads.max(1) {
                    scope.spawn(|| {
                        let mut count = 0;
                        loop {
                            let start = next.fetch_add(CHUNK, Ordering::Relaxed);
                            if start >= words.len() {
                                break;
                            }
                            for w in start..(start + CHUNK).min(words.len()) {
                                let word = words[w].load(Ordering::Relaxed);
                                if word == EMPTY_WORD {
                                    continue;
                                }
                                for n in 0..8 {
                                    if (word >> (4 * n)) & 0xF != level {
                                        continue;
                                    }
                                    let index = w * 8 + n;
                                    if index >= spec.states() {
                                        continue;
                                    }
                                    let bytes = spec.unrank(index);
                                    for m in 0..MOVES {
                                        let child = spec.moved(&bytes, m);
                                        if get(child) == UNSET {
                                            set(child, level + 1);
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        found.fetch_add(count, Ordering::Relaxed);
                    });
                }
            });
            // A child met by two threads at once is counted by both.
            let count = found.load(Ordering::Relaxed);
            if count == 0 {
                break;
            }
            level += 1;
            note(&format!("  depth {level}: about {count} states"));
        }
        let words: Vec<u32> = words.into_iter().map(AtomicU32::into_inner).collect();
        Table { spec, words }
    }
}

/// Several databases; the bound is the largest of their counts.
pub(crate) struct Heuristic {
    pub tables: Vec<Table>,
}

impl Heuristic {
    pub(crate) fn bound(&self, cube: &Cube) -> u8 {
        self.tables
            .iter()
            .map(|t| t.bound(cube).max(t.inverse_bound(cube)))
            .max()
            .unwrap_or(0)
    }

    /// Whether some database says more than `limit` moves are needed.
    #[inline]
    pub(crate) fn exceeds(&self, cube: &Cube, limit: u8) -> bool {
        self.tables
            .iter()
            .any(|t| t.bound(cube) > limit || t.inverse_bound(cube) > limit)
    }

    pub(crate) fn mib(&self) -> f64 {
        self.tables.iter().map(Table::mib).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_inverts() {
        for spec in [Spec::corners(), Spec::edges(0, 6), Spec::edges(5, 7)] {
            let n = spec.states();
            for index in (0..n).step_by(n / 997 + 1).chain([n - 1]) {
                let bytes = spec.unrank(index);
                let mut cube = Cube::SOLVED;
                if spec.corners {
                    cube.corners[..8].copy_from_slice(&bytes[..8]);
                } else {
                    for (i, &p) in spec.edges.iter().enumerate() {
                        cube.edges[usize::from(p)] = bytes[i];
                    }
                }
                assert_eq!(spec.index(&cube), index, "{}", spec.id);
            }
        }
    }
}
