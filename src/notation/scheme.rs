use std::fmt;
use std::str::FromStr;

use crate::puzzle::{Face, Facelets};

/// Which sticker colour sits on which face, in `URFDLB` order.
///
/// A cube given as colours rather than faces needs this to be read. Odd cubes
/// carry the answer in their fixed centres, see [`ColourScheme::from_centres`];
/// even cubes and the 2x2x2 have no centres and need a scheme named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColourScheme {
    letters: [char; 6],
}

impl ColourScheme {
    /// White up, green front, red right, yellow down, orange left, blue back.
    /// Opposite pairs: white-yellow, green-blue, red-orange.
    pub const WESTERN_BOY: ColourScheme = ColourScheme {
        letters: ['W', 'R', 'G', 'Y', 'O', 'B'],
    };

    /// White up, green front, red right, blue down, orange left, yellow back.
    /// Opposite pairs: white-blue, green-yellow, red-orange.
    pub const JAPANESE_BL_WT: ColourScheme = ColourScheme {
        letters: ['W', 'R', 'G', 'B', 'O', 'Y'],
    };

    /// The colour letters, written in `URFDLB` order. Rejects a letter used twice.
    pub fn new(letters: [char; 6]) -> Result<ColourScheme, SchemeError> {
        let letters = letters.map(|letter| letter.to_ascii_uppercase());
        for (index, letter) in letters.iter().enumerate() {
            if letters[..index].contains(letter) {
                return Err(SchemeError::Duplicate(*letter));
            }
        }
        Ok(ColourScheme { letters })
    }

    pub fn letters(&self) -> [char; 6] {
        self.letters
    }

    /// The face a colour letter belongs to, or `None` if the scheme lacks it.
    pub fn face_of(&self, letter: char) -> Option<Face> {
        let letter = letter.to_ascii_uppercase();
        self.letters
            .iter()
            .position(|&known| known == letter)
            .map(|index| Face::ALL[index])
    }

    pub fn letter_of(&self, face: Face) -> char {
        self.letters[face.index()]
    }

    /// The cube as colour letters, one per sticker in facelet order (the order
    /// of `Facelets`' own text form), so `parse_cube` reads it back with this
    /// scheme.
    pub fn paint(&self, cube: &Facelets) -> String {
        cube.stickers()
            .iter()
            .map(|&face| self.letter_of(face))
            .collect()
    }

    /// Reads the scheme off the six fixed centres of an odd cube, each of which
    /// shows the colour of its own face. `colours` holds `6 * n * n` letters.
    pub fn from_centres(colours: &[char], size: usize) -> Result<ColourScheme, SchemeError> {
        if size < 3 || size.is_multiple_of(2) {
            return Err(SchemeError::NoCentres(size));
        }
        let face_len = size * size;
        let mut letters = ['?'; 6];
        for face in Face::ALL {
            letters[face.index()] = colours[face.index() * face_len + face_len / 2];
        }
        ColourScheme::new(letters)
    }
}

impl fmt::Display for ColourScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters
            .iter()
            .try_for_each(|letter| write!(f, "{letter}"))
    }
}

/// Accepts a preset name (`boy`, `jp`; `western`, `western-boy`, `japanese`
/// and `japanese-bl-wt` also work) or six colour letters in `URFDLB` order,
/// such as `WRGYOB`.
impl FromStr for ColourScheme {
    type Err = SchemeError;

