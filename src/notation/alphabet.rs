//! Reading a move set: `--moves SET` of `solve` and `patterns find`.
//!
//! A set is one of
//!
//! - a name (see [`NAMED_SETS`]): `htm` (no restriction), `qtm`, `half-turns`,
//!   `quarter-slices`;
//! - generators in angle brackets, `<R,U>` or `<R2 U F>`: every turn of each
//!   generator (a half turn generator gives the half turn only);
//! - a list of moves in any notation [`super::parse_expression`] reads, taken
//!   literally: `"R U R2 U'"` has these four moves and not `R'`.
//!
//! The sets that name layers say which: on the 2x2x2 and 3x3x3 the faces
//! (`qtm` is the twelve quarter turns of the faces), on bigger cubes every
//! single layer.

use std::fmt;

use super::expr::{parse_expression, ExprError};
use crate::puzzle::{inner_layer_set, layer_set, Alphabet, Move, PuzzleKind, Turn};

/// The named sets and what they stand for, for help texts.
pub const NAMED_SETS: [(&str, &str); 4] = [
    (
        "htm",
        "no restriction: the engine's own moves (the default)",
    ),
    (
        "qtm",
        "quarter turns only (of every layer on cubes from the 4x4x4)",
    ),
    (
        "half-turns",
        "half turns only: U2 D2 F2 B2 L2 R2 (of every layer on cubes from the 4x4x4)",
    ),
    (
        "quarter-slices",
        "quarter turns of the inner layers only (M, E and S on the 3x3x3)",
    ),
];

/// Why a move set could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlphabetError {
    /// The text is empty.
    Empty,
    /// The moves could not be read.
    Moves(ExprError),
    /// An angle bracket is open or the generators are missing.
    Generators(String),
    /// The set has no move on this cube.
    NoMoves(String),
    /// More moves than a search can tell apart.
    TooMany { found: usize, most: usize },
}

impl fmt::Display for AlphabetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlphabetError::Empty => f.write_str("the move set is empty"),
            AlphabetError::Moves(error) => write!(f, "the move set is not valid ({error})"),
            AlphabetError::Generators(text) => write!(f, "the generator set is not valid ({text})"),
            AlphabetError::NoMoves(label) => {
                write!(f, "the move set {label:?} has no move on this cube")
            }
            AlphabetError::TooMany { found, most } => {
                write!(
                    f,
                    "the move set has {found} moves, at most {most} are supported"
                )
            }
        }
    }
}

impl std::error::Error for AlphabetError {}

/// Spaces, underscores and case do not matter in a name.
fn squash(text: &str) -> String {
    text.trim()
        .chars()
        .map(|c| if c == '_' || c == ' ' { '-' } else { c })
        .flat_map(char::to_lowercase)
        .collect()
}

/// Reads a move set for a cube of `kind`.
pub fn parse_alphabet(text: &str, kind: PuzzleKind) -> Result<Alphabet, AlphabetError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AlphabetError::Empty);
    }
    let name = squash(trimmed);
    let quarters = [Turn::Clockwise, Turn::CounterClockwise];
    let alphabet = match name.as_str() {
        "htm" => return Ok(Alphabet::unrestricted(kind)),
        "qtm" => Alphabet::new(kind, "qtm", layer_set(kind, &quarters, kind.size() <= 3)),
        "half-turns" | "half" | "halfturns" => Alphabet::new(
            kind,
            "half-turns",
            layer_set(kind, &[Turn::Half], kind.size() <= 3),
        ),
        "quarter-slices" | "quarter-slice" => {
            Alphabet::new(kind, "quarter-slices", inner_layer_set(kind, &quarters))
        }
        _ if trimmed.starts_with('<') => generated(trimmed, kind)?,
        _ => {
            let moves = parse_expression(trimmed, kind).map_err(AlphabetError::Moves)?;
            Alphabet::new(kind, trimmed, moves.moves().to_vec())
        }
    };
    if alphabet.is_empty() {
        return Err(AlphabetError::NoMoves(alphabet.label().to_owned()));
    }
    Ok(alphabet)
}

