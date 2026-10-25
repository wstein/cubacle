//! The walk over block sequences, the color-blind hash and the stored table.
//!
//! The hash of a state is the sum over colors of a mixed sum of the positions
//! of that color. It does not depend on how the colors are named and changes
//! in O(stickers moved) per block, so the last block of a walk is never applied
//! to the state.

use std::time::Instant;

use super::alphabet::Alphabet;

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

pub(crate) type Sums = [u64; 6];

/// Per-position random values.
pub(crate) fn position_keys(count: usize) -> Vec<u64> {
    (0..count as u64).map(|i| mix(i ^ 0xC0BA_C1E0)).collect()
}

pub(crate) fn sums_of(state: &[u8], keys: &[u64]) -> Sums {
    let mut sums = [0u64; 6];
    for (i, &c) in state.iter().enumerate() {
        sums[c as usize] = sums[c as usize].wrapping_add(keys[i]);
    }
    sums
}

/// The hash of per-colour sums. Colour-blind, it ignores which colour has
/// which sum; exact, a state and the same state with colours renamed differ.
pub(crate) fn hash_of(sums: &Sums, exact: bool) -> u64 {
    sums.iter().enumerate().fold(0u64, |h, (c, &s)| {
        let salt = if exact {
            0xA5A5_5A5A ^ (c as u64 + 1) << 40
        } else {
            0xA5A5_5A5A
        };
        h.wrapping_add(mix(s ^ salt))
    })
}

/// The state after the sticker permutation `perm`.
pub(crate) fn permuted(state: &[u8], perm: &[u16]) -> Vec<u8> {
    let mut out = vec![0u8; state.len()];
    for (i, &c) in state.iter().enumerate() {
        out[perm[i] as usize] = c;
    }
    out
}

/// Walks block sequences, with a deadline.
pub(crate) struct Walker<'a> {
    al: &'a Alphabet,
    keys: &'a [u64],
    deadline: Instant,
    exact: bool,
    ticks: u32,
    pub(crate) aborted: bool,
}

type Visit<'v> = &'v mut dyn FnMut(u64, &[u8]);

impl<'a> Walker<'a> {
    /// Hashes by exact colours instead of ignoring their names.
    pub(crate) fn exact(mut self) -> Walker<'a> {
        self.exact = true;
        self
    }

    /// The alphabet the walk runs over.
    pub(crate) fn alphabet(&self) -> &'a Alphabet {
        self.al
    }

    pub(crate) fn new(al: &'a Alphabet, keys: &'a [u64], deadline: Instant) -> Walker<'a> {
        Walker {
            al,
            keys,
            deadline,
            exact: false,
            ticks: 0,
            aborted: false,
        }
    }

    /// Visits the hash and blocks of every sequence of exactly `depth` blocks
    /// (of 0 to `depth` blocks when `all`) started from `start`.
    pub(crate) fn walk(&mut self, start: &[u8], depth: usize, all: bool, visit: Visit<'_>) {
        let mut levels: Vec<Vec<u8>> = vec![start.to_vec(); depth.max(1)];
        let sums = sums_of(start, self.keys);
        if all || depth == 0 {
            visit(hash_of(&sums, self.exact), &[]);
        }
        if depth > 0 {
            self.step(0, sums, &mut Vec::new(), depth, all, &mut levels, visit);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        &mut self,
        level: usize,
        sums: Sums,
        seq: &mut Vec<u8>,
        depth: usize,
        all: bool,
        levels: &mut [Vec<u8>],
        visit: Visit<'_>,
    ) {
        let last = seq.last().copied();
        for k in 0..self.al.len() {
            if self.aborted {
                return;
            }
            if !self.al.may_follow(last, k) {
                continue;
            }
            self.ticks += 1;
            if self.ticks & 0x3FFF == 0 && Instant::now() >= self.deadline {
                self.aborted = true;
                return;
            }
            let mut next = sums;
            for &(from, to) in &self.al.moved[k] {
                let c = levels[level][from as usize] as usize;
                next[c] = next[c]
                    .wrapping_sub(self.keys[from as usize])
                    .wrapping_add(self.keys[to as usize]);
            }
            seq.push(k as u8);
            if level + 1 == depth {
                visit(hash_of(&next, self.exact), seq);
            } else {
                if all {
                    visit(hash_of(&next, self.exact), seq);
                }
                {
                    let (head, tail) = levels.split_at_mut(level + 1);
                    tail[0].copy_from_slice(&head[level]);
                    for &(from, to) in &self.al.moved[k] {
                        tail[0][to as usize] = head[level][from as usize];
                    }
                }
                self.step(level + 1, next, seq, depth, all, levels, visit);
            }
            seq.pop();
        }
    }
}

/// Hash and block sequence of every state within a depth, sorted by hash;
/// one entry per hash, the shortest sequence.
pub(crate) struct Table {
    entries: Vec<(u64, u64)>,
}

pub(crate) const ENTRY_BYTES: usize = std::mem::size_of::<(u64, u64)>();

fn pack(seq: &[u8]) -> u64 {
    seq.iter().enumerate().fold(seq.len() as u64, |c, (i, &k)| {
        c | (u64::from(k) << (4 + 8 * i))
    })
}

fn unpack(code: u64) -> Vec<u8> {
    (0..(code & 15) as usize)
        .map(|i| (code >> (4 + 8 * i)) as u8)
        .collect()
}

impl Table {
    pub(crate) fn build(
        walker: &mut Walker<'_>,
        start: &[u8],
        depth: usize,
        expected: usize,
    ) -> Option<Table> {
        let mut entries: Vec<(u64, u64)> = Vec::with_capacity(expected);
        walker.walk(start, depth, true, &mut |h, seq| {
            entries.push((h, pack(seq)))
        });
        if walker.aborted {
            return None;
        }
        entries.sort_unstable_by_key(|e| (e.0, e.1 & 15));
        entries.dedup_by_key(|e| e.0);
        entries.shrink_to_fit();
        Some(Table { entries })
    }

    pub(crate) fn get(&self, hash: u64) -> Option<Vec<u8>> {
        let at = self.entries.partition_point(|e| e.0 < hash);
        self.entries
            .get(at)
            .filter(|e| e.0 == hash)
            .map(|e| unpack(e.1))
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}
