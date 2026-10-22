//! The 4-bit pruning table: sixteen distances to a goal in eight per word.
//!
//! Ported from cs0x7f/cube555 src/Util.java (PruningTable), MIT, Copyright
//! (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//! Entry `i` lives in bits `4 * (i % 8)` of word `i / 8`, as there.

/// A table of 4-bit entries.
pub(crate) struct Nibbles {
    words: Vec<u32>,
}

impl Nibbles {
    pub(crate) fn from_words(words: Vec<u32>) -> Nibbles {
        Nibbles { words }
    }

    /// Words needed for `entries` entries.
    pub(crate) fn words_for(entries: usize) -> usize {
        entries.div_ceil(8)
    }

    pub(crate) fn words(&self) -> &[u32] {
        &self.words
    }

    #[inline(always)]
    pub(crate) fn get(&self, index: usize) -> u32 {
        (self.words[index >> 3] >> ((index & 7) << 2)) & 0xf
    }

    /// Gives every unknown entry (`0xf`) of the first `entries` the value
    /// `value`, so that a table that stops early still never overestimates.
    pub(crate) fn fill_unknown(&mut self, entries: usize, value: u32) {
        for (w, word) in self.words.iter_mut().enumerate() {
            for j in 0..8 {
                if w * 8 + j < entries && (*word >> (j * 4)) & 0xf == 0xf {
                    *word = (*word & !(0xf << (j * 4))) | (value << (j * 4));
                }
            }
        }
    }

    /// Bytes held.
    pub(crate) fn bytes(&self) -> usize {
        self.words.len() * 4
    }
}

/// A pruning table over pairs `(state1, state2)`, or over one state.
///
/// `mask` folds the pair index into the table (all ones unless the table is
/// a hash of a larger space, as for the first phase's centres).
pub(crate) struct Prun {
    table: Nibbles,
    stride: usize,
    mask: usize,
}

impl Prun {
    pub(crate) fn new(table: Nibbles, stride: usize, mask: usize) -> Prun {
        Prun {
            table,
            stride,
            mask,
        }
    }

    pub(crate) fn nibbles(&self) -> &Nibbles {
        &self.table
    }

    #[inline(always)]
    pub(crate) fn get2(&self, state1: usize, state2: usize) -> i32 {
        self.table
            .get(state1.wrapping_mul(self.stride).wrapping_add(state2) & self.mask) as i32
    }

    #[inline(always)]
    pub(crate) fn get1(&self, state: usize) -> i32 {
        self.table.get(state & self.mask) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_sit_in_the_low_nibble_first() {
        let table = Nibbles::from_words(vec![0x7654_3210, 0x0000_00f8]);
        assert_eq!(
            (0..9).map(|i| table.get(i)).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(table.get(9), 0xf);
    }

    #[test]
    fn pair_index_folds_by_the_mask() {
        let prun = Prun::new(Nibbles::from_words(vec![0x7654_3210]), 4, 7);
        assert_eq!(prun.get2(1, 2), 6);
        assert_eq!(prun.get2(3, 3), 7); // 15 & 7
        assert_eq!(prun.get1(9), 1);
    }
}
