//! Phase 3, wings: the pruning table, two bits per state.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Edge3.java (`createPrun`,
//! `getprun`, `getPruning`, `setPruning`), MIT, Copyright (c) 2023 Chen Shuang;
//! this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! An entry holds the distance from the solved state modulo 3, or 3 for
//! "farther than the table goes" (9 moves for the full table). Distances are
//! exact, so the table does not depend on the order of work or the number of
//! threads. The reference scans the whole table for each layer; this keeps the
//! states of the last layer in a list instead, which gives the same table.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use super::edge3::{Edge, Edge3, N_PRUN, N_RAW, SEARCH_MOVES};
use crate::reduction::par;

/// Layers of the full table; states beyond are known to be at least 10.
pub(super) const FULL_LAYERS: u32 = 9;
/// States per piece of work.
const CHUNK: usize = 256;

pub(super) struct WingPrun {
    words: Vec<u32>,
    /// The deepest layer in the table.
    layers: u32,
}

#[inline(always)]
fn read(words: &[AtomicU32], idx: usize) -> u32 {
    (words[idx >> 4].load(Ordering::Relaxed) >> ((idx & 15) << 1)) & 3
}

/// Sets entry `idx` to `value` if it is unknown; true if this call did it.
/// All claims in a layer carry the same value, so a race is harmless.
#[inline(always)]
fn claim(words: &[AtomicU32], idx: usize, value: u32) -> bool {
    if read(words, idx) != 3 {
        return false;
    }
    let shift = (idx & 15) << 1;
    let old = words[idx >> 4].fetch_and(!(3 << shift) | (value << shift), Ordering::Relaxed);
    (old >> shift) & 3 == 3
}

impl WingPrun {
    /// Builds the table with layers down to `layers` (at most 9 are useful
    /// to the search) on `threads` threads.
    pub(super) fn build(t: &Edge3, threads: usize, layers: u32) -> WingPrun {
        let words: Vec<AtomicU32> = (0..N_PRUN / 16).map(|_| AtomicU32::new(u32::MAX)).collect();
        claim(&words, 0, 0);
        let mut frontier = vec![0usize];
        for depth in 0..layers {
            let value = (depth + 1) % 3;
            let found = Mutex::new(Vec::new());
            par::for_chunks(threads, frontier.len(), CHUNK, |range| {
                let mut next = Vec::new();
                let mut e = Edge::new();
                for &state in &frontier[range] {
                    e.set((t.sym2raw[state / N_RAW] as usize * N_RAW + state % N_RAW) as i32);
                    for m in 0..SEARCH_MOVES {
                        let to = t.after_move(&e.edge, m);
                        if !claim(&words, to.state, value) {
                            continue;
                        }
                        next.push(to.state);
                        let mut sym_state = t.symstate[to.class];
                        if sym_state == 1 {
                            continue;
                        }
                        let mut f = e;
                        f.apply(m);
                        f.rotate(to.sym);
                        let mut j = 1;
                        loop {
                            sym_state >>= 1;
                            if sym_state == 0 {
                                break;
                            }
                            if sym_state & 1 == 1 {
                                let mut g = f;
                                g.rotate(j);
                                let image = to.class * N_RAW + g.get(10) as usize % N_RAW;
                                if claim(&words, image, value) {
                                    next.push(image);
                                }
                            }
                            j += 1;
                        }
                    }
                }
                found
                    .lock()
                    .expect("no panic while holding the lock")
                    .push(next);
            });
            frontier = found
                .into_inner()
                .expect("no panic while holding the lock")
                .into_iter()
                .flatten()
                .collect();
        }
        WingPrun {
            words: words.into_iter().map(AtomicU32::into_inner).collect(),
            layers,
        }
    }

    #[inline(always)]
    fn get(&self, idx: usize) -> u32 {
        (self.words[idx >> 4] >> ((idx & 15) << 1)) & 3
    }

    /// What a state at least `layers + 1` away is taken to be at.
    fn far(&self) -> i32 {
        self.layers as i32 + 1
    }

    /// The distance of state `idx` given a lower bound `prun` that is at most
    /// one too small or one too large (the distance of its parent).
    #[inline(always)]
    pub(super) fn relative(&self, idx: usize, prun: i32) -> i32 {
        let depm3 = self.get(idx) as i32;
        if depm3 == 3 {
            return self.far();
        }
        (depm3 - prun + 16) % 3 + prun - 1
    }

    /// The exact distance of state `idx` by walking down the table.
    pub(super) fn exact(&self, t: &Edge3, idx: usize) -> i32 {
        let mut depm3 = self.get(idx) as i32;
        if depm3 == 3 {
            return self.far();
        }
        let mut idx = idx;
        let mut depth = 0;
        let mut e = Edge::new();
        while idx != 0 {
            depm3 = if depm3 == 0 { 2 } else { depm3 - 1 };
            e.set((t.sym2raw[idx / N_RAW] as usize * N_RAW + idx % N_RAW) as i32);
            let next = (0..SEARCH_MOVES)
                .map(|m| t.after_move(&e.edge, m).state)
                .find(|&to| self.get(to) as i32 == depm3);
            match next {
                Some(to) => {
                    depth += 1;
                    idx = to;
                }
                None => return self.far(),
            }
        }
        depth
    }

    /// Bytes held.
    pub(super) fn bytes(&self) -> usize {
        self.words.len() * 4
    }

    /// How many states have an entry (not unknown).
    #[cfg(test)]
    fn known(&self) -> usize {
        (0..N_PRUN).filter(|&i| self.get(i) != 3).count()
    }

    #[cfg(test)]
    pub(super) fn words(&self) -> &[u32] {
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_sizes_match_the_reference_and_do_not_depend_on_threads() {
        let t = Edge3::new();
        // The reference prints the number of states within each depth.
        let cumulative = [1usize, 4, 16, 55, 324, 1922, 12275];
        let one = WingPrun::build(&t, 1, 6);
        assert_eq!(one.known(), cumulative[6]);
        let four = WingPrun::build(&t, 4, 6);
        assert_eq!(four.words(), one.words());
        for (layers, &want) in cumulative.iter().enumerate().take(5) {
            assert_eq!(WingPrun::build(&t, 2, layers as u32).known(), want);
        }
    }

    #[test]
    fn exact_distances_agree_with_the_residues() {
        let t = Edge3::new();
        let table = WingPrun::build(&t, 2, 5);
        assert_eq!(table.exact(&t, 0), 0);
        let mut seen = [0usize; 7];
        for idx in (0..N_PRUN).step_by(7) {
            let d = table.exact(&t, idx);
            seen[d as usize] += 1;
            if table.get(idx) != 3 {
                assert_eq!(d % 3, table.get(idx) as i32);
            } else {
                assert_eq!(d, 6);
            }
        }
        assert!(seen[..6].iter().all(|&n| n > 0));
    }
}

#[cfg(test)]
mod full_tests {
    use super::super::testing::fnv;
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore = "builds the full wing table (about a second)"]
    fn four_full_wing_table_is_the_reference_table() {
        let t = Edge3::new();
        for threads in [1, 4] {
            let started = Instant::now();
            let table = WingPrun::build(&t, threads, FULL_LAYERS);
            eprintln!("threads {threads}: {:?}", started.elapsed());
            // Java: int[] eprun, hashed as signed ints
            assert_eq!(
                fnv(table.words().iter().map(|&w| i64::from(w as i32))),
                7659434398870190603
            );
            assert_eq!(table.known(), 2_778_197);
        }
    }
}
