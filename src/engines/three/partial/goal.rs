//! A target set: the states that count as reached.
//!
//! A goal is a list of views; a state meets the goal when it meets any view.
//! A view says, per corner slot and per edge slot it constrains, which
//! (piece, orientation) pairs may sit there: one bit each, 24 per slot. The
//! stickers of a view are named by identity (the home position of a sticker),
//! so a color target is the case where a position accepts every sticker of
//! its colors.

use super::super::cubie::{CORNER_FACELET, EDGE_FACELET};
use super::model::Pos;

/// Bit `i` is the sticker whose home position is facelet `i`.
pub(super) type Masks = [u64; 54];

pub(super) const EVERY_STICKER: u64 = (1 << 54) - 1;

/// What one view constrains: per slot, the accepted (piece, orientation)
/// pairs as bits `piece * 3 + twist` (corners) or `piece * 2 + flip` (edges).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct View {
    pub(super) corners: Vec<(u8, u32)>,
    pub(super) edges: Vec<(u8, u32)>,
}

#[derive(Debug, Default)]
pub(super) struct Goal {
    pub(super) views: Vec<View>,
}

impl Goal {
    /// The goal that accepts a state meeting any of `views`. A view whose
    /// center asks for a sticker other than the center there, or that no
    /// piece can meet, is dropped.
    pub(super) fn new(views: &[Masks]) -> Goal {
        let mut goal = Goal::default();
        for masks in views {
            if let Some(view) = view_of(masks) {
                if !goal.views.contains(&view) {
                    goal.views.push(view);
                }
            }
        }
        goal
    }

    /// One view per rotation: at position `p` the view of rotation `r` takes
    /// the stickers `masks` accepts at `r[p]` (the cube is turned, then looked
    /// at); `rotations` lists the position maps.
    pub(super) fn rotated(masks: &Masks, rotations: &[Vec<usize>]) -> Goal {
        let views: Vec<Masks> = rotations
            .iter()
            .map(|map| std::array::from_fn(|p| masks[map[p]]))
            .collect();
        Goal::new(&views)
    }

    /// Whether no state can meet the goal.
    pub(super) fn is_impossible(&self) -> bool {
        self.views.is_empty()
    }

    /// Whether `pos` meets some view.
    #[inline]
    pub(super) fn is_met(&self, pos: &Pos) -> bool {
        self.views.iter().any(|view| {
            view.corners.iter().all(|&(slot, bits)| {
                let s = usize::from(slot);
                bits >> (usize::from(pos.cp[s]) * 3 + usize::from(pos.co[s])) & 1 == 1
            }) && view.edges.iter().all(|&(slot, bits)| {
                let s = usize::from(slot);
                bits >> (usize::from(pos.ep[s]) * 2 + usize::from(pos.eo[s])) & 1 == 1
            })
        })
    }
}

/// The accepted pairs per slot of one view, or `None` when it cannot be met.
fn view_of(masks: &Masks) -> Option<View> {
    // The centers never move: the sticker on center `f` is the center `f`.
    for face in 0..6 {
        let center = face * 9 + 4;
        if masks[center] >> center & 1 == 0 {
            return None;
        }
    }
    let mut view = View {
        corners: Vec::new(),
        edges: Vec::new(),
    };
    for (slot, home) in CORNER_FACELET.iter().enumerate() {
        if home.iter().all(|&f| masks[usize::from(f)] == EVERY_STICKER) {
            continue;
        }
        let mut bits = 0u32;
        for (piece, pieces) in CORNER_FACELET.iter().enumerate() {
            for twist in 0..3 {
                let fits = (0..3).all(|k| {
                    let shown = pieces[(k + 3 - twist) % 3];
                    masks[usize::from(home[k])] >> shown & 1 == 1
                });
                if fits {
                    bits |= 1 << (piece * 3 + twist);
                }
            }
        }
        if bits == 0 {
            return None;
        }
        view.corners.push((slot as u8, bits));
    }
    for (slot, home) in EDGE_FACELET.iter().enumerate() {
        if home.iter().all(|&f| masks[usize::from(f)] == EVERY_STICKER) {
            continue;
        }
        let mut bits = 0u32;
        for (piece, pieces) in EDGE_FACELET.iter().enumerate() {
            for flip in 0..2 {
                let fits = (0..2).all(|k| {
                    let shown = pieces[(k + flip) % 2];
                    masks[usize::from(home[k])] >> shown & 1 == 1
                });
                if fits {
                    bits |= 1 << (piece * 2 + flip);
                }
            }
        }
        if bits == 0 {
            return None;
        }
        view.edges.push((slot as u8, bits));
    }
    Some(view)
}

/// The masks of a color target: a position accepts every sticker whose home
/// face color is in its set.
pub(super) fn color_masks(sets: &[u8]) -> Masks {
    std::array::from_fn(|p| {
        (0..54u32)
            .filter(|&j| sets[p] >> (j / 9) & 1 == 1)
            .fold(0u64, |acc, j| acc | 1 << j)
    })
}
