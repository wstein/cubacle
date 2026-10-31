//! Orbit64 V2 state tokens: the rank of a registry entry's state, written at
//! the entry's length.
//!
//! The format is defined by `FORMAT-V2.md` of <https://github.com/wstein/flix-orbit64>
//! (draft). V2 has no class bits: a token's length and value select a registry
//! entry (a puzzle type, here the `n x n x n` cubes for `n = 2..=15` in a
//! color or picture model, canonical or framed), and the rest of the value is
//! the rank of the state among that entry's states.
//!
//! This module works on orbit coordinates only ([`Cube`]); it has no facelet
//! bridge. The color framed entries of the 2x2x2, 3x3x3, 4x4x4, 6x6x6 and
//! 7x7x7 start at offset 0 at the widths of V1's state tokens and share V1's
//! digit order, so a V1 state token of those sizes is also the V2 token of the
//! same state. A V1 5x5x5 token (43 characters) is rejected.
//!
//! Origin: Rust re-implementation of `Orbit64.V2` (flix-orbit64, Apache-2.0,
//! Werner Stein).

use std::fmt;

use super::big::Big;
use super::ALPHABET;

mod cube;
mod digits;
mod registry;

pub use cube::Cube;
pub use registry::{entries, entry, Entry, Model, Pose};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2Error {
    /// The registry has no entry with this ID.
    UnknownEntry(usize),
    /// The token has a character outside the base64url alphabet.
    BadCharacter { position: usize, found: char },
    /// No entry holds the token: it is malformed, or belongs to an entry that
    /// a newer registry added.
    UnknownToken,
    /// The coordinates are not a state of the entry, for the reason given.
    Invalid(String),
}

impl fmt::Display for V2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            V2Error::UnknownEntry(id) => write!(f, "unknown registry entry {id}"),
            V2Error::BadCharacter { position, found } => write!(
                f,
                "orbit64 v2 token has {found:?} at {position}, outside the base64url alphabet"
            ),
            V2Error::UnknownToken => write!(f, "not a known V2 state token"),
            V2Error::Invalid(why) => write!(f, "not a state of this entry ({why})"),
        }
    }
}

impl std::error::Error for V2Error {}

/// Encodes a state of the registry entry `id` as a token.
///
/// The state must already be in the entry's pose. A state that breaks a
/// layout rule, an orientation law or a parity relation is an error, not a
/// token.
pub fn encode(id: usize, cube: &Cube) -> Result<String, V2Error> {
    let entry = entry(id).ok_or(V2Error::UnknownEntry(id))?;
    cube.validate(entry.n, entry.model, entry.pose)?;
    let digits = digits::digits(entry.n, entry.model, entry.pose, cube)?;
    let mut value = digits::rank(entry.n, entry.model, entry.pose, &digits);
    value.add(&entry.offset);
    value
        .to_base64(entry.length)
        .ok_or_else(|| V2Error::Invalid("the rank does not fit the token length".to_string()))
}

/// Decodes a token to its registry entry and state.
///
/// Never panics: a character outside the alphabet, a length no entry has and
/// a value no entry holds are errors.
pub fn decode(token: &str) -> Result<(&'static Entry, Cube), V2Error> {
    if let Some((position, found)) = token
        .chars()
        .enumerate()
        .find(|&(_, found)| !u8::try_from(found).is_ok_and(|byte| ALPHABET.contains(&byte)))
    {
        return Err(V2Error::BadCharacter { position, found });
    }
    let length = token.len();
    if !entries().iter().any(|entry| entry.length == length) {
        return Err(V2Error::UnknownToken);
    }
    let value = Big::from_base64(token)
        .map_err(|(position, found)| V2Error::BadCharacter { position, found })?;
    let (entry, rank) = registry::lookup(length, &value).ok_or(V2Error::UnknownToken)?;
    let digits = digits::split(entry.n, entry.model, entry.pose, rank);
    Ok((
        entry,
        digits::from_digits(entry.n, entry.model, entry.pose, &digits),
    ))
}

#[cfg(test)]
mod tests;
