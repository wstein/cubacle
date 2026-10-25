use std::fmt;

use super::orbit64::{self, Orbit64Error};
use super::{ColourScheme, SchemeError};
use crate::puzzle::{Face, Facelets, FaceletsError, PuzzleKind};

/// Why a cube description could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    /// The text has a character that fits no known notation.
    BadCharacter {
        position: usize,
        found: char,
    },
    /// The stickers are not those of a supported cube.
    Facelets(FaceletsError),
    /// The colours could not be assigned to faces.
    Scheme(SchemeError),
    /// An odd cube's centres disagree with the colour scheme that was named.
    CentresDisagree {
        scheme: ColourScheme,
    },
    Orbit64(Orbit64Error),
    /// A partial target that no state can meet.
    Target(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::BadCharacter { position, found } => write!(
                f,
                "character {found:?} at {position} is neither a face (URFDLB), a colour (WRGYOB) nor orbit64"
            ),
            InputError::Facelets(error) => error.fmt(f),
            InputError::Scheme(error) => error.fmt(f),
            InputError::CentresDisagree { scheme } => {
                write!(f, "the centres do not match the colour scheme {scheme}")
            }
            InputError::Orbit64(error) => error.fmt(f),
            InputError::Target(reason) => f.write_str(reason),
        }
    }
}

impl std::error::Error for InputError {}

impl From<FaceletsError> for InputError {
    fn from(error: FaceletsError) -> InputError {
        InputError::Facelets(error)
    }
}

impl From<SchemeError> for InputError {
    fn from(error: SchemeError) -> InputError {
        InputError::Scheme(error)
    }
}

impl From<Orbit64Error> for InputError {
    fn from(error: Orbit64Error) -> InputError {
        InputError::Orbit64(error)
    }
}

const FACE_LETTERS: &str = "URFDLB";
const COLOUR_LETTERS: &str = "WRGYOB";

/// Reads a cube from any of the supported notations.
///
/// - an Orbit64 state token, such as `A4ib8aejxj4Y`;
/// - facelets in `URFDLB` order, compact or spaced (`UUUUUUUUU RRRRRRRRR ...`);
/// - sticker colours `W R G Y O B`, compact or spaced.
///
/// Whitespace is ignored anywhere. Colours are read through `scheme` when one is
/// given; otherwise an odd cube's fixed centres decide, and a cube without
/// centres (the 2x2x2) falls back to [`ColourScheme::WESTERN_BOY`].
pub fn parse_cube(text: &str, scheme: Option<ColourScheme>) -> Result<Facelets, InputError> {
    let compact: String = text.split_whitespace().collect();

    if orbit64::looks_like_token(&compact) {
        return Ok(orbit64::decode(&compact)?);
    }
    // Face letters and colour letters share R and B; whichever alphabet covers
    // more of the text is the one the writer meant, and names the bad character.
    let upper = compact.to_ascii_uppercase();
    let hits = |alphabet: &str| upper.chars().filter(|c| alphabet.contains(*c)).count();
    if hits(FACE_LETTERS) >= hits(COLOUR_LETTERS) {
        if let Some((position, found)) = upper
            .chars()
            .enumerate()
            .find(|(_, c)| !FACE_LETTERS.contains(*c))
        {
            return Err(InputError::BadCharacter { position, found });
        }
        return Ok(Facelets::parse(&upper)?);
    }
    parse_colours(&upper, scheme)
}

/// The colour scheme a cube written as colours shows in its fixed centres, or
/// `None` for other notations and for cubes without readable centres.
pub fn colour_scheme_of(text: &str) -> Option<ColourScheme> {
    let upper: String = text
        .split_whitespace()
        .collect::<String>()
        .to_ascii_uppercase();
    if orbit64::looks_like_token(&upper) {
        return None;
    }
    let hits = |alphabet: &str| upper.chars().filter(|c| alphabet.contains(*c)).count();
    if hits(FACE_LETTERS) >= hits(COLOUR_LETTERS)
        || !upper.chars().all(|c| COLOUR_LETTERS.contains(c))
    {
        return None;
    }
    let colours: Vec<char> = upper.chars().collect();
    let kind = PuzzleKind::of_facelet_count(colours.len())?;
    ColourScheme::from_centres(&colours, kind.size()).ok()
}

