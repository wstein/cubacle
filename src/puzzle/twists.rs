//! The turn of the six fixed centers of an odd cube, for drawing.

use std::fmt;

use super::{Face, PuzzleKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untwisted_odd_cube_has_no_twist_on_any_face() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let twists = CenterTwists::untwisted(kind).unwrap();
            assert_eq!(twists.kind(), kind);
            assert!(twists.is_untwisted());
            assert!(Face::ALL.into_iter().all(|face| twists.twist(face) == 0));
        }
    }

    #[test]
    fn twists_are_kept_per_face_in_urfdlb_order() {
        let twists = CenterTwists::new(PuzzleKind::Cube3, [1, 0, 3, 2, 0, 1]).unwrap();
        let seen: Vec<u8> = Face::ALL.into_iter().map(|f| twists.twist(f)).collect();
        assert_eq!(seen, [1, 0, 3, 2, 0, 1]);
        assert!(!twists.is_untwisted());
        assert_eq!(twists.twist(Face::F), 3);
    }

    #[test]
    fn even_cubes_have_no_fixed_centers_and_a_twist_is_below_four() {
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube4, PuzzleKind::Cube6] {
            assert_eq!(CenterTwists::untwisted(kind), None);
            assert_eq!(
                CenterTwists::new(kind, [0; 6]),
                Err(CenterTwistsError::EvenSize(kind))
            );
        }
        assert_eq!(
            CenterTwists::new(PuzzleKind::Cube3, [0, 0, 0, 4, 0, 0]),
            Err(CenterTwistsError::OutOfRange {
                face: Face::D,
                found: 4
            })
        );
        let message = CenterTwistsError::EvenSize(PuzzleKind::Cube4).to_string();
        assert!(message.contains("4x4x4"), "{message}");
    }
}
