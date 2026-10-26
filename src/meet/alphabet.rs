//! The blocks a search may turn: single layers and a few wide blocks.
//!
//! A block is a run of adjacent layers on one axis, turned a quarter, a half
//! or a quarter back. A [`Kit`] picks the set: the outer blocks the normal
//! solver writes, with the single inner layers, or with every block up to
//! three layers and the layers 2-3 (the form `3Bw B'` writes). Whole-cube
//! rotations are never blocks.

use std::collections::BTreeSet;

use crate::puzzle::{destinations, Face, LayerMove, Move, MoveSequence, PuzzleKind, Turn};

/// Layers `lo..=hi` counted from the negative face (L, D, B) of an axis.
type Range = (usize, usize);

/// Which blocks an alphabet holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kit {
    /// The outer blocks of either side up to half the cube (at most three
    /// layers): face turns on the 2x2x2 and 3x3x3, `Rw` and `3Rw` above.
    Outer,
    /// `Outer` and the single inner layers (`2R`, `M`).
    Slices,
    /// Every block of up to three outer layers from either side, single
    /// layers and the layers 2-3.
    Blocks,
}

pub(crate) struct Alphabet {
    pub(crate) kind: PuzzleKind,
    ranges: Vec<Range>,
    /// Sticker `i` goes to `perms[k][i]`.
    pub(crate) perms: Vec<Vec<u16>>,
    /// The stickers a move displaces, as (from, to).
    pub(crate) moved: Vec<Vec<(u16, u16)>>,
    axis: Vec<u8>,
    range: Vec<u8>,
    /// The move that undoes each move.
    pub(crate) inverse: Vec<u8>,
    /// The most stickers one move displaces.
    pub(crate) max_moved: usize,
    /// The orbit of each sticker under the moves.
    orbit: Vec<u16>,
    /// The most stickers of each orbit one move displaces.
    orbit_moved: Vec<usize>,
}

const NEGATIVE: [Face; 3] = [Face::L, Face::D, Face::B];
const POSITIVE: [Face; 3] = [Face::R, Face::U, Face::F];

fn ranges_of(n: usize, kit: Kit) -> Vec<Range> {
    let mut set: BTreeSet<Range> = BTreeSet::new();
    match kit {
        Kit::Outer | Kit::Slices => {
            for w in 1..=(n / 2).min(3) {
                set.insert((0, w - 1));
                set.insert((n - w, n - 1));
            }
            if kit == Kit::Slices {
                set.extend((0..n).map(|i| (i, i)));
            }
        }
        Kit::Blocks => {
            set.extend((0..n).map(|i| (i, i)));
            for w in 2..=3.min(n) {
                set.insert((0, w - 1));
                set.insert((n - w, n - 1));
            }
            if n >= 4 {
                set.insert((1, 2));
                set.insert((n - 3, n - 2));
            }
        }
    }
    set.remove(&(0, n - 1));
    set.into_iter().collect()
}

/// The plain moves that turn `range` of `axis` by `q` quarters (1 to 3).
fn moves_of(n: usize, axis: usize, (lo, hi): Range, q: usize) -> Vec<Move> {
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][q - 1];
    let block = if hi == n - 1 && lo > 0 {
        LayerMove::new(POSITIVE[axis], 1, (n - lo) as u8, turn)
    } else {
        LayerMove::new(NEGATIVE[axis], (lo + 1) as u8, (hi + 1) as u8, turn)
    };
    block.to_moves()
}

/// The sticker permutation of plain moves, performed in order.
pub(crate) fn perm_of(kind: PuzzleKind, moves: &[Move]) -> Vec<u16> {
    let mut perm: Vec<u16> = (0..kind.facelet_count() as u16).collect();
    for mv in moves {
        let step = destinations(kind, mv);
        for slot in &mut perm {
            *slot = step[*slot as usize] as u16;
        }
    }
    perm
}

impl Alphabet {
    pub(crate) fn new(kind: PuzzleKind, kit: Kit) -> Alphabet {
        let n = kind.size();
        let ranges = ranges_of(n, kit);
        let (mut perms, mut axis, mut range) = (Vec::new(), Vec::new(), Vec::new());
        for a in 0..3 {
            for (r, &span) in ranges.iter().enumerate() {
                for q in 1..=3 {
                    perms.push(perm_of(kind, &moves_of(n, a, span, q)));
                    axis.push(a as u8);
                    range.push(r as u8);
                }
            }
        }
        let inverse = (0..perms.len())
            .map(|k| (k - k % 3 + 2 - k % 3) as u8)
            .collect();
        let moved: Vec<Vec<(u16, u16)>> = perms
            .iter()
            .map(|p| {
                (0..p.len() as u16)
                    .filter(|&i| p[i as usize] != i)
                    .map(|i| (i, p[i as usize]))
                    .collect()
            })
            .collect();
        let max_moved = moved.iter().map(Vec::len).max().unwrap_or(0);
        let orbit = orbits_of(&perms);
        let mut orbit_moved =
            vec![0usize; orbit.iter().map(|&o| o as usize + 1).max().unwrap_or(0)];
        for stickers in &moved {
            let mut count = vec![0usize; orbit_moved.len()];
            for &(from, _) in stickers {
                count[orbit[from as usize] as usize] += 1;
            }
            for (most, c) in orbit_moved.iter_mut().zip(count) {
                *most = (*most).max(c);
            }
        }
        Alphabet {
            kind,
            ranges,
            perms,
            moved,
            axis,
            range,
            inverse,
            max_moved,
            orbit,
            orbit_moved,
        }
    }

