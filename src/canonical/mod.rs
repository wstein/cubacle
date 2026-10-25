//! The canonical Orbit64 token: one identifier for a pattern.
//!
//! A pattern is a color state, so it must not depend on how the cube is
//! turned, mirrored or colored. [`canonical`] takes the smallest sticker
//! string of the state's orbit under the 48 symmetries and every renaming of
//! the six colors (the detector's canonical form), puts that class into the
//! standard color scheme and writes it as an Orbit64 token.
//!
//! The standard scheme is fixed without a choice: an odd cube's centers name
//! the colors; an even cube has no centers, so its corner in the URF slot is
//! made the solved URF corner. A state of the wrong handedness is mirrored across
//! the R/L plane before that. Orbit64 itself
//! reads colors (a wing by its ordered color pair, a center by its color),
//! so equal-colored pieces need no further choice.
//!
//! ```
//! use cubacle::canonical::canonical_text;
//!
//! // A solved 2x2x2 and the same cube turned and recolored give one token.
//! assert_eq!(canonical_text("UUUURRRRFFFFDDDDLLLLBBBB").unwrap().as_str(), "AAAAA");
//! assert_eq!(canonical_text("RRRRFFFFUUUULLLLBBBBDDDD").unwrap().as_str(), "AAAAA");
//! ```

use std::fmt;

use crate::notation::orbit64::{self, Orbit64Error};
use crate::notation::{parse_cube, Impossible, InputError, PieceKind};
use crate::patterns::{canonical_cube, mirrored};
use crate::puzzle::{Face, Facelets, MoveSequence, PuzzleKind};

/// The canonical Orbit64 state token of a class of cube states.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Token(String);

impl Token {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The size of the cube the token stands for.
    pub fn kind(&self) -> PuzzleKind {
        orbit64::size_of_width(self.0.len())
            .and_then(PuzzleKind::of_size)
            .expect("a canonical token has a state width")
    }

    /// The representative the token stands for: standard colors, right-handed.
    pub fn state(&self) -> Facelets {
        orbit64::decode(&self.0).expect("a canonical token decodes")
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a canonical token could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    /// The text is no cube in any supported notation.
    Input(InputError),
    /// The stickers are those of no reachable cube, in any coloring.
    State(Orbit64Error),
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonError::Input(error) => error.fmt(f),
            CanonError::State(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CanonError {}

/// The canonical token of a cube state.
///
/// Equal for states that differ only by turning, mirroring, recoloring or the
/// choice among equal-colored pieces, and different for every other pair of
/// states. An unreachable state is an error.
pub fn canonical(cube: &Facelets) -> Result<Token, CanonError> {
    let least = canonical_cube(cube);
    let mut first = None;
    for candidate in [least.clone(), mirrored(&least)] {
        let colors: Vec<u8> = candidate.stickers().iter().map(|&f| f as u8).collect();
        let result = standard(candidate.kind(), &colors).and_then(|state| {
            let faces = state.iter().map(|&c| Face::ALL[c as usize]).collect();
            let cube = Facelets::from_stickers(faces).expect("a renaming keeps every count");
            orbit64::encode(&cube).map_err(CanonError::State)
        });
        match result {
            Ok(token) => return Ok(Token(token)),
            Err(error) => first = first.or(Some(error)),
        }
    }
    Err(first.expect("two attempts were made"))
}

/// The canonical token of a cube written as an Orbit64 token, facelets in
/// `URFDLB` order or colors `WRGYOB`. The names of the colors do not matter.
pub fn canonical_text(text: &str) -> Result<Token, CanonError> {
    canonical(&parse_cube(text, None).map_err(CanonError::Input)?)
}

/// The canonical token of the solved cube of `kind` after `moves`.
pub fn canonical_moves(kind: PuzzleKind, moves: &MoveSequence) -> Result<Token, CanonError> {
    let cube = Facelets::solved(kind)
        .apply(moves)
        .map_err(|_| CanonError::Input(InputError::Target("a move does not exist".to_owned())))?;
    canonical(&cube)
}

/// `state` with its colors renamed to the standard scheme.
fn standard(kind: PuzzleKind, state: &[u8]) -> Result<Vec<u8>, CanonError> {
    let n = kind.size();
    let names = if n % 2 == 1 {
        by_centers(n, state)
    } else {
        by_corner(n, state)
    };
    let names = names.ok_or(CanonError::State(Orbit64Error::Impossible(if n % 2 == 1 {
        Impossible::FixedCentres
    } else {
        Impossible::NotDistinct(PieceKind::Corner)
    })))?;
    Ok(state.iter().map(|&c| names[c as usize]).collect())
}

/// Odd cubes: the color in the center of face `f` becomes `f`.
fn by_centers(n: usize, state: &[u8]) -> Option<[u8; 6]> {
    let mut names = [u8::MAX; 6];
    for (face, at) in orbit64::fixed_centers(n).into_iter().enumerate() {
        let slot = &mut names[state[at] as usize];
        if *slot != u8::MAX {
            return None;
        }
        *slot = face as u8;
    }
    Some(names)
}

/// Even cubes: the URF corner becomes `U R F` in turning order and the
/// colors opposite to them (found as the colors that share no corner) `D L B`.
fn by_corner(n: usize, state: &[u8]) -> Option<[u8; 6]> {
    let corners: Vec<[u8; 3]> = orbit64::corner_slots(n)?
        .iter()
        .map(|slot| [state[slot[0]], state[slot[1]], state[slot[2]]])
        .collect();
    let mut shares = [[false; 6]; 6];
    for corner in &corners {
        for a in corner {
            for b in corner {
                shares[*a as usize][*b as usize] = true;
            }
        }
    }
    let opposite = |c: u8| {
        let mut apart = (0..6).filter(|&o| !shares[c as usize][o]);
        match (apart.next(), apart.next()) {
            (Some(o), None) => Some(o as u8),
            _ => None,
        }
    };
    let mut names = [u8::MAX; 6];
    for (i, &c) in corners.first()?.iter().enumerate() {
        for (name, color) in [(i, c), (i + 3, opposite(c)?)] {
            let slot = &mut names[color as usize];
            if *slot != u8::MAX {
                return None;
            }
            *slot = name as u8;
        }
    }
    Some(names)
}

#[cfg(test)]
mod tests;