/// `<R,U>`: every turn of every generator.
fn generated(text: &str, kind: PuzzleKind) -> Result<Alphabet, AlphabetError> {
    let inner = text
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
        .ok_or_else(|| AlphabetError::Generators("expected <GENERATORS>".to_owned()))?;
    let words: Vec<&str> = inner
        .split(|c: char| c == ',' || c == '·' || c.is_whitespace())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        return Err(AlphabetError::Generators("no generator".to_owned()));
    }
    let mut moves: Vec<Move> = Vec::new();
    for word in words {
        let sequence = parse_expression(word, kind).map_err(AlphabetError::Moves)?;
        for mv in sequence.moves() {
            let turns: &[Turn] = match mv.turn {
                Turn::Half => &[Turn::Half],
                _ => &[Turn::Clockwise, Turn::Half, Turn::CounterClockwise],
            };
            moves.extend(
                turns
                    .iter()
                    .map(|&turn| Move::new(mv.face, mv.depth, mv.wide, turn)),
            );
        }
    }
    Ok(Alphabet::new(kind, text, moves))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(text: &str, kind: PuzzleKind) -> Alphabet {
        parse_alphabet(text, kind).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    fn texts(alphabet: &Alphabet) -> Vec<String> {
        alphabet.moves().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn htm_restricts_nothing() {
        for kind in PuzzleKind::ALL {
            assert!(set("htm", kind).is_unrestricted());
            assert!(set(" HTM ", kind).is_unrestricted());
        }
    }

    #[test]
    fn the_named_sets_of_the_3x3x3() {
        let kind = PuzzleKind::Cube3;
        assert_eq!(set("qtm", kind).len(), 12);
        assert_eq!(set("half-turns", kind).len(), 6);
        assert!(texts(&set("half turns", kind))
            .iter()
            .all(|m| m.ends_with('2')));
        let slices = set("quarter-slices", kind);
        assert_eq!(slices.len(), 6);
        assert!(slices.contains(&"2R".parse().unwrap()));
        assert!(!slices.contains(&"R".parse().unwrap()));
    }

    #[test]
    fn the_named_sets_of_a_bigger_cube_take_every_layer() {
        let kind = PuzzleKind::Cube5;
        assert_eq!(set("half-turns", kind).len(), 15);
        assert_eq!(set("qtm", kind).len(), 30);
        assert_eq!(set("quarter-slices", kind).len(), 18);
    }

    #[test]
    fn generators_give_every_turn() {
        let kind = PuzzleKind::Cube3;
        let ru = set("<R,U>", kind);
        assert_eq!(texts(&ru), ["R", "R2", "R'", "U", "U2", "U'"]);
        assert_eq!(texts(&set("<R2 U>", kind)), ["R2", "U", "U2", "U'"]);
        assert_eq!(set("<R, U, F>", kind).len(), 9);
        assert!(parse_alphabet("<R,U", kind).is_err());
        assert!(parse_alphabet("<>", kind).is_err());
    }

    #[test]
    fn a_list_is_taken_literally_in_any_notation() {
        let kind = PuzzleKind::Cube3;
        assert_eq!(texts(&set("R U R2 U'", kind)), ["R", "U", "R2", "U'"]);
        assert_eq!(texts(&set("R·U·R2", kind)), ["R", "U", "R2"]);
        assert_eq!(texts(&set("R R U", kind)), ["R", "U"]);
        let wide = set("r u", PuzzleKind::Cube4);
        assert!(wide.contains(&"Rw".parse().unwrap()));
        let range = set("2-3Rw", PuzzleKind::Cube5);
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn errors_say_what_is_wrong() {
        let kind = PuzzleKind::Cube3;
        assert_eq!(parse_alphabet("  ", kind), Err(AlphabetError::Empty));
        assert!(matches!(
            parse_alphabet("R X9", kind),
            Err(AlphabetError::Moves(_))
        ));
        assert!(matches!(
            parse_alphabet("quarter-slices", PuzzleKind::Cube2),
            Err(AlphabetError::NoMoves(_))
        ));
    }
}
