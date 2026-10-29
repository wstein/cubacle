//! Multi-goal pruning tables for the target-set search.
//!
//! Each table is a projection of the cube (all corners, or `k` tracked edges)
//! with a breadth-first search seeded from every projected state that some
//! view accepts. An entry is the distance to the nearest such state, which can
//! only be shorter than the true distance to the goal, so the maximum of the
//! tables is an admissible bound. Entries are 4 bits; 15 means "15 or more".

use super::goal::Goal;
use super::model::{Pos, Turns, TURNS};

const CORNER_PERMS: usize = 40320;
const CORNER_TWISTS: usize = 2187;
/// The most edges a table tracks: 12P6 * 2^6 = 42.6 million entries.
pub(super) const MAX_TRACKED: usize = 6;
const UNSEEN: u8 = 15;

/// Four-bit cells.
struct Nibbles(Vec<u8>);

impl Nibbles {
    fn filled(len: usize, value: u8) -> Nibbles {
        Nibbles(vec![value | value << 4; len.div_ceil(2)])
    }

    #[inline]
    fn get(&self, i: usize) -> u8 {
        (self.0[i >> 1] >> ((i & 1) * 4)) & 15
    }

    #[inline]
    fn set(&mut self, i: usize, value: u8) {
        let shift = (i & 1) * 4;
        self.0[i >> 1] = (self.0[i >> 1] & !(15 << shift)) | value << shift;
    }
}

/// Distances to the nearest seed over `n` states, or `None` without a seed.
fn bfs(
    n: usize,
    seeding: impl FnOnce(&mut dyn FnMut(usize)),
    next: impl Fn(usize, usize) -> usize,
) -> Option<Nibbles> {
    let mut dist = Nibbles::filled(n, UNSEEN);
    let mut seeds = 0usize;
    seeding(&mut |i| {
        if dist.get(i) == UNSEEN {
            dist.set(i, 0);
            seeds += 1;
        }
    });
    if seeds == 0 {
        return None;
    }
    for level in 0..UNSEEN - 1 {
        let mut grown = 0usize;
        for byte in 0..dist.0.len() {
            let cells = dist.0[byte];
            if cells & 15 != level && cells >> 4 != level {
                continue;
            }
            for half in 0..2 {
                let i = byte * 2 + half;
                if i < n && dist.get(i) == level {
                    for m in 0..TURNS {
                        let j = next(i, m);
                        if dist.get(j) == UNSEEN {
                            dist.set(j, level + 1);
                            grown += 1;
                        }
                    }
                }
            }
        }
        if grown == 0 {
            break;
        }
    }
    Some(dist)
}

fn perm_rank(perm: &[u8; 8]) -> usize {
    let mut rank = 0;
    for i in 0..8 {
        let smaller = perm[i + 1..].iter().filter(|&&p| p < perm[i]).count();
        rank = rank * (8 - i) + smaller;
    }
    rank
}

fn perm_unrank(mut rank: usize) -> [u8; 8] {
    let mut digits = [0usize; 8];
    for i in (0..8).rev() {
        digits[i] = rank % (8 - i);
        rank /= 8 - i;
    }
    let mut free: Vec<u8> = (0..8).collect();
    std::array::from_fn(|i| free.remove(digits[i]))
}

/// The twist coordinate: the first seven twists as base-3 digits.
fn twist_rank(twist: &[u8; 8]) -> usize {
    twist[..7].iter().fold(0, |r, &t| r * 3 + usize::from(t))
}

fn twist_unrank(mut rank: usize) -> [u8; 8] {
    let mut twist = [0u8; 8];
    let mut sum = 0u8;
    for i in (0..7).rev() {
        twist[i] = (rank % 3) as u8;
        sum += twist[i];
        rank /= 3;
    }
    twist[7] = (3 - sum % 3) % 3;
    twist
}

/// The distance table over all eight corners.
pub(super) struct CornerTable {
    perm_mv: Vec<u16>,
    twist_mv: Vec<u16>,
    dist: Nibbles,
}

impl CornerTable {
    /// Bytes the table takes, built or not.
    pub(super) fn bytes() -> usize {
        CORNER_PERMS * CORNER_TWISTS / 2 + (CORNER_PERMS + CORNER_TWISTS) * TURNS * 2
    }

