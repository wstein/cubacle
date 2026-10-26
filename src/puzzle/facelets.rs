use std::fmt;
use std::str::FromStr;

use super::PuzzleKind;

/// A face of the cube, in the `URFDLB` order used by the facelet string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    U,
    R,
    F,
    D,
    L,
    B,
}

impl Face {
    pub const ALL: [Face; 6] = [Face::U, Face::R, Face::F, Face::D, Face::L, Face::B];

    pub fn letter(self) -> char {
        match self {
            Face::U => 'U',
            Face::R => 'R',
            Face::F => 'F',
            Face::D => 'D',
            Face::L => 'L',
            Face::B => 'B',
        }
    }

    pub fn from_letter(letter: char) -> Option<Face> {
        Face::ALL.into_iter().find(|face| face.letter() == letter)
    }

    pub(crate) fn index(self) -> usize {
        self as usize
    }
}

/// Why a facelet string was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaceletsError {
    /// The length is not `6 * n * n` for any supported size.
    BadLength(usize),
    /// A character is not one of `U R F D L B`.
    BadSticker { position: usize, found: char },
    /// A color does not appear exactly `n * n` times.
    BadStickerCount {
        face: Face,
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for FaceletsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FaceletsError::BadLength(len) => {
                write!(f, "{len} stickers is not 6*n*n for a supported cube size")
            }
            FaceletsError::BadSticker { position, found } => {
                write!(f, "sticker {position} is {found:?}, expected one of URFDLB")
            }
            FaceletsError::BadStickerCount {
                face,
                expected,
                found,
            } => write!(
                f,
                "color {} appears {found} times, expected {expected}",
                face.letter()
            ),
        }
    }
}

impl std::error::Error for FaceletsError {}

/// A cube state given as its sticker colors.
///
/// Stickers are ordered face by face in `URFDLB` order, and row-major within
/// each face. A sticker's value is the [`Face`] whose color it shows, so the
/// solved state has every face showing its own letter. The string has
/// `6 * n * n` characters: 24, 54, 96, 150, 216 or 294.
///
/// Construction checks only that the string is well-formed (length and
/// characters) and that each color occurs `n * n` times. Whether the state is
/// reachable is the engine's concern, and is reported as invalid input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Facelets {
    kind: PuzzleKind,
    stickers: Vec<Face>,
}

impl Facelets {
    pub fn parse(text: &str) -> Result<Facelets, FaceletsError> {
        let kind = PuzzleKind::of_facelet_count(text.chars().count())
            .ok_or_else(|| FaceletsError::BadLength(text.chars().count()))?;
        let mut stickers = Vec::with_capacity(kind.facelet_count());
        for (position, found) in text.chars().enumerate() {
            let face =
                Face::from_letter(found).ok_or(FaceletsError::BadSticker { position, found })?;
            stickers.push(face);
        }
        let facelets = Facelets { kind, stickers };
        facelets.validate_sticker_counts()?;
        Ok(facelets)
    }

    /// Builds facelets from stickers already in `URFDLB` order.
    pub fn from_stickers(stickers: Vec<Face>) -> Result<Facelets, FaceletsError> {
        let kind = PuzzleKind::of_facelet_count(stickers.len())
            .ok_or(FaceletsError::BadLength(stickers.len()))?;
        let facelets = Facelets { kind, stickers };
        facelets.validate_sticker_counts()?;
        Ok(facelets)
    }

    pub fn solved(kind: PuzzleKind) -> Facelets {
        let stickers = Face::ALL
            .into_iter()
            .flat_map(|face| std::iter::repeat_n(face, kind.face_len()))
            .collect();
        Facelets { kind, stickers }
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    pub fn stickers(&self) -> &[Face] {
        &self.stickers
    }

    /// Checks that every color occurs exactly `n * n` times.
    pub fn validate_sticker_counts(&self) -> Result<(), FaceletsError> {
        let mut counts = [0usize; 6];
        for face in &self.stickers {
            counts[face.index()] += 1;
        }
        let expected = self.kind.face_len();
        for face in Face::ALL {
            if counts[face.index()] != expected {
                return Err(FaceletsError::BadStickerCount {
                    face,
                    expected,
                    found: counts[face.index()],
                });
            }
        }
        Ok(())
    }

    pub fn is_solved(&self) -> bool {
        *self == Facelets::solved(self.kind)
    }
}

impl fmt::Display for Facelets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.stickers
            .iter()
            .try_for_each(|face| write!(f, "{}", face.letter()))
    }
}

impl FromStr for Facelets {
    type Err = FaceletsError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Facelets::parse(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLVED_3X3: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

    #[test]
    fn solved_matches_the_known_3x3_string() {
        assert_eq!(Facelets::solved(PuzzleKind::Cube3).to_string(), SOLVED_3X3);
    }

    #[test]
    fn every_size_round_trips() {
        for kind in PuzzleKind::ALL {
            let solved = Facelets::solved(kind);
            assert_eq!(solved.stickers().len(), kind.facelet_count());
            assert_eq!(Facelets::parse(&solved.to_string()), Ok(solved.clone()));
            assert!(solved.is_solved());
        }
    }

    #[test]
    fn rejects_bad_length() {
        assert_eq!(Facelets::parse("UUU"), Err(FaceletsError::BadLength(3)));
    }

    #[test]
    fn rejects_unknown_sticker() {
        let text = SOLVED_3X3.replacen('U', "X", 1);
        assert_eq!(
            Facelets::parse(&text),
            Err(FaceletsError::BadSticker {
                position: 0,
                found: 'X'
            })
        );
    }

    #[test]
    fn rejects_wrong_color_counts() {
        let text = SOLVED_3X3.replacen('U', "R", 1);
        assert_eq!(
            Facelets::parse(&text),
            Err(FaceletsError::BadStickerCount {
                face: Face::U,
                expected: 9,
                found: 8
            })
        );
    }

    #[test]
    fn a_scrambled_state_is_not_solved() {
        let mut stickers: Vec<char> = SOLVED_3X3.chars().collect();
        stickers.swap(0, 9);
        let text: String = stickers.into_iter().collect();
        assert!(!Facelets::parse(&text).unwrap().is_solved());
    }
}
