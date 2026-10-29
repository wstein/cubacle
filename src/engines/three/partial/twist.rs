//! The center twists as a search coordinate: 4096 states, one table.
//!
//! The twists of the six centers (Z4 each) change by the face of the move only,
//! so the coordinate is the packed vector itself and its move table is the
//! arithmetic of `model::twist_step`. The table is a breadth-first search from
//! every vector some view accepts. It ignores the pieces, so it is an
//! admissible bound; it is worth the most when only a few centers need turning.

use super::goal::{twist_ok, Goal};
use super::model::{twist_step, TURNS};

const STATES: usize = 1 << 12;

pub(super) struct TwistTable {
    dist: Vec<u8>,
}

impl TwistTable {
    /// `None` when no view conditions a twist (every vector is a seed).
    pub(super) fn build(goal: &Goal) -> Option<TwistTable> {
        if !goal.has_twist_condition() {
            return None;
        }
        let mut dist = vec![u8::MAX; STATES];
        let mut frontier: Vec<u16> = (0..STATES as u16)
            .filter(|&tw| goal.views.iter().any(|v| twist_ok(&v.twist, tw)))
            .collect();
        for &tw in &frontier {
            dist[usize::from(tw)] = 0;
        }
        let mut level = 0u8;
        while !frontier.is_empty() {
            level += 1;
            let mut next = Vec::new();
            for &tw in &frontier {
                for m in 0..TURNS {
                    let to = twist_step(tw, m);
                    if dist[usize::from(to)] == u8::MAX {
                        dist[usize::from(to)] = level;
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        Some(TwistTable { dist })
    }

    #[inline]
    pub(super) fn bound(&self, tw: u16) -> u8 {
        self.dist[usize::from(tw)]
    }

    /// The distances as bytes, for the digest in a certificate.
    pub(super) fn bytes(&self) -> &[u8] {
        &self.dist
    }
}

#[cfg(test)]
mod tests {
    use super::super::goal::color_masks;
    use super::super::model::pack;
    use super::*;
    use crate::puzzle::{parse_twist_masks, Facelets, PuzzleKind, Target};

    #[test]
    fn the_distance_counts_the_faces_that_need_a_turn_at_least() {
        let target = Target::complete(&Facelets::solved(PuzzleKind::Cube3));
        let masks = color_masks(target.sets());
        let upright = parse_twist_masks("0,0,0,0,0,0").unwrap();
        let goal = Goal::with_twists(&[(masks, upright)]);
        let table = TwistTable::build(&goal).expect("a condition");
        assert_eq!(table.bound(pack([0; 6])), 0);
        // one face off by any amount is one move, three faces are three
        assert_eq!(table.bound(pack([1, 0, 0, 0, 0, 0])), 1);
        assert_eq!(table.bound(pack([0, 2, 0, 0, 0, 0])), 1);
        assert_eq!(table.bound(pack([3, 0, 1, 0, 2, 0])), 3);
        // every vector is reachable, so no entry stays unseen
        assert!(table.bytes().iter().all(|&d| d <= 6));
    }
}
