//! A target set: the states that count as reached.
//!
//! A goal is a list of views; a state meets the goal when it meets any view.
//! A view says, per corner slot and per edge slot it constrains, which
//! (piece, orientation) pairs may sit there: one bit each, 24 per slot. The
//! stickers of a view are named by identity (the home position of a sticker),
//! so a color target is the case where a position accepts every sticker of
//! its colors.

use super::super::cubie::{CORNER_FACELET, EDGE_FACELET};
use super::model::{twist_of, Pos};
use crate::puzzle::CenterRotation;
use crate::puzzle::{TwistMasks, FREE_TWIST};

/// Bit `i` is the sticker whose home position is facelet `i`.
pub(super) type Masks = [u64; 54];

pub(super) const EVERY_STICKER: u64 = (1 << 54) - 1;

/// What one view constrains: per slot, the accepted (piece, orientation)
/// pairs as bits `piece * 3 + twist` (corners) or `piece * 2 + flip` (edges).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct View {
    pub(super) corners: Vec<(u8, u32)>,
    pub(super) edges: Vec<(u8, u32)>,
    /// The twists each face center may show; free unless the target says.
    pub(super) twist: TwistMasks,
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
        let pairs: Vec<(Masks, TwistMasks)> = views.iter().map(|m| (*m, FREE_TWIST)).collect();
        Goal::with_twists(&pairs)
    }

    /// Like [`Goal::new`], each view with its condition on the center twists.
    pub(super) fn with_twists(views: &[(Masks, TwistMasks)]) -> Goal {
        let mut goal = Goal::default();
        for (masks, twist) in views {
            if let Some(mut view) = view_of(masks) {
                view.twist = *twist;
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

    /// Like [`Goal::rotated`] with a condition on the center twists. Under
    /// rotation `r` the center of face `g` sits on face `dest[g]` with its
    /// picture turned `add[g]` quarters more, so it may show the twists `k`
    /// for which `k + add[g]` is accepted on face `dest[g]`.
    pub(super) fn rotated_twisted(
        masks: &Masks,
        twist: &TwistMasks,
        rotations: &[(Vec<usize>, CenterRotation)],
    ) -> Goal {
        let views: Vec<(Masks, TwistMasks)> = rotations
            .iter()
            .map(|(map, center)| {
                let shown: Masks = std::array::from_fn(|p| masks[map[p]]);
                let accepted: TwistMasks = std::array::from_fn(|g| {
                    let want = twist[center.dest[g]];
                    (0..4u8)
                        .filter(|&k| want >> ((k + center.add[g]) & 3) & 1 == 1)
                        .fold(0, |acc, k| acc | 1 << k)
                });
                (shown, accepted)
            })
            .collect();
        Goal::with_twists(&views)
    }

    /// Whether any view conditions the center twists.
    pub(super) fn has_twist_condition(&self) -> bool {
        self.views.iter().any(|v| v.twist != FREE_TWIST)
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
            }) && twist_ok(&view.twist, pos.tw)
        })
    }
}

/// Whether the packed twists `tw` are accepted by `masks`.
#[inline]
pub(super) fn twist_ok(masks: &TwistMasks, tw: u16) -> bool {
    (0..6).all(|g| masks[g] >> twist_of(tw, g) & 1 == 1)
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
        twist: FREE_TWIST,
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
