//! Hand units: the `·` and `( )` structure of an expression, kept next to
//! the flat move list.

use super::inverse;
use crate::puzzle::Move;

/// The most group marks an expression may expand to.
pub(super) const MAX_MARKS: usize = 400_000;

/// What a mark says about the flat move list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarkKind {
    /// A `·`: the group of the current level ends here.
    Dot,
    Open,
    Close,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Mark {
    /// Index of the move the mark stands before.
    pub(super) at: usize,
    pub(super) kind: MarkKind,
}

/// Moves with their marks, while an expression is read.
#[derive(Clone, Default)]
pub(super) struct Run {
    pub(super) moves: Vec<Move>,
    pub(super) marks: Vec<Mark>,
}

impl Run {
    pub(super) fn plain(moves: Vec<Move>) -> Run {
        Run {
            moves,
            marks: Vec::new(),
        }
    }

    pub(super) fn append(&mut self, other: Run) {
        let shift = self.moves.len();
        self.marks.extend(other.marks.into_iter().map(|m| Mark {
            at: m.at + shift,
            kind: m.kind,
        }));
        self.moves.extend(other.moves);
    }

    /// The run inside parentheses.
    pub(super) fn enclosed(mut self) -> Run {
        let end = self.moves.len();
        self.marks.insert(
            0,
            Mark {
                at: 0,
                kind: MarkKind::Open,
            },
        );
        self.marks.push(Mark {
            at: end,
            kind: MarkKind::Close,
        });
        self
    }

    pub(super) fn repeat(self, count: usize) -> Run {
        let mut out = Run::default();
        for _ in 1..count {
            out.append(self.clone());
        }
        if count >= 1 {
            out.append(self);
        }
        out
    }

    /// Undone: reversed, with the marks mirrored.
    pub(super) fn inverse(self) -> Run {
        let len = self.moves.len();
        Run {
            moves: inverse(&self.moves),
            marks: self
                .marks
                .iter()
                .rev()
                .map(|m| Mark {
                    at: len - m.at,
                    kind: match m.kind {
                        MarkKind::Open => MarkKind::Close,
                        MarkKind::Close => MarkKind::Open,
                        MarkKind::Dot => MarkKind::Dot,
                    },
                })
                .collect(),
        }
    }
}

/// A move or a nested group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Move(Move),
    Group(Group),
}

/// A run of items. The root returned by [`super::parse_grouped`] holds the hand
/// units (groups of moves) and the parenthesised blocks (groups of units).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Group {
    pub items: Vec<Item>,
}

impl Group {
    /// The moves in order, groups dissolved.
    pub fn flatten(&self) -> Vec<Move> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }

    fn collect(&self, out: &mut Vec<Move>) {
        for item in &self.items {
            match item {
                Item::Move(mv) => out.push(*mv),
                Item::Group(group) => group.collect(out),
            }
        }
    }

    /// Whether the group holds moves only (a hand unit).
    pub fn is_unit(&self) -> bool {
        self.items.iter().all(|i| matches!(i, Item::Move(_)))
    }

    pub(super) fn build(moves: &[Move], marks: &[Mark]) -> Group {
        let mut builder = Builder::default();
        let mut marks = marks.iter().peekable();
        for index in 0..=moves.len() {
            while let Some(mark) = marks.next_if(|m| m.at == index) {
                match mark.kind {
                    MarkKind::Dot => builder.flush(),
                    MarkKind::Open => {
                        // parentheses inside a block are flattened
                        if builder.open == 0 {
                            builder.flush();
                        }
                        builder.open += 1;
                    }
                    MarkKind::Close => {
                        if builder.open <= 1 {
                            builder.flush();
                        }
                        builder.open = builder.open.saturating_sub(1);
                        if builder.open == 0 && !builder.block.is_empty() {
                            let items = std::mem::take(&mut builder.block);
                            builder.root.push(Item::Group(Group { items }));
                        }
                    }
                }
            }
            if let Some(mv) = moves.get(index) {
                builder.unit.push(Item::Move(*mv));
            }
        }
        builder.flush();
        Group {
            items: builder.root,
        }
    }
}

#[derive(Default)]
struct Builder {
    root: Vec<Item>,
    block: Vec<Item>,
    unit: Vec<Item>,
    open: usize,
}

impl Builder {
    /// Ends the unit being collected, into the open block or the root.
    fn flush(&mut self) {
        if self.unit.is_empty() {
            return;
        }
        let item = Item::Group(Group {
            items: std::mem::take(&mut self.unit),
        });
        if self.open > 0 {
            self.block.push(item);
        } else {
            self.root.push(item);
        }
    }
}
