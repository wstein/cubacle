//! Breadth-first construction of pruning tables, in parallel.
//!
//! Ported from cs0x7f/cube555 src/Util.java (PruningTable.initPrunTable),
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md. The layers are the same: while less than half of the states are
//! known, the states of the last layer are expanded; after that, every unknown
//! state looks for a neighbor in the last layer. Distances are exact, so the
//! result does not depend on the order of work or on the thread count.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use super::nibbles::Nibbles;
use super::par;

/// The deepest layer a 4-bit entry can hold; `0xf` marks an unknown state.
const MAX_DEPTH: u32 = 15;
/// Words handed to a worker at a time.
const CHUNK_WORDS: usize = 2048;

/// A graph of states with moves, the search space of one pruning table.
pub(crate) trait Space: Sync {
    /// The number of states.
    fn size(&self) -> usize;

    /// The states at distance 0.
    fn solved(&self) -> Vec<usize>;

    /// Calls `f` with the state each move leads to from `state`, and a tag
    /// that is handed back to [`Space::images`] for that target.
    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F);

    /// Calls `f` with the states that share the distance of `target` (reached
    /// with `tag`) because of a symmetry that fixes its class. None for
    /// plain coordinates.
    fn images<F: FnMut(usize)>(&self, _target: usize, _tag: u32, _f: &mut F) {}
}

/// Sets nibble `index` to `value` if it is unknown; true if this call did it.
#[inline(always)]
fn claim(words: &[AtomicU32], index: usize, value: u32) -> bool {
    let shift = (index & 7) << 2;
    let old = words[index >> 3].fetch_and(!(0xf << shift) | (value << shift), Ordering::Relaxed);
    (old >> shift) & 0xf == 0xf
}

#[inline(always)]
fn read(words: &[AtomicU32], index: usize) -> u32 {
    (words[index >> 3].load(Ordering::Relaxed) >> ((index & 7) << 2)) & 0xf
}

/// Builds the table of exact distances for `space` with `threads` workers.
/// Layers go down to `cap` (at most 15, the original's limit; states deeper
/// than that stay at `0xf`, unknown).
pub(crate) fn build<S: Space>(space: &S, threads: usize, cap: u32) -> Nibbles {
    let n = space.size();
    let words: Vec<AtomicU32> = (0..Nibbles::words_for(n))
        .map(|_| AtomicU32::new(u32::MAX))
        .collect();
    let mut cumulative = 0usize;
    for state in space.solved() {
        if claim(&words, state, 0) {
            cumulative += 1;
        }
    }
    let mut depth = 0u32;
    loop {
        let inverse = cumulative > n / 2;
        let (select, check, fill) = if inverse {
            (0xf, depth, depth + 1)
        } else {
            (depth, 0xf, depth + 1)
        };
        depth += 1;
        let done = AtomicUsize::new(0);
        par::for_chunks(threads, words.len(), CHUNK_WORDS, |range| {
            let mut found = 0;
            for w in range {
                let word = words[w].load(Ordering::Relaxed);
                if !inverse && word == u32::MAX {
                    continue;
                }
                for j in 0..8 {
                    let state = w * 8 + j;
                    if state >= n {
                        break;
                    }
                    if (word >> (j * 4)) & 0xf != select {
                        continue;
                    }
                    if inverse {
                        let mut hit = false;
                        space.moves(state, &mut |t, _| {
                            if !hit && read(&words, t) == check {
                                hit = true;
                            }
                        });
                        if hit && claim(&words, state, fill) {
                            found += 1;
                        }
                    } else {
                        space.moves(state, &mut |t, tag| {
                            if read(&words, t) == check && claim(&words, t, fill) {
                                found += 1;
                                space.images(t, tag, &mut |e| {
                                    if read(&words, e) == check && claim(&words, e, fill) {
                                        found += 1;
                                    }
                                });
                            }
                        });
                    }
                }
            }
            done.fetch_add(found, Ordering::Relaxed);
        });
        let done = done.into_inner();
        cumulative += done;
        if done == 0 || depth >= cap.min(MAX_DEPTH) {
            break;
        }
    }
    Nibbles::from_words(words.into_iter().map(AtomicU32::into_inner).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring of `n` states with moves +1 and -1.
    struct Ring(usize);

    impl Space for Ring {
        fn size(&self) -> usize {
            self.0
        }
        fn solved(&self) -> Vec<usize> {
            vec![0]
        }
        fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
            f((state + 1) % self.0, 0);
            f((state + self.0 - 1) % self.0, 0);
        }
    }

    #[test]
    fn distances_on_a_ring_are_exact_for_any_thread_count() {
        for threads in [1, 4] {
            let table = build(&Ring(25), threads, 15);
            for state in 0..25usize {
                assert_eq!(table.get(state) as usize, state.min(25 - state));
            }
        }
    }

    #[test]
    fn a_cap_stops_the_layers_early() {
        let table = build(&Ring(3001), 2, 4);
        assert_eq!(table.get(3), 3);
        assert_eq!(table.get(4), 4);
        assert_eq!(table.get(5), 0xf);
    }

    #[test]
    fn inverse_layers_agree_with_forward_layers() {
        // 13 of 25 states are known after layer 6, so the last layers run inverted.
        let table = build(&Ring(25), 2, 15);
        assert_eq!(table.get(12), 12);
        let big = build(&Ring(3001), 3, 15);
        assert_eq!(big.get(14), 14);
        assert_eq!(big.get(15), 0xf);
        assert_eq!(big.get(3001 - 14), 14);
    }
}
