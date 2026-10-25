//! The face centers of an odd cube.
//!
//! On a 3x3x3, 5x5x5 or 7x7x7 the center of each face is one piece that no
//! turn of an outer layer moves. A middle-slice turn moves them, as a whole
//! cube rotation does; they then sit on other faces. Such a cube is reachable.

use super::{Face, Facelets, Failure, MoveSequence};

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

/// The color on the center of every face, as face numbers, in face order.
fn center_colors(cube: &Facelets) -> [usize; 6] {
    let area = cube.kind().face_len();
    let stickers = cube.stickers();
    Face::ALL.map(|face| stickers[face.index() * area + area / 2].index())
}

/// What an engine that needs the face centers on their own faces answers for
/// a cube whose centers are not: a cube no turns reach (two centers swapped,
/// say) is [`Failure::InvalidInput`] with the reason; a reachable one, whose
/// centers a slice turn moved, is [`Failure::CentersOffFace`].
pub(crate) fn centers_off_face(cube: &Facelets) -> (Failure, String) {
    if rotations().contains(&center_colors(cube)) {
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

/// The cube with its colors renamed so that every face center carries the
/// color of the face it sits on, when a 3x3x3, 5x5x5 or 7x7x7 has its centers
/// off their faces in a way a turn can cause.
///
/// The color of a face is the color of its center, the one piece no outer
/// turn moves. The renamed cube is a cube the engines take, and any moves that
/// solve it solve the given cube too: moves permute positions, a renaming of
/// colors does not care where they sit. Afterwards every face of the given
/// cube shows one color, the color of its center, not the color that face
/// showed before; [`solves_by_color`] is the test for that.
///
/// `None` for an even cube (it has no fixed centers, its engines find the
/// frame themselves), for a cube whose centers are in place, and for centers
/// that are no rotation of the cube.
pub fn oriented_by_centers(cube: &Facelets) -> Option<Facelets> {
    if cube.kind().size().is_multiple_of(2) {
        return None;
    }
    let seen = center_colors(cube);
    if seen == [0, 1, 2, 3, 4, 5] || !rotations().contains(&seen) {
        return None;
    }
    // The color on the center of face `face` becomes `face`.
    let mut renamed = [Face::U; 6];
    for (face, color) in seen.into_iter().enumerate() {
        renamed[color] = Face::ALL[face];
    }
    let stickers = cube
        .stickers()
        .iter()
        .map(|color| renamed[color.index()])
        .collect();
    Facelets::from_stickers(stickers).ok()
}

/// True when `moves` leave every face of `start` in one color, whichever
/// color. [`Facelets::is_solved`] wants each face in its own color; this is
/// the test for a cube whose centers are off their faces.
pub fn solves_by_color(start: &Facelets, moves: &MoveSequence) -> bool {
    let Ok(end) = start.apply(moves) else {
        return false;
    };
    end.stickers()
        .chunks(start.kind().face_len())
        .all(|face| face.iter().all(|&color| color == face[0]))
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
        assert_eq!(oriented_by_centers(&swapped), None);
    }

    fn turned(kind: PuzzleKind, moves: &str) -> Facelets {
        let moves: MoveSequence = moves.parse().unwrap();
        Facelets::solved(kind).apply(&moves).unwrap()
    }

    #[test]
    fn a_cube_with_its_centers_in_place_is_left_alone() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            assert_eq!(oriented_by_centers(&turned(kind, "R U F'")), None);
        }
        assert_eq!(
            oriented_by_centers(&turned(PuzzleKind::Cube4, "R Uw")),
            None
        );
    }

    #[test]
    fn a_slice_turned_cube_is_renamed_after_its_centers() {
        // (size, scramble, turns that leave every face one color)
        for (kind, moves, solution) in [
            (PuzzleKind::Cube3, "2R", "R L'"),
            (PuzzleKind::Cube5, "3R", "Lw' Rw"),
            (PuzzleKind::Cube7, "4R", ""),
        ] {
            let cube = turned(kind, moves);
            assert!(!cube.is_solved(), "{kind:?}");
            let renamed = oriented_by_centers(&cube).expect("renamed");
            assert_eq!(center_colors(&renamed), [0, 1, 2, 3, 4, 5], "{kind:?}");
            assert_eq!(oriented_by_centers(&renamed), None, "{kind:?}");
            if kind != PuzzleKind::Cube7 {
                let solution: MoveSequence = solution.parse().unwrap();
                assert!(solves_by_color(&cube, &solution), "{kind:?}");
            }
        }
    }
}
