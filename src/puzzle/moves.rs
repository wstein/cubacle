use std::fmt;
use std::str::FromStr;

use super::{Face, PuzzleKind};

/// How far a layer is turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Turn {
    Clockwise,
    Half,
    CounterClockwise,
}

impl Turn {
    /// The turn a suffix such as `'`, `2`, `2'`, `'2`, `1` or `3` stands for;
    /// the empty suffix is a clockwise quarter. Shared by layer moves and the
    /// rotation and slice letters.
    pub(crate) fn from_suffix(suffix: &str) -> Option<Turn> {
        match suffix {
            "" | "1" => Some(Turn::Clockwise),
            "2" | "2'" | "'2" => Some(Turn::Half),
            "'" | "3" => Some(Turn::CounterClockwise),
            _ => None,
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Turn::Clockwise => "",
            Turn::Half => "2",
            Turn::CounterClockwise => "'",
        }
    }
}

/// One layer turn in standard notation.
///
/// `depth` is 1 for the outer layer. Without `wide`, only that single layer
/// turns (`R`, `2R`, `3R`); with `wide`, the outer `depth` layers turn together
/// (`Rw` is two layers, `3Rw` is three). A bare `Rw` therefore has depth 2.
/// Whole-cube rotations are not moves: a solution never contains them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub face: Face,
    pub depth: u8,
    pub wide: bool,
    pub turn: Turn,
}

/// Why a move or sequence could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotationError {
    pub token: String,
}

impl fmt::Display for NotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid move {:?}", self.token)
    }
}

impl std::error::Error for NotationError {}

impl Move {
    pub fn new(face: Face, depth: u8, wide: bool, turn: Turn) -> Move {
        Move {
            face,
            depth,
            wide,
            turn,
        }
    }

    /// Whether this move exists on a cube of the given size.
    ///
    /// A turn must leave at least one layer behind, so `depth < n`.
    pub fn fits(&self, kind: PuzzleKind) -> bool {
        self.depth >= 1 && usize::from(self.depth) < kind.size()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.wide {
            if self.depth != 2 {
                write!(f, "{}", self.depth)?;
            }
            write!(f, "{}w", self.face.letter())?;
        } else {
            if self.depth != 1 {
                write!(f, "{}", self.depth)?;
            }
            write!(f, "{}", self.face.letter())?;
        }
        f.write_str(self.turn.suffix())
    }
}

impl FromStr for Move {
    type Err = NotationError;

    fn from_str(token: &str) -> Result<Move, NotationError> {
        let bad = || NotationError {
            token: token.to_owned(),
        };
        let digits = token.chars().take_while(char::is_ascii_digit).count();
        let (prefix, rest) = token.split_at(digits);
        let mut chars = rest.chars();
        let face = chars.next().and_then(Face::from_letter).ok_or_else(bad)?;
        let rest = chars.as_str();
        let (wide, rest) = match rest.strip_prefix('w') {
            Some(rest) => (true, rest),
            None => (false, rest),
        };
        let turn = Turn::from_suffix(rest).ok_or_else(bad)?;
        let depth = if prefix.is_empty() {
            if wide {
                2
            } else {
                1
            }
        } else {
            prefix.parse::<u8>().map_err(|_| bad())?
        };
        if depth == 0 || (wide && depth < 2) {
            return Err(bad());
        }
        Ok(Move {
            face,
            depth,
            wide,
            turn,
        })
    }
}

/// An ordered list of layer turns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MoveSequence {
    moves: Vec<Move>,
}

impl MoveSequence {
    pub fn new(moves: Vec<Move>) -> MoveSequence {
        MoveSequence { moves }
    }

    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Whether every move exists on a cube of the given size.
    pub fn fits(&self, kind: PuzzleKind) -> bool {
        self.moves.iter().all(|mv| mv.fits(kind))
    }
}

impl fmt::Display for MoveSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, mv) in self.moves.iter().enumerate() {
            if index > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{mv}")?;
        }
        Ok(())
    }
}

impl FromStr for MoveSequence {
    type Err = NotationError;

    fn from_str(text: &str) -> Result<MoveSequence, NotationError> {
        text.split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<Move>, _>>()
            .map(MoveSequence::new)
    }
}

impl FromIterator<Move> for MoveSequence {
    fn from_iter<I: IntoIterator<Item = Move>>(iter: I) -> Self {
        MoveSequence::new(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_notation_round_trips() {
        for text in [
            "R", "R2", "R'", "Rw", "Rw2", "3Rw'", "2R", "3U2", "4Fw", "B'",
        ] {
            let mv: Move = text.parse().expect(text);
            assert_eq!(mv.to_string(), text);
        }
    }

    #[test]
    fn whole_token_aliases_normalize() {
        for (alias, canonical) in [
            ("R1", "R"),
            ("R3", "R'"),
            ("R2'", "R2"),
            ("R'2", "R2"),
            ("U2'", "U2"),
            ("Rw3", "Rw'"),
            ("3Rw2'", "3Rw2"),
        ] {
            let mv: Move = alias.parse().expect(alias);
            assert_eq!(mv.to_string(), canonical, "{alias}");
        }
        let seq: MoveSequence = "R1 U3 F2' D'2".parse().unwrap();
        assert_eq!(seq.to_string(), "R U' F2 D2");
    }

    #[test]
    fn runs_without_separators_are_rejected() {
        for text in ["RUR'", "R U R'U'", "-", "R -", "RU"] {
            assert!(text.parse::<MoveSequence>().is_err(), "{text:?}");
        }
    }

    #[test]
    fn depth_and_width_are_distinguished() {
        let inner: Move = "3R".parse().unwrap();
        assert_eq!((inner.depth, inner.wide), (3, false));
        let wide: Move = "Rw".parse().unwrap();
        assert_eq!((wide.depth, wide.wide), (2, true));
        let wide3: Move = "3Rw".parse().unwrap();
        assert_eq!((wide3.depth, wide3.wide), (3, true));
    }

    #[test]
    fn rejects_malformed_tokens() {
        for text in [
            "", "X", "R4", "R''", "R22", "R1'", "R3'", "0R", "1Rw", "2", "r", "-", "RUR'",
        ] {
            assert!(text.parse::<Move>().is_err(), "{text:?} should be rejected");
        }
    }

    #[test]
    fn sequences_parse_padded_engine_output() {
        // The 3x3 engine pads single-character moves, e.g. "L " before a gap.
        let seq: MoveSequence = "U' R2 L  U' ".parse().unwrap();
        assert_eq!(seq.len(), 4);
        assert_eq!(seq.to_string(), "U' R2 L U'");
    }

    #[test]
    fn fits_respects_the_cube_size() {
        let seq: MoveSequence = "R 3Rw".parse().unwrap();
        assert!(seq.fits(PuzzleKind::Cube4));
        assert!(!seq.fits(PuzzleKind::Cube3));
        assert!(!"6R".parse::<Move>().unwrap().fits(PuzzleKind::Cube6));
        assert!("6R".parse::<Move>().unwrap().fits(PuzzleKind::Cube7));
    }
}
