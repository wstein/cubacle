//! Why a cube state cannot be reached by turning a real cube.
//!
//! One typed reason for every size from the 2x2x2 to the 7x7x7, found by
//! [`check`] and written in the same words on every size, so that a twisted
//! corner, a flipped edge or two swapped pieces is said the same way whether
//! the cube has 2 or 7 layers.

use std::fmt;

use crate::puzzle::Facelets;

use super::orbit64;

/// The kind of piece a reason is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Corner,
    /// The middle edge of an odd cube, the only "edge" of a 3x3x3.
    Edge,
    /// Any other edge piece: a 4x4x4 and larger have them.
    Wing,
}

impl PieceKind {
    fn name(self) -> &'static str {
        match self {
            PieceKind::Corner => "corner",
            PieceKind::Edge => "edge",
            PieceKind::Wing => "wing",
        }
    }
}

/// The reason a state, whose colours each cover `n * n` stickers, cannot be
/// reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Impossible {
    /// The six fixed centres of an odd cube are not a rotation of the cube.
    FixedCentres,
    /// A centre orbit does not hold each colour exactly four times.
    CentreColours,
    /// The stickers of one cubie, named by the faces of its place, are the
    /// colours of no piece of a cube (a repeated, opposite or mirrored set).
    Sticker {
        kind: PieceKind,
        place: String,
        seen: String,
    },
    /// Some piece of the kind appears twice and another not at all.
    NotDistinct(PieceKind),
    /// The corner twists do not add up to a multiple of three.
    CornerTwist,
    /// The edge flips do not add up to an even number.
    EdgeFlip,
    /// The corner and edge permutations have different parity.
    Parity,
}

impl Impossible {
    /// A short hint what makes the state possible, where the repair is certain.
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Impossible::CornerTwist => Some("turn one corner back"),
            Impossible::EdgeFlip => Some("flip one edge back"),
            Impossible::Parity => Some("swap two edges"),
            _ => None,
        }
    }
}

impl fmt::Display for Impossible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Impossible::FixedCentres => write!(
                f,
                "the centres are invalid: the six fixed centres are not a rotation of the cube"
            ),
            Impossible::CentreColours => write!(
                f,
                "the centres are invalid: each colour must appear exactly four times in a centre orbit"
            ),
            Impossible::Sticker { kind, place, seen } => write!(
                f,
                "the {0}s are invalid: the stickers {seen} at {0} {place} are not a {0} of the cube",
                kind.name()
            ),
            Impossible::NotDistinct(kind) => write!(
                f,
                "the {0}s are invalid: not every {0} piece appears exactly once",
                kind.name()
            ),
            Impossible::CornerTwist => write!(
                f,
                "the corners are invalid: one corner is twisted on its own"
            ),
            Impossible::EdgeFlip => {
                write!(f, "the edges are invalid: one edge is flipped on its own")
            }
            Impossible::Parity => write!(
                f,
                "the cube is invalid: two pieces are swapped (parity error)"
            ),
        }
    }
}

impl std::error::Error for Impossible {}

/// Checks that a state can be reached by turning a cube of its size.
///
/// Works for every size from 2 to 7. The first reason found is returned; a
/// state with several faults reports one of them.
pub fn check(facelets: &Facelets) -> Result<(), Impossible> {
    orbit64::encode(facelets)
        .map(|_| ())
        .or_else(|error| match error {
            orbit64::Orbit64Error::Impossible(reason) => Err(reason),
            // Encoding a well-formed state fails for no other reason.
            _ => Ok(()),
        })
}

#[cfg(test)]
mod tests {
    include!("impossible_tests.rs");
}
