//! The meeting itself: a walk from one end checked against the stored table of
//! the other, and how the two halves make one sequence.

use super::{Alphabet, Table, Walker};

/// `A B^-1` from the walked blocks `A` and the stored blocks `B` when the
/// walked side comes first, else `B A^-1`.
fn joined(al: &Alphabet, walked_first: bool, walked: &[u8], stored: &[u8]) -> Vec<u8> {
    let (a, b) = if walked_first {
        (walked, stored)
    } else {
        (stored, walked)
    };
    let mut x = a.to_vec();
    x.extend(al.invert(b));
    x
}

/// Walks every sequence of exactly `walked` blocks from `start` and returns
/// the joined sequences of every hit in `table` that is shorter than `bound`.
/// `None` when the walker's deadline ended the walk.
pub(crate) fn hits(
    walker: &mut Walker<'_>,
    table: &Table,
    start: &[u8],
    walked: usize,
    bound: usize,
    walked_first: bool,
) -> Option<Vec<Vec<u8>>> {
    let al = walker.alphabet();
    let mut found: Vec<Vec<u8>> = Vec::new();
    walker.walk(start, walked, false, &mut |h, seq| {
        if let Some(stored) = table.get(h) {
            if stored.len() + seq.len() < bound {
                found.push(joined(al, walked_first, seq, &stored));
            }
        }
    });
    (!walker.aborted).then_some(found)
}

/// The deepest stored side (at most `blocks` deep) that fits `memory_mb`
/// megabytes; 0 when not even one level fits.
pub(crate) fn stored_depth(al: &Alphabet, memory_mb: usize, blocks: usize) -> usize {
    let cap = memory_mb.saturating_mul(1 << 20) / super::ENTRY_BYTES;
    (0..=blocks)
        .take_while(|&d| al.count_up_to(d) <= cap as u64)
        .last()
        .unwrap_or(0)
}
