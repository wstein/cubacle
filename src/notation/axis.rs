//! Hand units by axis: consecutive turns about one axis whose layers do not
//! overlap are one unit (the axial move of the ATM metric).

use super::expr::{Group, Item};
use crate::puzzle::{Move, PuzzleKind, Turn};

/// What a move turns: the axis, a bit for each layer (bit 0 is the layer of
/// U, R or F) and the spin about the axis counted from U, R or F: 1 or -1 for
/// a quarter turn, 2 for a half turn.
#[derive(Clone, Copy)]
struct Block {
    axis: usize,
    mask: u16,
    spin: i8,
}

fn block_of(mv: &Move, n: u8) -> Block {
    let from = if mv.wide { 1 } else { mv.depth };
    let to = mv.depth.min(n);
    let (lo, hi) = if mv.face.index() >= 3 {
        (n + 1 - to, n + 1 - from.min(n))
    } else {
        (from.min(n), to)
    };
    let mut mask = 0u16;
    for layer in lo..=hi {
        mask |= 1 << (layer - 1);
    }
    let sign = if mv.face.index() >= 3 { -1 } else { 1 };
    let spin = match mv.turn {
        Turn::Clockwise => sign,
        Turn::CounterClockwise => -sign,
        Turn::Half => 2,
    };
    Block {
        axis: mv.face.index() % 3,
        mask,
        spin,
    }
}

/// The unit being collected: the layers still turned by each move of it (a
/// layer given back by a nested turn, as in `3Rw R'`, is free again).
#[derive(Default)]
struct Open {
    axis: Option<usize>,
    entries: Vec<(u16, i8)>,
    moves: Vec<Item>,
}

impl Open {
    fn layers(&self) -> u16 {
        self.entries.iter().fold(0, |all, (mask, _)| all | mask)
    }

    fn start(&mut self, axis: usize) {
        self.axis = Some(axis);
    }

    fn take(&mut self, out: &mut Vec<Item>) {
        if !self.moves.is_empty() {
            out.push(Item::Group(Group {
                items: std::mem::take(&mut self.moves),
            }));
        }
        self.axis = None;
        self.entries.clear();
    }

    /// Whether a turn of `block` can join: its layers are free, or it takes
    /// back layers of an earlier, wider turn (a layer range written as two).
    fn admit(&mut self, block: Block) -> bool {
        if self.axis != Some(block.axis) {
            return false;
        }
        if block.mask & self.layers() == 0 {
            self.entries.push((block.mask, block.spin));
            return true;
        }
        let inverse = if block.spin == 2 { 2 } else { -block.spin };
        for entry in &mut self.entries {
            if entry.1 == inverse && entry.0 & block.mask == block.mask && entry.0 != block.mask {
                entry.0 &= !block.mask;
                return true;
            }
        }
        false
    }
}

