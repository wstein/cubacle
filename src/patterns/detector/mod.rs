//! The pattern detector: reads the structure of a cube state and says what
//! it is, without searching.
//!
//! [`detect`] reports the symmetries that fix the state (with colour swaps),
//! its canonical form and the library pattern it is, if any.
//! It explains a state and never replaces the search for a solution.
//!
//! ```
//! use cubacle::patterns::{self, detect};
//! use cubacle::puzzle::PuzzleKind;
//!
//! let cube = patterns::find("checkerboard").unwrap().state(PuzzleKind::Cube3).unwrap();
//! assert_eq!(detect(&cube).symmetry.order(), 48);
//! ```

mod canon;
mod geometry;
mod library;
mod symmetry;
mod text;

#[cfg(test)]
mod tests;

pub use library::Match;
pub use symmetry::{Operation, Symmetry};

use crate::puzzle::Facelets;

/// What the detector found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The symmetries that fix the state, up to a colour swap.
    pub symmetry: Symmetry,
    /// The smallest sticker string in the state's orbit under the 48
    /// symmetries and every renaming of the colours, as a cube: equal for
    /// states that differ only by turning, mirroring or recolouring.
    pub canonical: Facelets,
    /// The library patterns the state is (up to turning, mirroring and
    /// recolouring); empty if it is none of them.
    pub matches: Vec<Match>,
}

/// The sticker colours of a cube as numbers 0 to 5 in `URFDLB` order.
pub(crate) fn colours(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| *f as u8).collect()
}

/// Reads the structure of a cube state, any size from 2 to 7.
pub fn detect(cube: &Facelets) -> Report {
    let kind = cube.kind();
    let state = colours(cube);
    let canon = canon::canonical(kind, &state);
    Report {
        symmetry: symmetry::symmetry_of(kind, &state),
        matches: library::matches(kind, &state, &canon),
        canonical: canon::facelets(&canon),
    }
}
