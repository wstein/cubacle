//! Targets with open stickers: facelets or colours where `X` (or `?`) marks a
//! sticker that may show any colour.

use super::input::InputError;
use super::{orbit64, ColourScheme};
use crate::puzzle::{Face, FaceletsError, PuzzleKind, Target, TargetError};

const FACE_LETTERS: &str = "URFDLB";
const COLOUR_LETTERS: &str = "WRGYOB";

/// Reads `text` as a partial target, or returns `Ok(None)` when it is not one.
///
/// A text is partial when, whitespace aside, it has the length of a cube string
/// and has an uppercase `X` or a `?` as a sticker. Lowercase `x` is the cube
/// rotation and never a sticker. The other stickers are face letters
/// (`URFDLB`) or colour letters read through `scheme` (the centres of an odd
/// cube decide when none is given, else the Western scheme). Orbit64 tokens
/// cannot express open stickers and are not read here.
pub fn parse_partial(
    text: &str,
    scheme: Option<ColourScheme>,
) -> Result<Option<Target>, InputError> {
    let compact: String = text.split_whitespace().collect();
    let open = |c: char| c == 'X' || c == '?';
    if !compact.chars().any(open) || orbit64::looks_like_token(&compact) {
        return Ok(None);
    }
    let Some(kind) = PuzzleKind::of_facelet_count(compact.chars().count()) else {
        return Ok(None);
    };
    let chars: Vec<char> = compact.chars().collect();
    let upper: Vec<char> = chars.iter().map(|c| c.to_ascii_uppercase()).collect();
    let letters = |alphabet: &str| {
        upper
            .iter()
            .zip(&chars)
            .filter(|(u, c)| !open(**c) && alphabet.contains(**u))
            .count()
    };
    let faces = letters(FACE_LETTERS) >= letters(COLOUR_LETTERS);
    let scheme = if faces {
        None
    } else {
        let seen = ColourScheme::from_centres(&upper, kind.size())
            .ok()
            .filter(|seen| !seen.letters().contains(&'X') && !seen.letters().contains(&'?'));
        match (scheme, seen) {
            (Some(named), Some(seen)) if named != seen => {
                return Err(InputError::CentresDisagree { scheme: named })
            }
            (Some(named), _) => Some(named),
            (None, seen) => Some(seen.unwrap_or(ColourScheme::WESTERN_BOY)),
        }
    };
    let mut sets = Vec::with_capacity(chars.len());
    for (position, (&c, &u)) in chars.iter().zip(&upper).enumerate() {
        if open(c) {
            sets.push(0b11_1111);
            continue;
        }
        let face = if faces {
            Face::from_letter(u)
        } else {
            scheme.and_then(|scheme| scheme.face_of(u))
        };
        match face {
            Some(face) => sets.push(1 << face.index()),
            None => return Err(InputError::BadCharacter { position, found: c }),
        }
    }
    Target::from_sets(sets)
        .map(Some)
        .map_err(|error| match error {
            TargetError::BadLength(len) => InputError::Facelets(FaceletsError::BadLength(len)),
            other => InputError::Target(other.to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP_ONLY: &str = "UUUUUUUUUXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";

    #[test]
    fn faces_with_open_stickers_are_partial() {
        let target = parse_partial(UP_ONLY, None).unwrap().unwrap();
        assert_eq!(target.constrained(), 9);
        assert_eq!(target.to_text().as_deref(), Some(UP_ONLY));
    }

    #[test]
    fn question_marks_are_read_and_x_is_written_back() {
        let marks = UP_ONLY.replace('X', "?");
        let target = parse_partial(&marks, None).unwrap().unwrap();
        assert_eq!(target.to_text().as_deref(), Some(UP_ONLY));
    }

    #[test]
    fn colours_are_read_through_the_scheme() {
        let text = "WWWWWWWWWXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
        let target = parse_partial(text, None).unwrap().unwrap();
        assert_eq!(target.to_text().as_deref(), Some(UP_ONLY));
    }

    #[test]
    fn complete_texts_and_expressions_are_not_partial() {
        assert!(parse_partial("R U", None).unwrap().is_none());
        let solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        assert!(parse_partial(solved, None).unwrap().is_none());
        // lowercase x is the rotation, not a sticker
        assert!(parse_partial("x y z", None).unwrap().is_none());
        assert!(parse_partial(&UP_ONLY.replace('X', "x"), None)
            .unwrap()
            .is_none());
    }

    #[test]
    fn bad_characters_and_contradictions_are_named() {
        let text = UP_ONLY.replacen('U', "Q", 1);
        assert!(matches!(
            parse_partial(&text, None),
            Err(InputError::BadCharacter {
                position: 0,
                found: 'Q'
            })
        ));
        let too_many = format!("UUUUUUUUUU{}", &UP_ONLY[10..]);
        assert!(matches!(
            parse_partial(&too_many, None),
            Err(InputError::Target(_))
        ));
    }
}
