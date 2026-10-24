//! The pattern detector: reads the structure of a cube state and says what
//! it is, without searching.
//!
//! [`detect`] reports the symmetries that fix the state (with colour swaps).
//! It explains a state and never replaces the search for a solution.
//!
//! ```
//! use cubacle::patterns::{self, detect};
//! use cubacle::puzzle::PuzzleKind;
//!
//! let cube = patterns::find("checkerboard").unwrap().state(PuzzleKind::Cube3).unwrap();
//! assert_eq!(detect(&cube).symmetry.order(), 48);
//! ```

mod geometry;
mod symmetry;
mod text;

pub use symmetry::{Operation, Symmetry};

use crate::puzzle::Facelets;

/// What the detector found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The symmetries that fix the state, up to a colour swap.
    pub symmetry: Symmetry,
}

/// The sticker colours of a cube as numbers 0 to 5 in `URFDLB` order.
pub(crate) fn colours(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| *f as u8).collect()
}

/// Reads the structure of a cube state, any size from 2 to 7.
pub fn detect(cube: &Facelets) -> Report {
    Report {
        symmetry: symmetry::symmetry_of(cube.kind(), &colours(cube)),
    }
}