fn parse_colours(compact: &str, scheme: Option<ColourScheme>) -> Result<Facelets, InputError> {
    let colours: Vec<char> = compact.chars().map(|c| c.to_ascii_uppercase()).collect();
    if let Some((position, &found)) = colours
        .iter()
        .enumerate()
        .find(|(_, c)| !COLOUR_LETTERS.contains(**c))
    {
        return Err(InputError::BadCharacter { position, found });
    }
    let kind = PuzzleKind::of_facelet_count(colours.len())
        .ok_or(FaceletsError::BadLength(colours.len()))?;

    let from_centres = ColourScheme::from_centres(&colours, kind.size()).ok();
    let scheme = match (scheme, from_centres) {
        (Some(named), Some(seen)) if named != seen => {
            return Err(InputError::CentresDisagree { scheme: named })
        }
        (Some(named), _) => named,
        (None, Some(seen)) => seen,
        (None, None) if kind.size() % 2 == 1 => {
            // Odd cube whose centres repeat a colour: report that, not a default.
            return Err(ColourScheme::from_centres(&colours, kind.size())
                .expect_err("centres are not a scheme")
                .into());
        }
        (None, None) => ColourScheme::WESTERN_BOY,
    };

    let stickers = colours
        .iter()
        .enumerate()
        .map(|(position, &colour)| {
            scheme.face_of(colour).ok_or(InputError::BadCharacter {
                position,
                found: colour,
            })
        })
        .collect::<Result<Vec<Face>, _>>()?;
    Ok(Facelets::from_stickers(stickers)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLVED_3X3: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    /// The cube from the user's western-colours example.
    const SCRAMBLED_COLOURS: &str = "GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO";
    const SCRAMBLED_FACES: &str = "FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL";

    #[test]
    fn compact_and_spaced_facelets_agree() {
        let spaced = "UUUUUUUUU RRRRRRRRR FFFFFFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB";
        let per_sticker: String = SOLVED_3X3.chars().flat_map(|c| [c, ' ']).collect();
        for text in [SOLVED_3X3, spaced, per_sticker.as_str()] {
            assert_eq!(parse_cube(text, None).unwrap().to_string(), SOLVED_3X3);
        }
    }

    #[test]
    fn western_colours_are_read_from_the_centres() {
        let cube = parse_cube(SCRAMBLED_COLOURS, None).unwrap();
        assert_eq!(cube.to_string(), SCRAMBLED_FACES);
        let spaced: String = SCRAMBLED_COLOURS
            .as_bytes()
            .chunks(9)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(parse_cube(&spaced, None).unwrap(), cube);
    }

    #[test]
    fn the_japanese_scheme_is_read_from_its_centres() {
        let jap = "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
        assert_eq!(parse_cube(jap, None).unwrap().to_string(), SOLVED_3X3);
        assert_eq!(
            parse_cube(jap, Some(ColourScheme::JAPANESE_BL_WT))
                .unwrap()
                .to_string(),
            SOLVED_3X3
        );
        // The same letters mean something else under the western scheme.
        assert_eq!(
            parse_cube(jap, Some(ColourScheme::WESTERN_BOY)),
            Err(InputError::CentresDisagree {
                scheme: ColourScheme::WESTERN_BOY
            })
        );
    }

    #[test]
    fn colours_are_case_insensitive() {
        assert_eq!(
            parse_cube(&SCRAMBLED_COLOURS.to_lowercase(), None)
                .unwrap()
                .to_string(),
            SCRAMBLED_FACES
        );
    }

    #[test]
    fn a_2x2_without_centres_needs_a_scheme_or_takes_the_western_one() {
        let solved = "WWWW RRRR GGGG YYYY OOOO BBBB";
        assert!(parse_cube(solved, None).unwrap().is_solved());
        let japanese = "WWWW RRRR GGGG BBBB OOOO YYYY";
        assert!(parse_cube(japanese, Some(ColourScheme::JAPANESE_BL_WT))
            .unwrap()
            .is_solved());
        assert!(!parse_cube(japanese, None).unwrap().is_solved());
    }

    #[test]
    fn orbit64_tokens_are_recognised() {
        assert_eq!(
            parse_cube("A4ib8aejxj4Y", None).unwrap().to_string(),
            SCRAMBLED_FACES
        );
        assert!(parse_cube("AAAAAAAAAAAA", None).unwrap().is_solved());
        assert!(parse_cube("AAAAA", None).unwrap().is_solved());
    }

    #[test]
    fn rejects_garbage_with_a_position() {
        assert_eq!(
            parse_cube(
                "UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU?",
                None
            ),
            Err(InputError::BadCharacter {
                position: 54,
                found: '?'
            })
        );
        assert!(parse_cube("WWW", None).is_err());
        assert!(parse_cube("", None).is_err());
    }

    #[test]
    fn repeated_center_colors_are_reported() {
        let text = "WWWWWWWWWWWWWWWWWWGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
        assert!(matches!(
            parse_cube(text, None),
            Err(InputError::Scheme(SchemeError::Duplicate('W')))
        ));
    }
}
