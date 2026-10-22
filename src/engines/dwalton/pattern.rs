//! Pruning tables over colour patterns of sticker sets.
//!
//! New code. A table has groups of stickers; the pattern of a group says
//! which of its stickers show a colour of a given set (`marks`), exactly `k`
//! of them. The state is the combination rank of each group, in mixed radix.
//! Distances are exact, by breadth-first search from the goal patterns over
//! the stage's moves, and are cached on disk like the 5x5x5 tables.

use crate::reduction::bfs::{self, Space};
use crate::reduction::cache::Cache;
use crate::reduction::nibbles::Nibbles;

const fn binomials() -> [[u32; 33]; 33] {
    let mut c = [[0u32; 33]; 33];
    let mut n = 0;
    while n < 33 {
        c[n][0] = 1;
        let mut k = 1;
        while k <= n {
            c[n][k] = c[n - 1][k - 1] + if k < n { c[n - 1][k] } else { 0 };
            k += 1;
        }
        n += 1;
    }
    c
}
const C: [[u32; 33]; 33] = binomials();

/// Rank of a set of positions among all sets of its size.
#[inline(always)]
pub(super) fn rank(mut mask: u32) -> u32 {
    let (mut r, mut i) = (0, 1);
    while mask != 0 {
        r += C[mask.trailing_zeros() as usize][i];
        i += 1;
        mask &= mask - 1;
    }
    r
}

pub(super) fn unrank(mut r: u32, k: usize) -> u32 {
    let mut mask = 0;
    for i in (1..=k).rev() {
        let mut p = i - 1;
        while C[p + 1][i] <= r {
            p += 1;
        }
        r -= C[p][i];
        mask |= 1 << p;
    }
    mask
}

pub(super) struct Group {
    /// Slot of the group in the stage's node.
    pub slot: usize,
    /// Sticker indices of the group, in the order of the mask bits.
    pub cells: Vec<usize>,
    pub k: usize,
}

impl Group {
    fn count(&self) -> usize {
        C[self.cells.len()][self.k] as usize
    }
}

pub(super) struct Table {
    pub groups: Vec<Group>,
    radix: Vec<usize>,
    size: usize,
    nibbles: Nibbles,
    /// For tables over few reachable states: the depth of each, by key.
    reach: Option<(Vec<usize>, Reach)>,
}

type Reach = std::collections::HashMap<u128, u8, std::hash::BuildHasherDefault<KeyHash>>;

#[derive(Default)]
pub(super) struct KeyHash(u64);

impl std::hash::Hasher for KeyHash {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only u128 keys")
    }
    fn write_u128(&mut self, key: u128) {
        let x = (key as u64) ^ ((key >> 64) as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        self.0 = x.wrapping_mul(0xff51_afd7_ed55_8ccd).rotate_left(31);
    }
}

struct Build<'a> {
    groups: &'a [Group],
    radix: &'a [usize],
    size: usize,
    /// `[move][group][local cell]` -> local cell.
    local: Vec<Vec<Vec<u8>>>,
    solved: Vec<usize>,
}

impl Space for Build<'_> {
    fn size(&self) -> usize {
        self.size
    }
    fn solved(&self) -> Vec<usize> {
        self.solved.clone()
    }
    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        let mut masks = [0u32; 4];
        for (g, group) in self.groups.iter().enumerate() {
            masks[g] = unrank(((state / self.radix[g]) % group.count()) as u32, group.k);
        }
        for perms in &self.local {
            let mut next = 0;
            for (g, perm) in perms.iter().enumerate() {
                let (mut m, mut out) = (masks[g], 0u32);
                while m != 0 {
                    out |= 1 << perm[m.trailing_zeros() as usize];
                    m &= m - 1;
                }
                next += rank(out) as usize * self.radix[g];
            }
            f(next, 0);
        }
    }
}

