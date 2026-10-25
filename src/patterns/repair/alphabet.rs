//! The blocks a repair may turn: single layers and a few wide blocks.
//!
//! A block is a run of adjacent layers on one axis, turned a quarter, a half
//! or a quarter back. The set is single layers, the outer two and three layers
//! from either side, and the layers 2-3 (the form `3Bw B'` writes). Whole-cube
//! rotations are never blocks.

use std::collections::BTreeSet;

use crate::puzzle::{destinations, Face, LayerMove, Move, MoveSequence, PuzzleKind, Turn};

/// Layers `lo..=hi` counted from the negative face (L, D, B) of an axis.
type Range = (usize, usize);

pub(super) struct Alphabet {
    pub(super) kind: PuzzleKind,
    ranges: Vec<Range>,
    /// Sticker `i` goes to `perms[k][i]`.
    pub(super) perms: Vec<Vec<u16>>,
    /// The stickers a move displaces, as (from, to).
    pub(super) moved: Vec<Vec<(u16, u16)>>,
    axis: Vec<u8>,
    range: Vec<u8>,
    /// The move that undoes each move.
    pub(super) inverse: Vec<u8>,
    /// The most stickers one move displaces.
    pub(super) max_moved: usize,
}

const NEGATIVE: [Face; 3] = [Face::L, Face::D, Face::B];
const POSITIVE: [Face; 3] = [Face::R, Face::U, Face::F];

fn ranges_of(n: usize) -> Vec<Range> {
    let mut set: BTreeSet<Range> = (0..n).map(|i| (i, i)).collect();
    for w in 2..=3.min(n) {
        set.insert((0, w - 1));
        set.insert((n - w, n - 1));
    }
    if n >= 4 {
        set.insert((1, 2));
        set.insert((n - 3, n - 2));
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
pub(super) fn perm_of(kind: PuzzleKind, moves: &[Move]) -> Vec<u16> {
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
    pub(super) fn new(kind: PuzzleKind) -> Alphabet {
        let n = kind.size();
        let ranges = ranges_of(n);
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
        Alphabet {
            kind,
            ranges,
            perms,
            moved,
            axis,
            range,
            inverse,
            max_moved,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.perms.len()
    }

    /// Whether `next` may follow `last`: blocks of one axis commute, so a run
    /// of them is only allowed in increasing order.
    pub(super) fn may_follow(&self, last: Option<u8>, next: usize) -> bool {
        match last {
            Some(l) => {
                let l = l as usize;
                self.axis[l] != self.axis[next] || self.range[l] < self.range[next]
            }
            None => true,
        }
    }

    /// How many sequences of 0 to `depth` blocks exist.
    pub(super) fn count_up_to(&self, depth: usize) -> u64 {
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
    pub(super) fn sequence(&self, blocks: &[u8]) -> MoveSequence {
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
    pub(super) fn invert(&self, blocks: &[u8]) -> Vec<u8> {
        blocks
            .iter()
            .rev()
            .map(|&k| self.inverse[k as usize])
            .collect()
    }
}
