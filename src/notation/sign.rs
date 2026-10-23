//! SiGN spellings of layer turns, next to the WCA ones.
//!
//! Reading is automatic, token by token: a token whose letter is lowercase
//! (`r`, `3u2`, `f'`) is SiGN and means the outer layers `Rw`, `3Uw2`, `Fw'`.
//! Every other spelling is read as before, because the two notations agree on
//! it (`R`, `2R`, `Rw`, `3Rw`). A spelling has one meaning only; WCA has none
//! for a lowercase letter on big cubes, so nothing needs a tie-break.
//! A layer range (`2-3r`) is expanded by [`super::range`] into plain moves.
//! Writing is chosen with [`Notation`]; the engines never see it.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use crate::puzzle::{Move, MoveSequence, Turn};

/// How moves are written out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Notation {
    /// `Rw`, `3Rw`, `2R`.
    #[default]
    Wca,
    /// `r`, `3r`, `2R`.
    Sign,
}

impl FromStr for Notation {
    type Err = String;

    fn from_str(text: &str) -> Result<Notation, String> {
        match text {
            "wca" => Ok(Notation::Wca),
            "sign" => Ok(Notation::Sign),
            _ => Err(format!("notation must be wca or sign, not {text:?}")),
        }
    }
}

impl Notation {
    /// One move in this notation.
    pub fn write_move(self, mv: &Move) -> String {
        if self == Notation::Wca || !mv.wide {
            return mv.to_string();
        }
        // SiGN leaves out the 2 of the two-layer turn: `u`, not `2u`.
        let depth = if mv.depth == 2 {
            String::new()
        } else {
            mv.depth.to_string()
        };
        let turn = match mv.turn {
            Turn::Clockwise => "",
            Turn::Half => "2",
            Turn::CounterClockwise => "'",
        };
        format!("{depth}{}{turn}", mv.face.letter().to_ascii_lowercase())
    }

    /// A sequence in this notation, separated by single spaces.
    pub fn write(self, moves: &MoveSequence) -> String {
        let words: Vec<String> = moves.moves().iter().map(|mv| self.write_move(mv)).collect();
        words.join(" ")
    }
}

/// Why a move string was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovesError {
    pub token: String,
    pub reason: &'static str,
}

impl fmt::Display for MovesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid move {:?} ({})", self.token, self.reason)
    }
}

impl std::error::Error for MovesError {}

/// The WCA spelling of a SiGN token; any other token comes back unchanged.
///
/// `r` is `Rw`, `3r` is `3Rw`, `2r` is `2Rw`.
pub fn to_wca_token(token: &str) -> Result<Cow<'_, str>, MovesError> {
    let fail = |reason| MovesError {
        token: token.to_owned(),
        reason,
    };
    let digits = token.chars().take_while(char::is_ascii_digit).count();
    let (prefix, rest) = token.split_at(digits);
    let mut chars = rest.chars();
    match chars.next() {
        Some(c) if matches!(c, 'u' | 'r' | 'f' | 'd' | 'l' | 'b') => {
            let tail = chars.as_str();
            if tail.starts_with('w') {
                return Err(fail("a lowercase letter already means a wide turn"));
            }
            Ok(Cow::Owned(format!(
                "{prefix}{}w{tail}",
                c.to_ascii_uppercase()
            )))
        }
        _ if token.contains('-') && token.chars().any(|c| c.is_ascii_alphabetic()) => Err(fail(
            "a layer range is more than one move; read it as a sequence",
        )),
        _ => Ok(Cow::Borrowed(token)),
    }
}

/// One move in WCA, SiGN or SSE spelling.
pub fn parse_move(token: &str) -> Result<Move, MovesError> {
    if let Some(sse) = super::sse::expand(token, None) {
        return match sse?.as_slice() {
            [one] => Ok(*one),
            _ => Err(MovesError {
                token: token.to_owned(),
                reason: "an SSE twist of several moves; read it as a sequence",
            }),
        };
    }
    to_wca_token(token)?.parse().map_err(|_| MovesError {
        token: token.to_owned(),
        reason: "not a move in WCA or SiGN notation",
    })
}

/// Moves separated by whitespace, each in WCA, SiGN or SSE spelling (mixing is
/// fine). A layer range such as `2-3r` is expanded into its plain moves. SSE
/// twists that depend on the cube size (`M W S C`) are refused here, because
/// no size is given; [`crate::notation::parse_expression`] reads them.
pub fn parse_moves(text: &str) -> Result<MoveSequence, MovesError> {
    let mut moves = Vec::new();
    for word in text.split_whitespace() {
        match super::range::expand(word) {
            Some(range) => moves.extend(range?),
            None => match super::sse::expand(word, None) {
                Some(sse) => moves.extend(sse?),
                None => moves.push(parse_move(word)?),
            },
        }
    }
    Ok(MoveSequence::new(moves))
}
