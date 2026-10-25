//! The face centers of an odd cube.
//!
//! On a 3x3x3, 5x5x5 or 7x7x7 the center of each face is one piece that no
//! turn of an outer layer moves. A middle-slice turn moves them, as a whole
//! cube rotation does; they then sit on other faces. Such a cube is reachable.

use super::{Face, Facelets, Failure};

/// The 24 rotations of a cube, each as the face every face goes to.
fn rotations() -> Vec<[usize; 6]> {
    // Faces are in the order U R F D L B. `x` cycles U, F, D, B; `y` cycles
    // F, R, B, L.
    let x = [2, 1, 3, 5, 4, 0];
    let y = [0, 5, 1, 3, 2, 4];
    let mut found = vec![[0, 1, 2, 3, 4, 5]];
    let mut next = 0;
    while next < found.len() {
        let turn = found[next];
        next += 1;
        for generator in [x, y] {
            let product = turn.map(|face| generator[face]);
            if !found.contains(&product) {
                found.push(product);
            }
        }
    }
    found
}

/// What an engine that needs the face centers on their own faces answers for
/// a cube whose centers are not: a cube no turns reach (two centers swapped,
/// say) is [`Failure::InvalidInput`] with the reason; a reachable one, whose
/// centers a slice turn moved, is [`Failure::CentersOffFace`].
pub(crate) fn centers_off_face(cube: &Facelets) -> (Failure, String) {
    let area = cube.kind().face_len();
    let stickers = cube.stickers();
    let seen = Face::ALL.map(|face| stickers[face.index() * area + area / 2].index());
    if rotations().contains(&seen) {
        (
            Failure::CentersOffFace,
            "the face centers are not on their own faces (a middle-slice turn moved them)"
                .to_owned(),
        )
    } else {
        (
            Failure::InvalidInput,
            "the centers are invalid: the six fixed centers are not a rotation of the cube"
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{MoveSequence, PuzzleKind};

    #[test]
    fn there_are_24_rotations() {
        assert_eq!(rotations().len(), 24);
    }

    #[test]
    fn a_slice_turn_is_reachable_and_swapped_centers_are_not() {
        let slice: MoveSequence = "2R".parse().unwrap();
        let moved = Facelets::solved(PuzzleKind::Cube3).apply(&slice).unwrap();
        assert_eq!(centers_off_face(&moved).0, Failure::CentersOffFace);
        let mut stickers = Facelets::solved(PuzzleKind::Cube3).stickers().to_vec();
        stickers.swap(4, 9 + 4);
        let swapped = Facelets::from_stickers(stickers).unwrap();
        assert_eq!(centers_off_face(&swapped).0, Failure::InvalidInput);
    }
}
