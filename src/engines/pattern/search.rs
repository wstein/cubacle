//! Meet in the middle over products of macros.
//!
//! The table holds every product of up to [`STORED`] macros applied to the
//! solved cube. A state is looked up after every product of up to [`WALKED`]
//! macros, so a hit is a product of at most `STORED + WALKED` macros that
//! solves it. The search is exhaustive over that space and deterministic; only
//! the optional deadline can cut it short.

use std::collections::HashMap;
use std::time::Instant;

use super::geometry::{apply, hash};
use super::library::Library;
use super::Cap;
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

fn stickers_of(facelets: &Facelets) -> Vec<u8> {
    facelets.stickers().iter().map(|&f| f as u8).collect()
}

/// Bytes one table entry takes, counting the hash map's spare room.
const ENTRY_BYTES: usize = 40;
/// Bytes one frontier item takes besides its stickers: the path, the cost and
/// the vector headers.
const FRONTIER_BYTES: usize = 128;

/// The products of macros that reach each state from the solved cube.
pub(super) struct Table {
    /// Packed products by state; the cost is not needed after the build.
    reached: HashMap<u64, (usize, u64)>,
    capped: bool,
}

impl Table {
    /// Builds the table, stopping early (and saying so, see [`Table::capped`])
    /// when its memory would pass `cap`. Without a cut the table is complete
    /// and the search exhaustive.
    pub(super) fn build(library: &Library, cap: Cap) -> Table {
        let solved = stickers_of(&Facelets::solved(library.kind));
        let mut buffer = vec![0; solved.len()];
        let mut reached: HashMap<u64, (usize, u64)> = HashMap::new();
        reached.insert(hash(&solved), (0, 0));
        let frontier_item = solved.len() + FRONTIER_BYTES;
        let mut frontier: Vec<(Vec<u8>, Vec<usize>, usize)> = vec![(solved, Vec::new(), 0)];
        let mut capped = false;
        'levels: for level in 0..STORED {
            let last = level + 1 == STORED;
            let mut next = Vec::new();
            for (stickers, path, cost) in &frontier {
                let used =
                    reached.len() * ENTRY_BYTES + (frontier.len() + next.len()) * frontier_item;
                if used > cap.max_bytes {
                    capped = true;
                    break 'levels;
                }
                for (m, entry) in library.macros.iter().enumerate() {
                    if path.last().is_some_and(|&p| library.macros[p].inverse == m) {
                        continue;
                    }
                    apply(&entry.step.forward, stickers, &mut buffer);
                    let mut longer = path.clone();
                    longer.push(m);
                    let cost = cost + entry.moves.len();
                    let slot = reached.entry(hash(&buffer)).or_insert((usize::MAX, 0));
                    if cost < slot.0 {
                        *slot = (cost, pack(&longer));
                        // The last level is not expanded, so it needs no frontier.
                        if !last {
                            next.push((buffer.clone(), longer, cost));
                        }
                    }
                }
            }
            frontier = next;
        }
        Table { reached, capped }
    }

    /// Whether the cap cut the build short: products may exist that the
    /// table does not know.
    pub(super) fn capped(&self) -> bool {
        self.capped
    }

    /// The shortest solution that is a product of macros, if one is within
    /// reach before `deadline`; the answer is replayed before it is returned.
    /// With `merge`, turns of one axis are merged into shorter blocks.
    pub(super) fn solve(
        &self,
        library: &Library,
        state: &Facelets,
        deadline: Option<Instant>,
        merge: bool,
    ) -> Option<MoveSequence> {
        let mut walk = Walk {
            library,
            table: self,
            deadline,
            merge,
            nodes: 0,
            expired: false,
            best: None,
        };
        walk.visit(&stickers_of(state), &mut Vec::new());
        let (_, path) = walk.best?;
        let sequence = library.sequence(&path, merge);
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
    merge: bool,
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
        if let Some(&(_, packed)) = self.table.reached.get(&hash(stickers)) {
            let mut full = path.clone();
            full.extend(
                unpack(packed)
                    .iter()
                    .rev()
                    .map(|&m| self.library.macros[m].inverse),
            );
            if !full.is_empty() {
                let length = self.library.sequence(&full, self.merge).len();
                if self.best.as_ref().is_none_or(|(best, _)| length < *best) {
                    self.best = Some((length, full));
                }
            }
        }
        if path.len() == WALKED {
            return;
        }
        let mut moved = vec![0; stickers.len()];
        for (m, entry) in self.library.macros.iter().enumerate() {
            if path
                .last()
                .is_some_and(|&p| self.library.macros[p].inverse == m)
            {
                continue;
            }
            apply(&entry.step.forward, stickers, &mut moved);
            path.push(m);
            self.visit(&moved, path);
            path.pop();
        }
    }
}
