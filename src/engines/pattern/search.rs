//! Meet in the middle over products of macros.
//!
//! The table holds every product of up to [`STORED`] macros applied to the
//! solved cube. A state is looked up after every product of up to [`WALKED`]
//! macros, so a hit is a product of at most `STORED + WALKED` macros that
//! solves it. The search is exhaustive over that space and deterministic; only
//! the optional deadline can cut it short.

use std::collections::HashMap;
use std::time::Instant;

use super::library::Library;
use crate::puzzle::{Facelets, MoveSequence};

/// Products stored from the solved cube.
const STORED: usize = 2;
/// Products tried from the cube to solve.
const WALKED: usize = 2;

/// Macro numbers are packed 16 bits each, plus one so that 0 ends the path.
fn pack(path: &[usize]) -> u64 {
    path.iter()
        .enumerate()
        .fold(0, |packed, (i, &m)| packed | ((m as u64 + 1) << (16 * i)))
}

fn unpack(mut packed: u64) -> Vec<usize> {
    let mut path = Vec::new();
    while packed != 0 {
        path.push((packed & 0xffff) as usize - 1);
        packed >>= 16;
    }
    path
}

fn hash(stickers: &[u8]) -> u128 {
    let mut a = 0xcbf2_9ce4_8422_2325u64;
    let mut b = 0x9e37_79b9_7f4a_7c15u64;
    for &x in stickers {
        a = (a ^ u64::from(x)).wrapping_mul(0x0100_0000_01b3);
        b = (b.rotate_left(5) ^ u64::from(x)).wrapping_mul(0xff51_afd7_ed55_8ccd);
    }
    (u128::from(a) << 64) | u128::from(b)
}

fn turned(stickers: &[u8], dest: &[u16]) -> Vec<u8> {
    let mut out = vec![0; stickers.len()];
    for (i, &d) in dest.iter().enumerate() {
        out[usize::from(d)] = stickers[i];
    }
    out
}

fn stickers_of(facelets: &Facelets) -> Vec<u8> {
    facelets.stickers().iter().map(|&f| f as u8).collect()
}

/// The products of macros that reach each state from the solved cube.
pub(super) struct Table {
    reached: HashMap<u128, u64>,
}

impl Table {
    pub(super) fn build(library: &Library) -> Table {
        let solved = stickers_of(&Facelets::solved(library.kind));
        let mut reached: HashMap<u128, (usize, u64)> = HashMap::new();
        reached.insert(hash(&solved), (0, 0));
        let mut frontier: Vec<(Vec<u8>, Vec<usize>, usize)> = vec![(solved, Vec::new(), 0)];
        for _ in 0..STORED {
            let mut next = Vec::new();
            for (stickers, path, cost) in &frontier {
                for (m, entry) in library.macros.iter().enumerate() {
                    if path.last().is_some_and(|&p| library.macros[p].inverse == m) {
                        continue;
                    }
                    let moved = turned(stickers, &entry.dest);
                    let mut longer = path.clone();
                    longer.push(m);
                    let cost = cost + entry.moves.len();
                    let slot = reached.entry(hash(&moved)).or_insert((usize::MAX, 0));
                    if cost < slot.0 {
                        *slot = (cost, pack(&longer));
                        next.push((moved, longer, cost));
                    }
                }
            }
            frontier = next;
        }
        Table {
            reached: reached.into_iter().map(|(k, (_, p))| (k, p)).collect(),
        }
    }

    /// The shortest solution that is a product of macros, if one is within
    /// reach before `deadline`; the answer is replayed before it is returned.
    pub(super) fn solve(
        &self,
        library: &Library,
        state: &Facelets,
        deadline: Option<Instant>,
    ) -> Option<MoveSequence> {
        let mut walk = Walk {
            library,
            table: self,
            deadline,
            nodes: 0,
            expired: false,
            best: None,
        };
        walk.visit(&stickers_of(state), &mut Vec::new());
        let (_, path) = walk.best?;
        let sequence = library.sequence(&path);
        state
            .apply(&sequence)
            .is_ok_and(|end| end.is_solved())
            .then_some(sequence)
    }
}

struct Walk<'a> {
    library: &'a Library,
    table: &'a Table,
    deadline: Option<Instant>,
    nodes: u64,
    expired: bool,
    best: Option<(usize, Vec<usize>)>,
}

impl Walk<'_> {
    fn visit(&mut self, stickers: &[u8], path: &mut Vec<usize>) {
        self.nodes += 1;
        if self.nodes % 1024 == 0 && self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.expired = true;
        }
        if self.expired {
            return;
        }
        if let Some(&packed) = self.table.reached.get(&hash(stickers)) {
            let mut full = path.clone();
            full.extend(
                unpack(packed)
                    .iter()
                    .rev()
                    .map(|&m| self.library.macros[m].inverse),
            );
            if !full.is_empty() {
                let length = self.library.sequence(&full).len();
                if self.best.as_ref().is_none_or(|(best, _)| length < *best) {
                    self.best = Some((length, full));
                }
            }
        }
        if path.len() == WALKED {
            return;
        }
        for (m, entry) in self.library.macros.iter().enumerate() {
            if path
                .last()
                .is_some_and(|&p| self.library.macros[p].inverse == m)
            {
                continue;
            }
            let moved = turned(stickers, &entry.dest);
            path.push(m);
            self.visit(&moved, path);
            path.pop();
        }
    }
}