    /// `None` when no view constrains a corner (nothing to prune) or no
    /// corner arrangement meets the goal.
    pub(super) fn build(goal: &Goal, turns: &Turns) -> Option<CornerTable> {
        if goal.views.iter().any(|v| v.corners.is_empty()) {
            return None;
        }
        let mut perm_mv = vec![0u16; CORNER_PERMS * TURNS];
        for rank in 0..CORNER_PERMS {
            let perm = perm_unrank(rank);
            for m in 0..TURNS {
                let moved: [u8; 8] = std::array::from_fn(|t| perm[usize::from(turns.cfrom[m][t])]);
                perm_mv[rank * TURNS + m] = perm_rank(&moved) as u16;
            }
        }
        let mut twist_mv = vec![0u16; CORNER_TWISTS * TURNS];
        for rank in 0..CORNER_TWISTS {
            let twist = twist_unrank(rank);
            for m in 0..TURNS {
                let moved: [u8; 8] = std::array::from_fn(|t| {
                    (twist[usize::from(turns.cfrom[m][t])] + turns.cadd[m][t]) % 3
                });
                twist_mv[rank * TURNS + m] = twist_rank(&moved) as u16;
            }
        }
        let twists: Vec<[u8; 8]> = (0..CORNER_TWISTS).map(twist_unrank).collect();
        let dist = bfs(
            CORNER_PERMS * CORNER_TWISTS,
            |mark| {
                for rank in 0..CORNER_PERMS {
                    let perm = perm_unrank(rank);
                    for (t, twist) in twists.iter().enumerate() {
                        let met = goal.views.iter().any(|view| {
                            view.corners.iter().all(|&(slot, bits)| {
                                let s = usize::from(slot);
                                bits >> (usize::from(perm[s]) * 3 + usize::from(twist[s])) & 1 == 1
                            })
                        });
                        if met {
                            mark(rank * CORNER_TWISTS + t);
                        }
                    }
                }
            },
            |i, m| {
                usize::from(perm_mv[(i / CORNER_TWISTS) * TURNS + m]) * CORNER_TWISTS
                    + usize::from(twist_mv[(i % CORNER_TWISTS) * TURNS + m])
            },
        )?;
        Some(CornerTable {
            perm_mv,
            twist_mv,
            dist,
        })
    }

    pub(super) fn index(pos: &Pos) -> (u16, u16) {
        (perm_rank(&pos.cp) as u16, twist_rank(&pos.co) as u16)
    }

    #[inline]
    pub(super) fn step(&self, at: (u16, u16), m: usize) -> (u16, u16) {
        (
            self.perm_mv[usize::from(at.0) * TURNS + m],
            self.twist_mv[usize::from(at.1) * TURNS + m],
        )
    }

    #[inline]
    pub(super) fn bound(&self, at: (u16, u16)) -> u8 {
        self.dist
            .get(usize::from(at.0) * CORNER_TWISTS + usize::from(at.1))
    }
}

/// Ordered choices of `k` slots out of 12.
fn arrangements(k: usize) -> usize {
    (0..k).map(|j| 12 - j).product()
}

fn slots_rank(slots: &[u8]) -> usize {
    let mut used = 0u16;
    let mut rank = 0;
    for (j, &s) in slots.iter().enumerate() {
        let smaller = (0..s).filter(|&q| used & (1 << q) == 0).count();
        rank = rank * (12 - j) + smaller;
        used |= 1 << s;
    }
    rank
}

fn slots_unrank(mut rank: usize, k: usize) -> Vec<u8> {
    let mut digits = vec![0usize; k];
    for j in (0..k).rev() {
        digits[j] = rank % (12 - j);
        rank /= 12 - j;
    }
    let mut free: Vec<u8> = (0..12).collect();
    digits.iter().map(|&d| free.remove(d)).collect()
}

/// The distance table over the positions and flips of `k` tracked edges.
pub(super) struct EdgeTable {
    pieces: Vec<u8>,
    k: usize,
    /// New arrangement rank in the low 22 bits, flips to xor above them.
    mv: Vec<u32>,
    dist: Nibbles,
}

impl EdgeTable {
    /// Bytes a table of `k` edges takes.
    pub(super) fn bytes(k: usize) -> usize {
        (arrangements(k) << k) / 2 + arrangements(k) * TURNS * 4
    }

    /// The edges worth tracking, in order: those that can fill the fewest
    /// constrained slots first. Empty when some view constrains no edge.
    pub(super) fn candidates(goal: &Goal) -> Vec<u8> {
        if goal.views.iter().any(|v| v.edges.is_empty()) {
            return Vec::new();
        }
        let mut count = [0usize; 12];
        for view in &goal.views {
            for &(_, bits) in &view.edges {
                for (piece, c) in count.iter_mut().enumerate() {
                    if bits >> (piece * 2) & 3 != 0 {
                        *c += 1;
                    }
                }
            }
        }
        let mut pieces: Vec<u8> = (0..12u8).filter(|&p| count[usize::from(p)] > 0).collect();
        pieces.sort_by_key(|&p| count[usize::from(p)]);
        pieces
    }

