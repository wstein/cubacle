//! The 24 whole-cube rotations as sticker maps, and what they mean for a target.
//!
//! A rotation moves every sticker and changes no color. A state meets a target
//! "in any orientation" when some rotation of it meets the target. The maps
//! are built from the layer turns of one axis (all layers together), so they
//! follow the sticker layout of [`super::Facelets`] for every cube size.

use super::apply::destinations;
use super::{Face, Facelets, Move, MoveSequence, PuzzleKind, Target, Turn};

/// The 24 rotations of a cube of `kind`: entry `r` sends sticker `i` to
/// position `r[i]`. The identity comes first.
pub(crate) fn rotation_maps(kind: PuzzleKind) -> Vec<Vec<usize>> {
    let count = kind.facelet_count();
    let depth_count = u8::try_from(kind.size()).expect("cube sizes are small");
    let generators: Vec<Vec<usize>> = [Face::R, Face::U, Face::F]
        .into_iter()
        .map(|face| {
            let mut map: Vec<usize> = (0..count).collect();
            for depth in 1..=depth_count {
                let step = destinations(kind, &Move::new(face, depth, false, Turn::Clockwise));
                map = map.iter().map(|&at| step[at]).collect();
            }
            map
        })
        .collect();
    let mut seen: Vec<Vec<usize>> = vec![(0..count).collect()];
    let mut at = 0;
    while at < seen.len() {
        for generator in &generators {
            let next: Vec<usize> = seen[at].iter().map(|&p| generator[p]).collect();
            if !seen.contains(&next) {
                seen.push(next);
            }
        }
        at += 1;
    }
    seen
}

impl Target {
    /// Whether some whole-cube rotation of `state` meets this target.
    pub fn matches_any_rotation(&self, state: &Facelets) -> bool {
        if state.kind() != self.kind() {
            return false;
        }
        let stickers = state.stickers();
        rotation_maps(self.kind()).iter().any(|map| {
            let mut rotated = stickers.to_vec();
            for (from, &to) in map.iter().enumerate() {
                rotated[to] = stickers[from];
            }
            self.sets()
                .iter()
                .zip(&rotated)
                .all(|(set, face)| set >> face.index() & 1 == 1)
        })
    }

    /// Whether `moves` take `start` to a state that meets this target after
    /// some whole-cube rotation (the rotation costs no moves).
    pub fn reached_by_any_rotation(&self, start: &Facelets, moves: &MoveSequence) -> bool {
        start
            .apply(moves)
            .is_ok_and(|end| self.matches_any_rotation(&end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cube_has_24_distinct_rotations() {
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3, PuzzleKind::Cube4] {
            let maps = rotation_maps(kind);
            assert_eq!(maps.len(), 24, "{kind:?}");
            assert!(maps[0].iter().enumerate().all(|(i, &p)| i == p));
        }
    }

    #[test]
    fn a_face_target_is_met_in_the_orientation_that_shows_the_face() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        // the whole F face, centers included
        let target = Target::masked(&solved, |p| (18..27).contains(&p));
        assert!(target.matches_any_rotation(&solved));
        // a cube with one F sticker swapped for a U sticker meets it in no orientation
        let mut stickers = solved.stickers().to_vec();
        stickers.swap(8, 20);
        let broken = Facelets::from_stickers(stickers).unwrap();
        let whole = Target::complete(&solved);
        assert!(!whole.matches_any_rotation(&broken));
        // a rotated solved cube (the colors stay, the positions move) meets the
        // complete target only in the orientation that undoes the rotation
        let rotated = {
            let map = &rotation_maps(PuzzleKind::Cube3)[5];
            let mut out = solved.stickers().to_vec();
            for (from, &to) in map.iter().enumerate() {
                out[to] = solved.stickers()[from];
            }
            Facelets::from_stickers(out).unwrap()
        };
        assert!(!whole.matches(&rotated) || rotated == solved);
        assert!(whole.matches_any_rotation(&rotated));
    }
}
