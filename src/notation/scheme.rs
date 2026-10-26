use std::fmt;
use std::str::FromStr;

use crate::puzzle::{Face, Facelets};

/// Which sticker color sits on which face, in `URFDLB` order.
///
/// A cube given as colors rather than faces needs this to be read. Odd cubes
/// carry the answer in their fixed centers, see [`ColorScheme::from_centers`];
/// even cubes and the 2x2x2 have no centers and need a scheme named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorScheme {
    letters: [char; 6],
}

impl ColorScheme {
    /// White up, green front, red right, yellow down, orange left, blue back.
    /// Opposite pairs: white-yellow, green-blue, red-orange.
    pub const WESTERN_BOY: ColorScheme = ColorScheme {
        letters: ['W', 'R', 'G', 'Y', 'O', 'B'],
    };

    /// White up, green front, red right, blue down, orange left, yellow back.
    /// Opposite pairs: white-blue, green-yellow, red-orange.
    pub const JAPANESE_BL_WT: ColorScheme = ColorScheme {
        letters: ['W', 'R', 'G', 'B', 'O', 'Y'],
    };

    /// The color letters, written in `URFDLB` order. Rejects a letter used twice.
    pub fn new(letters: [char; 6]) -> Result<ColorScheme, SchemeError> {
        let letters = letters.map(|letter| letter.to_ascii_uppercase());
        for (index, letter) in letters.iter().enumerate() {
            if letters[..index].contains(letter) {
                return Err(SchemeError::Duplicate(*letter));
            }
        }
        Ok(ColorScheme { letters })
    }

    pub fn letters(&self) -> [char; 6] {
        self.letters
    }

    /// The face a color letter belongs to, or `None` if the scheme lacks it.
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

    /// The cube as color letters, one per sticker in facelet order (the order
    /// of `Facelets`' own text form), so `parse_cube` reads it back with this
    /// scheme.
    pub fn paint(&self, cube: &Facelets) -> String {
        cube.stickers()
            .iter()
            .map(|&face| self.letter_of(face))
            .collect()
    }

    /// Reads the scheme off the six fixed centers of an odd cube, each of which
    /// shows the color of its own face. `colors` holds `6 * n * n` letters.
    pub fn from_centers(colors: &[char], size: usize) -> Result<ColorScheme, SchemeError> {
        if size < 3 || size.is_multiple_of(2) {
            return Err(SchemeError::NoCenters(size));
        }
        let face_len = size * size;
        let mut letters = ['?'; 6];
        for face in Face::ALL {
            letters[face.index()] = colors[face.index() * face_len + face_len / 2];
        }
        ColorScheme::new(letters)
    }
}

impl fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters
            .iter()
            .try_for_each(|letter| write!(f, "{letter}"))
    }
}

/// Accepts a preset name (`boy`, `jp`; `western`, `western-boy`, `japanese`
/// and `japanese-bl-wt` also work) or six color letters in `URFDLB` order,
/// such as `WRGYOB`.
impl FromStr for ColorScheme {
    type Err = SchemeError;

    fn from_str(text: &str) -> Result<ColorScheme, SchemeError> {
        match text.trim().to_ascii_lowercase().as_str() {
            "western-boy" | "western" | "boy" => Ok(ColorScheme::WESTERN_BOY),
            "japanese-bl-wt" | "japanese" | "jp" => Ok(ColorScheme::JAPANESE_BL_WT),
            other => {
                let letters: Vec<char> = other.chars().collect();
                let letters: [char; 6] = letters
                    .try_into()
                    .map_err(|_| SchemeError::Unknown(text.to_owned()))?;
                ColorScheme::new(letters)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeError {
    /// Neither a preset name nor six letters.
    Unknown(String),
    /// A color letter was given for two faces.
    Duplicate(char),
    /// The cube has no fixed centers to read a scheme from.
    NoCenters(usize),
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeError::Unknown(text) => write!(
                f,
                "{text:?} is not a scheme; use boy, jp or six letters like WRGYOB"
            ),
            SchemeError::Duplicate(letter) => {
                write!(f, "color {letter} is used for more than one face")
            }
            SchemeError::NoCenters(size) => write!(
                f,
                "a {size}x{size}x{size} has no fixed centers; name a color scheme"
            ),
        }
    }
}

impl std::error::Error for SchemeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_differ_only_in_the_down_and_back_colors() {
        let western = ColorScheme::WESTERN_BOY.letters();
        let japanese = ColorScheme::JAPANESE_BL_WT.letters();
        assert_eq!(western, ['W', 'R', 'G', 'Y', 'O', 'B']);
        assert_eq!(japanese, ['W', 'R', 'G', 'B', 'O', 'Y']);
    }

    #[test]
    fn parses_names_and_letters() {
        assert_eq!("western".parse(), Ok(ColorScheme::WESTERN_BOY));
        assert_eq!("Japanese".parse(), Ok(ColorScheme::JAPANESE_BL_WT));
        assert_eq!("western-boy".parse(), Ok(ColorScheme::WESTERN_BOY));
        assert_eq!("BOY".parse(), Ok(ColorScheme::WESTERN_BOY));
        assert_eq!("JP".parse(), Ok(ColorScheme::JAPANESE_BL_WT));
        assert_eq!("Japanese-BL-WT".parse(), Ok(ColorScheme::JAPANESE_BL_WT));
        assert_eq!("wrgboy".parse(), Ok(ColorScheme::JAPANESE_BL_WT));
        assert!("WRGYO".parse::<ColorScheme>().is_err());
        assert_eq!(
            "WWGYOB".parse::<ColorScheme>(),
            Err(SchemeError::Duplicate('W'))
        );
    }

    #[test]
    fn the_centers_of_an_odd_cube_name_the_scheme() {
        let text = "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
        let colors: Vec<char> = text.chars().collect();
        assert_eq!(
            ColorScheme::from_centers(&colors, 3),
            Ok(ColorScheme::JAPANESE_BL_WT)
        );
        assert_eq!(
            ColorScheme::from_centers(&colors[..24], 2),
            Err(SchemeError::NoCenters(2))
        );
    }

    #[test]
    fn painting_a_cube_is_read_back_with_the_same_scheme() {
        use crate::notation::parse_cube;
        use crate::puzzle::PuzzleKind;
        let solved = Facelets::solved(PuzzleKind::Cube3);
        assert_eq!(
            ColorScheme::WESTERN_BOY.paint(&solved),
            "WWWWWWWWWRRRRRRRRRGGGGGGGGGYYYYYYYYYOOOOOOOOOBBBBBBBBB"
        );
        assert_eq!(
            ColorScheme::JAPANESE_BL_WT.paint(&solved),
            "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY"
        );
        let moves = "R U F' D2".parse().unwrap();
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3, PuzzleKind::Cube5] {
            let cube = Facelets::solved(kind).apply(&moves).unwrap();
            for scheme in [ColorScheme::WESTERN_BOY, ColorScheme::JAPANESE_BL_WT] {
                assert_eq!(
                    parse_cube(&scheme.paint(&cube), Some(scheme)).unwrap(),
                    cube
                );
            }
        }
    }

    #[test]
    fn face_lookup_round_trips() {
        let scheme = ColorScheme::JAPANESE_BL_WT;
        for face in Face::ALL {
            assert_eq!(scheme.face_of(scheme.letter_of(face)), Some(face));
        }
        assert_eq!(scheme.face_of('X'), None);
    }
}
