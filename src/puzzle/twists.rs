//! The turn of the six fixed centers of an odd cube, for drawing.

use std::fmt;

use super::twist_track::{self, Tracked, Tracker};
use super::{Face, MoveSequence, PuzzleKind};

/// The version of the twist convention, printed in certificates. In the solved
/// orientation the top of the picture of U points to the back, the top of D to
/// the front and the top of the four side pictures up. A turn `f` of a face adds
/// 1 to the twist of its center, `f2` adds 2, `f'` adds 3.
pub const TWIST_CONVENTION: &str = "center-twist-1 (t=URFDLB, clockwise quarters seen from outside, up: U back, D front, sides up)";

/// Why a twist text or a replay was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwistError {
    /// The text is no list of six twists.
    Syntax(String),
    /// The move or cube is outside what the tracker supports.
    Unsupported(String),
}

impl fmt::Display for TwistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwistError::Syntax(text) | TwistError::Unsupported(text) => f.write_str(text),
        }
    }
}

impl std::error::Error for TwistError {}

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

    /// Whether the sum of the six twists is even. On a cube it equals the parity of
    /// the corner permutation.
    pub fn sum_is_even(&self) -> bool {
        self.twists.iter().map(|&t| u32::from(t)).sum::<u32>() % 2 == 0
    }

    /// Reads six numbers 0 to 3 separated by commas, spaces ignored: `0,1,2,0,3,0`.
    pub fn parse(kind: PuzzleKind, text: &str) -> Result<CenterTwists, TwistError> {
        let bad = || {
            TwistError::Syntax(format!(
                "a center twist is six numbers 0 to 3 for U,R,F,D,L,B separated by commas, like 0,1,2,0,3,0 (not {text:?})"
            ))
        };
        let parts: Vec<&str> = text.split(',').map(str::trim).collect();
        if parts.len() != 6 {
            return Err(bad());
        }
        let mut twists = [0u8; 6];
        for (slot, part) in twists.iter_mut().zip(&parts) {
            *slot = match *part {
                "0" => 0,
                "1" => 1,
                "2" => 2,
                "3" => 3,
                _ => return Err(bad()),
            };
        }
        CenterTwists::new(kind, twists).map_err(|e| TwistError::Unsupported(e.to_string()))
    }

    /// The twists seen on each face after the whole-cube rotations of `word`
    /// (`x y z` turns separated by spaces): each picture goes with its center.
    pub fn rotated(&self, word: &str) -> Result<CenterTwists, TwistError> {
        let mut tracker = Tracker::from_twists(self)?;
        tracker.rotate_word(word)?;
        Ok(tracker.read())
    }

    /// What `moves` do to the twists of a 3x3x3. The result is read in the frame
    /// where every center is on its own face; the whole-cube rotation that gets
    /// there is in `Tracked::frame` and costs no move.
    pub fn track(&self, moves: &MoveSequence) -> Result<Tracked, TwistError> {
        twist_track::track(self, moves)
    }
}

impl fmt::Display for CenterTwists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [u, r, ff, d, l, b] = self.twists;
        write!(f, "{u},{r},{ff},{d},{l},{b}")
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
