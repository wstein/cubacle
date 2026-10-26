//! A pruning table over a space too large to enumerate, kept as a hash.
//!
//! Ported from cs0x7f/cube555 src/Util.java (PruningTable.initPrunTablePartial),
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md. The states within `depth` moves of the goal are found by
//! breadth-first search from the goal; entry `key & mask` of the table holds
//! the smallest depth of the keys that fall on it, and `depth + 1` where none
//! does. A lookup is therefore a lower bound, never an overestimate.

use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};

use super::nibbles::Nibbles;
use super::par;

/// A cheap hasher for already well-mixed integer keys.
#[derive(Default)]
struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only u64 keys")
    }
    fn write_u64(&mut self, key: u64) {
        self.0 = key.wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(29);
    }
}

type Seen = HashSet<u64, BuildHasherDefault<Mix>>;

/// A space of keys with moves, to be searched to a limited depth.
pub(crate) trait Hashed: Sync {
    fn solved(&self) -> Vec<u64>;

    /// Calls `f` with every key one move away from `key`, and with the keys
    /// that a symmetry makes equivalent to those.
    fn neighbors<F: FnMut(u64)>(&self, key: u64, f: &mut F);
}

fn put_min(words: &[AtomicU32], index: usize, value: u32) {
    let shift = (index & 7) << 2;
    let cell = &words[index >> 3];
    let mut old = cell.load(Ordering::Relaxed);
    loop {
        if (old >> shift) & 0xf <= value {
            return;
        }
        let new = (old & !(0xf << shift)) | (value << shift);
        match cell.compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return,
            Err(now) => old = now,
        }
    }
}

/// Builds the table of `1 << bits` entries from the goal out to `depth`
/// moves (at most 14).
pub(crate) fn build<S: Hashed>(space: &S, depth: u32, bits: u32, threads: usize) -> Nibbles {
    let entries = 1usize << bits;
    let mask = entries as u64 - 1;
    let words: Vec<AtomicU32> = (0..entries / 8)
        .map(|_| AtomicU32::new(0x1111_1111 * (depth + 1)))
        .collect();
    let slot = |key: u64| (key & mask) as usize;
    let mut seen = Seen::default();
    let mut frontier = space.solved();
    frontier.sort_unstable();
    frontier.dedup();
    for level in 0..depth {
        for &key in &frontier {
            seen.insert(key);
            put_min(&words, slot(key), level);
        }
        let last = level + 1 == depth;
        let produced = std::sync::Mutex::new(Vec::<Vec<u64>>::new());
        let chunk = 4096;
        let seen_ref = &seen;
        par::for_chunks(threads, frontier.len(), chunk, |range| {
            let mut fresh = Vec::new();
            for &key in &frontier[range] {
                space.neighbors(key, &mut |n| {
                    if seen_ref.contains(&n) {
                        return;
                    }
                    if last {
                        put_min(&words, slot(n), level + 1);
                    } else {
                        fresh.push(n);
                    }
                });
            }
            if !fresh.is_empty() {
                produced.lock().expect("lock").push(fresh);
            }
        });
        frontier = produced.into_inner().expect("lock").concat();
        frontier.sort_unstable();
        frontier.dedup();
        if frontier.is_empty() {
            break;
        }
    }
    Nibbles::from_words(words.into_iter().map(AtomicU32::into_inner).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys 0..1000 on a ring, hashed into 64 entries.
    struct Ring;

    impl Hashed for Ring {
        fn solved(&self) -> Vec<u64> {
            vec![0]
        }
        fn neighbors<F: FnMut(u64)>(&self, key: u64, f: &mut F) {
            f((key + 1) % 1000);
            f((key + 999) % 1000);
        }
    }

    #[test]
    fn entries_are_the_smallest_depth_of_the_keys_that_share_them() {
        for threads in [1, 3] {
            let table = build(&Ring, 5, 6, threads);
            // key 0 and key 64 share an entry; 64 is far away, 0 is the goal
            assert_eq!(table.get(0), 0);
            assert_eq!(table.get(1), 1);
            assert_eq!(table.get(3), 3);
            assert_eq!(table.get(5), 5);
            // key 6 is beyond depth 5, but key 6 + 64 * 15 = 966 is 34 away
            // and key 6 is not near 0 or 1000 - 6: the entry stays at depth + 1
            assert_eq!(table.get(30), 6);
            // 1000 - 3 = 997 shares entry 997 & 63 = 37 with nothing close;
            // 997 is 3 from the goal, entry 37 gets 3
            assert_eq!(table.get(37), 3);
        }
    }
}
