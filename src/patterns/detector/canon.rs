//! The canonical form of a state: the smallest sticker string in its orbit
//! under the 48 symmetries and every renaming of the six colors.
//!
//! Renaming the colors so that they first appear in the order `U R F D L B`
//! gives the smallest string of any renaming, so the orbit minimum is the
//! least of the 48 relabelled images.

use super::geometry::{image, symmetries};
use crate::puzzle::{Face, Facelets, PuzzleKind};

/// `state` with its colors renamed in order of first appearance.
pub(crate) fn relabel(state: &[u8]) -> Vec<u8> {
    let mut name = [u8::MAX; 6];
    let mut next = 0;
    state
        .iter()
        .map(|&c| {
            if name[c as usize] == u8::MAX {
                name[c as usize] = next;
                next += 1;
            }
            name[c as usize]
        })
        .collect()
}

/// The canonical form of a state.
pub(crate) fn canonical(kind: PuzzleKind, state: &[u8]) -> Vec<u8> {
    (0..symmetries().len())
        .map(|g| relabel(&image(kind, state, g)))
        .min()
        .expect("48 symmetries")
}

/// The canonical form of a cube, as a cube of the same size (not necessarily
/// a reachable one: the colors are named by first appearance).
pub(crate) fn canonical_cube(cube: &Facelets) -> Facelets {
    facelets(&canonical(cube.kind(), &super::colors(cube)))
}

#[cfg(test)]
/// A cube mirrored across the plane between its R and L faces: the stickers
/// move, their colors stay.
pub(crate) fn mirrored(cube: &Facelets) -> Facelets {
    let kind = cube.kind();
    let g = symmetries()
        .iter()
        .position(|s| {
            s.apply([1, 0, 0]) == [-1, 0, 0]
                && s.apply([0, 1, 0]) == [0, 1, 0]
                && s.apply([0, 0, 1]) == [0, 0, 1]
        })
        .expect("the mirror across the R/L plane is a symmetry");
    facelets(&image(kind, &super::colors(cube), g))
}

/// A state back as a cube.
pub(crate) fn facelets(state: &[u8]) -> Facelets {
    let stickers = state.iter().map(|&c| Face::ALL[c as usize]).collect();
    Facelets::from_stickers(stickers).expect("a relabelling keeps every color count")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::detector::colors;

    #[test]
    fn the_canonical_form_is_idempotent_and_ignores_turns_and_colors() {
        let kind = PuzzleKind::Cube3;
        let cube = Facelets::solved(kind)
            .apply(&"R U F' L2 D".parse().unwrap())
            .unwrap();
        let state = colors(&cube);
        let canon = canonical(kind, &state);
        assert_eq!(canonical(kind, &canon), canon);
        for g in 0..48 {
            assert_eq!(canonical(kind, &image(kind, &state, g)), canon);
        }
        let swapped: Vec<u8> = state.iter().map(|&c| (c + 2) % 6).collect();
        assert_eq!(canonical(kind, &swapped), canon);
    }
}
