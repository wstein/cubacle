//! The turn of the six fixed centers of an odd cube, for drawing.

use std::fmt;

use super::{Face, PuzzleKind};

/// How far each of the six fixed centers of an odd cube is turned, in quarter
/// turns clockwise (`0..=3`), indexed by the face in `URFDLB` order.
///
/// This is the orientation a picture sticker on a center shows; [`Facelets`]
/// holds colors only and does not carry it. Notation code fills the value, the
/// renderer reads it.
///
/// [`Facelets`]: super::Facelets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CenterTwists {
    kind: PuzzleKind,
    twists: [u8; 6],
}

/// Why no [`CenterTwists`] exists for the given input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterTwistsError {
    /// Even cubes have no fixed centers.
    EvenSize(PuzzleKind),
    /// A twist is not a number of quarter turns below four.
    OutOfRange { face: Face, found: u8 },
}

impl fmt::Display for CenterTwistsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CenterTwistsError::EvenSize(kind) => {
                let n = kind.size();
                write!(f, "the {n}x{n}x{n} has no fixed centers")
            }
            CenterTwistsError::OutOfRange { face, found } => write!(
                f,
                "center twist {found} on {} is not in 0..=3",
                face.letter()
            ),
        }
    }
}

impl std::error::Error for CenterTwistsError {}

impl CenterTwists {
    /// The twists of `kind`, one per face in `URFDLB` order.
    pub fn new(kind: PuzzleKind, twists: [u8; 6]) -> Result<CenterTwists, CenterTwistsError> {
        if kind.size().is_multiple_of(2) {
            return Err(CenterTwistsError::EvenSize(kind));
        }
        if let Some((face, &found)) = Face::ALL
            .into_iter()
            .zip(&twists)
            .find(|&(_, &twist)| twist > 3)
        {
            return Err(CenterTwistsError::OutOfRange { face, found });
        }
        Ok(CenterTwists { kind, twists })
    }

    /// No center turned; `None` for an even cube.
    pub fn untwisted(kind: PuzzleKind) -> Option<CenterTwists> {
        CenterTwists::new(kind, [0; 6]).ok()
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// Quarter turns of the center of `face`, `0..=3`.
    pub fn twist(&self, face: Face) -> u8 {
        self.twists[face.index()]
    }

    pub fn is_untwisted(&self) -> bool {
        self.twists == [0; 6]
    }
}

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