/// The moves cut into hand units by axis. A unit ends where `breaks` has an
/// index (a move that must start a unit, such as the first one of a solver
/// phase), where the axis changes, or where a turn would overlap the layers
/// already in the unit. A whole-cube rotation (one turn of every layer, or two
/// turns of the layers of an axis that together make every layer turn the same
/// way) joins a unit of its own axis, like a rotation in ATM. The result is a
/// root group of units, as [`super::parse_grouped`] returns for `·`.
pub fn axis_units(moves: &[Move], kind: PuzzleKind, breaks: &[usize]) -> Group {
    let n = u8::try_from(kind.size()).expect("sizes are small");
    let all: u16 = (1 << n) - 1;
    let mut items = Vec::new();
    let mut open = Open::default();
    let mut at = 0;
    while at < moves.len() {
        if breaks.contains(&at) {
            open.take(&mut items);
        }
        let block = block_of(&moves[at], n);
        let pair = moves.get(at + 1).filter(|_| !breaks.contains(&(at + 1)));
        let rotation = if block.mask == all {
            Some(1)
        } else {
            pair.map(|next| block_of(next, n))
                .filter(|next| {
                    next.axis == block.axis
                        && next.mask & block.mask == 0
                        && next.mask | block.mask == all
                        && next.spin == block.spin
                })
                .map(|_| 2)
        };
        match rotation {
            Some(count) => {
                if open.axis != Some(block.axis) {
                    open.take(&mut items);
                    open.start(block.axis);
                }
                // a rotation cannot be nested into or taken back
                open.entries.push((all, 0));
                open.moves
                    .extend(moves[at..at + count].iter().map(|mv| Item::Move(*mv)));
                at += count;
            }
            None => {
                if !open.admit(block) {
                    open.take(&mut items);
                    open.start(block.axis);
                    open.entries.push((block.mask, block.spin));
                }
                open.moves.push(Item::Move(moves[at]));
                at += 1;
            }
        }
    }
    open.take(&mut items);
    Group { items }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::{parse_expression, parse_grouped};

    fn units(text: &str, size: usize) -> Vec<usize> {
        let kind = PuzzleKind::of_size(size).unwrap();
        let moves = parse_expression(text, kind).unwrap();
        let tree = axis_units(moves.moves(), kind, &[]);
        assert_eq!(tree.flatten(), moves.moves());
        tree.items
            .iter()
            .map(|item| match item {
                Item::Group(unit) => unit.items.len(),
                Item::Move(_) => unreachable!("the root holds units"),
            })
            .collect()
    }

    #[test]
    fn opposite_faces_form_a_unit() {
        assert_eq!(units("U D' R L F B2", 3), [2, 2, 2]);
        assert_eq!(units("R U", 3), [1, 1]);
    }

    #[test]
    fn overlapping_layers_start_a_new_unit() {
        assert_eq!(units("U D' U", 3), [2, 1]);
        assert_eq!(units("R R", 3), [1, 1]);
        assert_eq!(units("R L R'", 3), [2, 1]);
    }

    #[test]
    fn wide_and_inner_turns_on_big_cubes() {
        assert_eq!(units("Rw Lw", 4), [2]);
        assert_eq!(units("Rw R", 4), [1, 1]);
        assert_eq!(units("R 2R 3R", 4), [3]);
        assert_eq!(units("2R 2L' 3U", 5), [2, 1]);
        assert_eq!(units("3Rw 3Lw'", 7), [2]);
    }

    #[test]
    fn a_range_is_one_block() {
        // 2-3Rw is written 3Rw R'
        assert_eq!(units("2-3Rw", 5), [2]);
        assert_eq!(units("2-3Rw 2-3Lw", 7), [4]);
        assert_eq!(units("2-3Rw R", 5), [3]);
        assert_eq!(units("2-3Rw 3R", 5), [2, 1]);
        assert_eq!(units("N2-4R U", 6), [2, 1]);
    }

    #[test]
    fn rotations_join_the_run_of_their_axis() {
        assert_eq!(units("R x", 3), [3]);
        assert_eq!(units("x R", 3), [2, 1]);
        assert_eq!(units("R y", 3), [1, 2]);
        assert_eq!(units("x' x", 3), [4]);
        assert_eq!(units("R x L", 3), [3, 1]);
        assert_eq!(units("x M", 3), [2, 1]);
        assert_eq!(units("Rw Lw' U", 4), [2, 1]);
        assert_eq!(units("R2 x", 2), [3]);
    }

    #[test]
    fn breaks_end_a_unit() {
        let kind = PuzzleKind::Cube3;
        let moves = parse_expression("U D' R", kind).unwrap();
        let tree = axis_units(moves.moves(), kind, &[1]);
        assert_eq!(tree.items.len(), 3);
    }

    #[test]
    fn dotted_text_reads_back_as_the_same_units() {
        let kind = PuzzleKind::Cube5;
        let moves = parse_expression("U D' R 2-3Lw x F B' R2", kind).unwrap();
        let tree = axis_units(moves.moves(), kind, &[]);
        let text = tree
            .items
            .iter()
            .map(|item| match item {
                Item::Group(unit) => unit
                    .flatten()
                    .iter()
                    .map(|mv| mv.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                Item::Move(_) => unreachable!(),
            })
            .collect::<Vec<_>>()
            .join(" · ");
        assert_eq!(parse_grouped(&text, kind).unwrap(), tree);
    }
}