    /// `None` when no arrangement of the tracked edges meets the goal.
    pub(super) fn build(goal: &Goal, turns: &Turns, pieces: &[u8]) -> Option<EdgeTable> {
        let k = pieces.len();
        let n = arrangements(k);
        let mut to = [[0u8; 12]; TURNS];
        let mut flip = [[0u8; 12]; TURNS];
        for m in 0..TURNS {
            for t in 0..12 {
                let from = usize::from(turns.efrom[m][t]);
                to[m][from] = t as u8;
                flip[m][from] = turns.eadd[m][t];
            }
        }
        let mut mv = vec![0u32; n * TURNS];
        for rank in 0..n {
            let slots = slots_unrank(rank, k);
            for m in 0..TURNS {
                let mut moved = vec![0u8; k];
                let mut flips = 0u32;
                for (j, &s) in slots.iter().enumerate() {
                    moved[j] = to[m][usize::from(s)];
                    flips |= u32::from(flip[m][usize::from(s)]) << j;
                }
                mv[rank * TURNS + m] = slots_rank(&moved) as u32 | flips << 22;
            }
        }
        let tracked: u32 = pieces
            .iter()
            .fold(0, |acc, &p| acc | 3 << (usize::from(p) * 2));
        let dist = bfs(
            n << k,
            |mark| {
                for rank in 0..n {
                    let slots = slots_unrank(rank, k);
                    let mut at = [usize::MAX; 12];
                    for (j, &s) in slots.iter().enumerate() {
                        at[usize::from(s)] = j;
                    }
                    for flips in 0..1usize << k {
                        let met = goal.views.iter().any(|view| {
                            view.edges
                                .iter()
                                .all(|&(slot, bits)| match at[usize::from(slot)] {
                                    usize::MAX => bits & !tracked != 0,
                                    j => {
                                        let piece = usize::from(pieces[j]);
                                        bits >> (piece * 2 + (flips >> j & 1)) & 1 == 1
                                    }
                                })
                        });
                        if met {
                            mark(rank << k | flips);
                        }
                    }
                }
            },
            |i, m| {
                let t = mv[(i >> k) * TURNS + m];
                ((t & 0x3F_FFFF) as usize) << k | ((i & ((1 << k) - 1)) ^ (t >> 22) as usize)
            },
        )?;
        Some(EdgeTable {
            pieces: pieces.to_vec(),
            k,
            mv,
            dist,
        })
    }

    pub(super) fn index(&self, pos: &Pos) -> u32 {
        let mut slots = vec![0u8; self.k];
        let mut flips = 0u32;
        for (j, &piece) in self.pieces.iter().enumerate() {
            let s = (0..12)
                .find(|&s| pos.ep[s] == piece)
                .expect("a cube has every edge");
            slots[j] = s as u8;
            flips |= u32::from(pos.eo[s]) << j;
        }
        (slots_rank(&slots) as u32) << self.k | flips
    }

    #[inline]
    pub(super) fn step(&self, at: u32, m: usize) -> u32 {
        let t = self.mv[(at >> self.k) as usize * TURNS + m];
        (t & 0x3F_FFFF) << self.k | ((at & ((1 << self.k) - 1)) ^ (t >> 22))
    }

    #[inline]
    pub(super) fn bound(&self, at: u32) -> u8 {
        self.dist.get(at as usize)
    }

    pub(super) fn tracked(&self) -> usize {
        self.k
    }
}

/// Which tables fit in `budget` bytes: the corner table first, then the
/// largest edge set of at most [`MAX_TRACKED`] edges that still fits.
pub(super) fn plan(goal: &Goal, budget: usize) -> (bool, Vec<u8>) {
    let mut left = budget;
    let wants_corners = !goal.views.iter().any(|v| v.corners.is_empty());
    let corners = wants_corners && CornerTable::bytes() <= left;
    if corners {
        left -= CornerTable::bytes();
    }
    let candidates = EdgeTable::candidates(goal);
    let mut k = candidates.len().min(MAX_TRACKED);
    while k > 0 && EdgeTable::bytes(k) > left {
        k -= 1;
    }
    (corners, candidates[..k].to_vec())
}
