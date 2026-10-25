//! Which part of a move list a block of pictures draws, and how its moves are
//! numbered.

use cubacle::phases::Phase;

/// A block of move pictures inside a longer story: `solve --now --icons` draws the
/// rollback (`R1`, `R2`, ...) and the rest of the printed solution (its own
/// move numbers) as two blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Block<'a> {
    /// The number of the move before the first one drawn (global numbering).
    pub base: usize,
    /// The moves in the whole list the numbers count in: `move 21 of 27`.
    pub total: usize,
    /// Written before each number: `R` for the rollback.
    pub prefix: &'a str,
    /// Whether the block ends with the end state picture and its token line.
    pub end: bool,
}

impl Block<'_> {
    /// A whole move list, numbered from 1, ending with the end state.
    pub(crate) fn whole(total: usize) -> Block<'static> {
        Block {
            base: 0,
            total,
            prefix: "",
            end: true,
        }
    }

    /// The label of the move at `at` (counted from 0 inside the block).
    pub(crate) fn label(&self, at: usize) -> String {
        format!("{}{}", self.prefix, self.base + at + 1)
    }
}

/// The phases cut to the moves from global index `base` on, still numbered
/// globally. A phase that ended before is dropped.
pub(crate) fn clipped(found: &[Phase], base: usize) -> Vec<Phase> {
    found
        .iter()
        .filter(|p| p.moves.end > base || (p.moves.is_empty() && p.moves.start >= base))
        .map(|p| Phase {
            number: p.number,
            name: p.name,
            moves: p.moves.start.max(base)..p.moves.end,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_count_globally_with_the_prefix() {
        let rollback = Block {
            base: 0,
            total: 3,
            prefix: "R",
            end: false,
        };
        assert_eq!(rollback.label(0), "R1");
        let rest = Block {
            base: 20,
            total: 27,
            prefix: "",
            end: false,
        };
        assert_eq!(rest.label(0), "21");
        assert_eq!(rest.label(6), "27");
    }

    #[test]
    fn phases_are_cut_at_the_first_move_drawn() {
        let phase = |number, moves| Phase {
            number,
            name: "p",
            moves,
        };
        let found = [phase(1, 0..0), phase(2, 0..10), phase(3, 10..27)];
        let cut = clipped(&found, 20);
        assert_eq!(cut.len(), 1);
        assert_eq!((cut[0].number, cut[0].moves.clone()), (3, 20..27));
    }
}