    /// A lower bound on the moves between two states (color per sticker), or
    /// `None` when no sequence of the alphabet connects them.
    ///
    /// Proof. A move changes the color only of stickers it displaces, so after
    /// `k` moves at most `k * m` stickers of an orbit differ from the start,
    /// where `m` is the most stickers of that orbit one move displaces. The
    /// states differ at `d` stickers of the orbit, so `d <= k * m` and
    /// `k >= ceil(d / m)`; the bound is the largest over the orbits. If `m`
    /// is 0 (the moves never touch the orbit, such as the centers of a 3x3x3)
    /// and `d > 0` no number of moves helps.
    pub(crate) fn lower_bound(&self, a: &[u8], b: &[u8]) -> Option<usize> {
        let mut differing = vec![0usize; self.orbit_moved.len()];
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            if x != y {
                differing[self.orbit[i] as usize] += 1;
            }
        }
        let mut bound = 0;
        for (d, &m) in differing.into_iter().zip(&self.orbit_moved) {
            if d > 0 {
                if m == 0 {
                    return None;
                }
                bound = bound.max(d.div_ceil(m));
            }
        }
        Some(bound)
    }

    pub(crate) fn len(&self) -> usize {
        self.perms.len()
    }

    /// Whether `next` may follow `last`: blocks of one axis commute, so a run
    /// of them is only allowed in increasing order.
    pub(crate) fn may_follow(&self, last: Option<u8>, next: usize) -> bool {
        match last {
            Some(l) => {
                let l = l as usize;
                self.axis[l] != self.axis[next] || self.range[l] < self.range[next]
            }
            None => true,
        }
    }

    /// How many sequences of 0 to `depth` blocks exist.
    pub(crate) fn count_up_to(&self, depth: usize) -> u64 {
        let m = self.len();
        let mut at_level: Vec<u64> = Vec::new();
        let mut total: u64 = 1;
        for level in 0..depth {
            let next: Vec<u64> = (0..m)
                .map(|k| {
                    if level == 0 {
                        1
                    } else {
                        (0..m)
                            .filter(|&l| self.may_follow(Some(l as u8), k))
                            .map(|l| at_level[l])
                            .fold(0u64, u64::saturating_add)
                    }
                })
                .collect();
            total = next.iter().fold(total, |t, &c| t.saturating_add(c));
            at_level = next;
        }
        total
    }

    /// The plain moves of a block sequence.
    pub(crate) fn sequence(&self, blocks: &[u8]) -> MoveSequence {
        let n = self.kind.size();
        let per_axis = self.ranges.len() * 3;
        blocks
            .iter()
            .flat_map(|&k| {
                let k = k as usize;
                let (a, rest) = (k / per_axis, k % per_axis);
                moves_of(n, a, self.ranges[rest / 3], rest % 3 + 1)
            })
            .collect()
    }

    /// The inverse of a block sequence.
    pub(crate) fn invert(&self, blocks: &[u8]) -> Vec<u8> {
        blocks
            .iter()
            .rev()
            .map(|&k| self.inverse[k as usize])
            .collect()
    }
}

/// The orbit of every sticker under the permutations (union of `i` and `perm[i]`).
fn orbits_of(perms: &[Vec<u16>]) -> Vec<u16> {
    let count = perms.first().map_or(0, Vec::len);
    let mut parent: Vec<usize> = (0..count).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for perm in perms {
        for (i, &j) in perm.iter().enumerate() {
            let (a, b) = (root(&mut parent, i), root(&mut parent, j as usize));
            parent[a.max(b)] = a.min(b);
        }
    }
    let mut ids: Vec<usize> = Vec::new();
    (0..count)
        .map(|i| {
            let r = root(&mut parent, i);
            match ids.iter().position(|&x| x == r) {
                Some(id) => id as u16,
                None => {
                    ids.push(r);
                    (ids.len() - 1) as u16
                }
            }
        })
        .collect()
}