    fn from_str(text: &str) -> Result<ColourScheme, SchemeError> {
        match text.trim().to_ascii_lowercase().as_str() {
            "western-boy" | "western" | "boy" => Ok(ColourScheme::WESTERN_BOY),
            "japanese-bl-wt" | "japanese" | "jp" => Ok(ColourScheme::JAPANESE_BL_WT),
            other => {
                let letters: Vec<char> = other.chars().collect();
                let letters: [char; 6] = letters
                    .try_into()
                    .map_err(|_| SchemeError::Unknown(text.to_owned()))?;
                ColourScheme::new(letters)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeError {
    /// Neither a preset name nor six letters.
    Unknown(String),
    /// A colour letter was given for two faces.
    Duplicate(char),
    /// The cube has no fixed centres to read a scheme from.
    NoCentres(usize),
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeError::Unknown(text) => write!(
                f,
                "{text:?} is not a scheme; use boy, jp or six letters like WRGYOB"
            ),
            SchemeError::Duplicate(letter) => {
                write!(f, "colour {letter} is used for more than one face")
            }
            SchemeError::NoCentres(size) => write!(
                f,
                "a {size}x{size}x{size} has no fixed centres; name a colour scheme"
            ),
        }
    }
}

impl std::error::Error for SchemeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_differ_only_in_the_down_and_back_colours() {
        let western = ColourScheme::WESTERN_BOY.letters();
        let japanese = ColourScheme::JAPANESE_BL_WT.letters();
        assert_eq!(western, ['W', 'R', 'G', 'Y', 'O', 'B']);
        assert_eq!(japanese, ['W', 'R', 'G', 'B', 'O', 'Y']);
    }

    #[test]
    fn parses_names_and_letters() {
        assert_eq!("western".parse(), Ok(ColourScheme::WESTERN_BOY));
        assert_eq!("Japanese".parse(), Ok(ColourScheme::JAPANESE_BL_WT));
        assert_eq!("western-boy".parse(), Ok(ColourScheme::WESTERN_BOY));
        assert_eq!("BOY".parse(), Ok(ColourScheme::WESTERN_BOY));
        assert_eq!("JP".parse(), Ok(ColourScheme::JAPANESE_BL_WT));
        assert_eq!("Japanese-BL-WT".parse(), Ok(ColourScheme::JAPANESE_BL_WT));
        assert_eq!("wrgboy".parse(), Ok(ColourScheme::JAPANESE_BL_WT));
        assert!("WRGYO".parse::<ColourScheme>().is_err());
        assert_eq!(
            "WWGYOB".parse::<ColourScheme>(),
            Err(SchemeError::Duplicate('W'))
        );
    }

    #[test]
    fn the_centres_of_an_odd_cube_name_the_scheme() {
        let text = "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
        let colours: Vec<char> = text.chars().collect();
        assert_eq!(
            ColourScheme::from_centres(&colours, 3),
            Ok(ColourScheme::JAPANESE_BL_WT)
        );
        assert_eq!(
            ColourScheme::from_centres(&colours[..24], 2),
            Err(SchemeError::NoCentres(2))
        );
    }

    #[test]
    fn painting_a_cube_is_read_back_with_the_same_scheme() {
        use crate::notation::parse_cube;
        use crate::puzzle::PuzzleKind;
        let solved = Facelets::solved(PuzzleKind::Cube3);
        assert_eq!(
            ColourScheme::WESTERN_BOY.paint(&solved),
            "WWWWWWWWWRRRRRRRRRGGGGGGGGGYYYYYYYYYOOOOOOOOOBBBBBBBBB"
        );
        assert_eq!(
            ColourScheme::JAPANESE_BL_WT.paint(&solved),
            "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY"
        );
        let moves = "R U F' D2".parse().unwrap();
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3, PuzzleKind::Cube5] {
            let cube = Facelets::solved(kind).apply(&moves).unwrap();
            for scheme in [ColourScheme::WESTERN_BOY, ColourScheme::JAPANESE_BL_WT] {
                assert_eq!(
                    parse_cube(&scheme.paint(&cube), Some(scheme)).unwrap(),
                    cube
                );
            }
        }
    }

    #[test]
    fn face_lookup_round_trips() {
        let scheme = ColourScheme::JAPANESE_BL_WT;
        for face in Face::ALL {
            assert_eq!(scheme.face_of(scheme.letter_of(face)), Some(face));
        }
        assert_eq!(scheme.face_of('X'), None);
    }
}