impl Table {
    /// Builds (or loads) the table. `goals[j][g]` is the marked-cell mask of
    /// group `g` in goal `j`; `dest[m][t]` the destination of tracked sticker
    /// `t` under move `m`.
    pub(super) fn new(
        name: &str,
        groups: Vec<Group>,
        goals: &[Vec<u32>],
        dest: &[&[usize]],
        cache: &Cache,
        threads: usize,
    ) -> Table {
        assert!(groups.len() <= 4);
        let mut radix = vec![0; groups.len()];
        let mut size = 1;
        for g in (0..groups.len()).rev() {
            radix[g] = size;
            size *= groups[g].count();
        }
        if let Some(words) = cache.load(name, Nibbles::words_for(size)) {
            return Table {
                groups,
                radix,
                size,
                nibbles: Nibbles::from_words(words),
                reach: None,
            };
        }
        let local = dest
            .iter()
            .map(|d| {
                groups
                    .iter()
                    .map(|group| {
                        group
                            .cells
                            .iter()
                            .map(|&t| {
                                let to = d[t];
                                group
                                    .cells
                                    .iter()
                                    .position(|&c| c == to)
                                    .expect("a move keeps a group in itself")
                                    as u8
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        let solved = goals
            .iter()
            .map(|goal| {
                goal.iter()
                    .enumerate()
                    .map(|(g, &m)| rank(m) as usize * radix[g])
                    .sum()
            })
            .collect();
        let space = Build {
            groups: &groups,
            radix: &radix,
            size,
            local,
            solved,
        };
        let nibbles = bfs::build(&space, threads, 15);
        cache.store(name, nibbles.words());
        Table {
            groups,
            radix,
            size,
            nibbles,
            reach: None,
        }
    }

    /// A table of the states reachable from the goals, found by breadth-first
    /// search and kept by key. `goals[j][g]` is the mask of group `g`.
    pub(super) fn reachable(groups: Vec<Group>, goals: &[Vec<u32>], dest: &[&[usize]]) -> Table {
        let widths: Vec<usize> = {
            let mut at = 0;
            groups
                .iter()
                .map(|g| {
                    let w = at;
                    at += g.cells.len();
                    w
                })
                .collect()
        };
        assert!(
            widths.last().copied().unwrap_or(0) + groups.last().map_or(0, |g| g.cells.len()) <= 128
        );
        let local: Vec<Vec<Vec<u8>>> = dest
            .iter()
            .map(|d| {
                groups
                    .iter()
                    .map(|group| {
                        group
                            .cells
                            .iter()
                            .map(|&t| {
                                group
                                    .cells
                                    .iter()
                                    .position(|&c| c == d[t])
                                    .expect("a move keeps a group in itself")
                                    as u8
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        let pack = |masks: &[u32]| -> u128 {
            masks
                .iter()
                .zip(&widths)
                .map(|(&m, &w)| u128::from(m) << w)
                .sum()
        };
        let mut seen = Reach::default();
        let mut frontier: Vec<Vec<u32>> = Vec::new();
        for goal in goals {
            if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(pack(goal)) {
                e.insert(0);
                frontier.push(goal.clone());
            }
        }
        let mut depth = 0u8;
        while !frontier.is_empty() && depth < 14 {
            depth += 1;
            let mut next = Vec::new();
            for masks in &frontier {
                for perms in &local {
                    let moved: Vec<u32> = masks
                        .iter()
                        .zip(perms)
                        .map(|(&m, perm)| {
                            let (mut m, mut out) = (m, 0u32);
                            while m != 0 {
                                out |= 1 << perm[m.trailing_zeros() as usize];
                                m &= m - 1;
                            }
                            out
                        })
                        .collect();
                    if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(pack(&moved)) {
                        e.insert(depth);
                        next.push(moved);
                    }
                }
            }
            frontier = next;
        }
        let size = seen.len();
        Table {
            groups,
            radix: widths,
            size,
            nibbles: Nibbles::from_words(Vec::new()),
            reach: Some((Vec::new(), seen)),
        }
    }

    /// Distance to the goal of the group masks of a node; 15 when 15 or more
    /// (or unreachable).
    #[inline(always)]
    pub(super) fn depth(&self, masks: &[u32]) -> i32 {
        if let Some((_, reach)) = &self.reach {
            let mut key = 0u128;
            for (g, group) in self.groups.iter().enumerate() {
                key |= u128::from(masks[group.slot]) << self.radix[g];
            }
            return reach.get(&key).map_or(15, |&d| i32::from(d));
        }
        let mut index = 0;
        for (g, group) in self.groups.iter().enumerate() {
            index += rank(masks[group.slot]) as usize * self.radix[g];
        }
        self.nibbles.get(index) as i32
    }

    #[allow(dead_code)] // for measuring the tables
    pub(super) fn bytes(&self) -> usize {
        match &self.reach {
            Some((_, r)) => r.len() * 24,
            None => self.nibbles.bytes(),
        }
    }

    #[allow(dead_code)] // for measuring the tables
    pub(super) fn states(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_are_a_bijection() {
        let mut seen = std::collections::HashSet::new();
        for mask in 0u32..(1 << 10) {
            if mask.count_ones() == 4 {
                let r = rank(mask);
                assert!(seen.insert(r));
                assert_eq!(unrank(r, 4), mask);
            }
        }
        assert_eq!(seen.len(), 210);
        assert_eq!(seen.iter().max(), Some(&209));
    }
}
