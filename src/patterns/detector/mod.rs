//! The pattern detector: reads the structure of a cube state and says what
//! it is, without searching.
//!
//! [`detect`] reports the symmetries that fix the state (with colour swaps),
//! its canonical form, the library pattern it is (if any) and a fingerprint of
//! its piece orbits.
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
mod centre_names;
mod cycles;
mod fingerprint;
mod geometry;
mod halfturn;
mod library;
mod matching;
mod pieces;
mod subgroup;
mod symmetry;
mod text;

#[cfg(test)]
mod tests;

pub(crate) use canon::canonical;
pub use cycles::{Cycle, Cycles, OrbitCycles, Order};
pub use fingerprint::{Fingerprint, OrbitStat};
pub use library::Match;
pub use subgroup::{Membership, Verdict};
pub use symmetry::{Operation, Symmetry};

use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

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
    /// Per piece orbit: how many pieces are displaced, twisted or flipped,
    /// and the cycle type where pieces can be told apart.
    pub fingerprint: Fingerprint,
    /// Well-known subgroups the state is, or may be, in (see `subgroup`).
    pub subgroups: Vec<Membership>,
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
        fingerprint: fingerprint::fingerprint(kind, &state),
        subgroups: subgroup::memberships(kind, &state),
    }
}

/// The cycles in which a move sequence carries the pieces of a cube of `kind`,
/// with the order of the permutation. Exact: pieces are followed through the
/// moves.
pub fn cycles_of_moves(kind: PuzzleKind, moves: &MoveSequence) -> Result<Cycles, String> {
    let pi = matching::of_moves(kind, moves)
        .ok_or_else(|| "a move does not exist on this cube".to_owned())?;
    Ok(cycles::analyse(kind, &pi, true))
}

/// The cycles of a cube given by its colours. On the 2x2x2 and 3x3x3 every
/// piece is told apart and the order is given; on bigger cubes equal pieces
/// are matched in index order, which reproduces the colours but defines no
/// order.
pub fn cycles_of_state(cube: &Facelets) -> Result<Cycles, String> {
    let pi = matching::of_state(cube)?;
    Ok(cycles::analyse(cube.kind(), &pi, cube.kind().size() <= 3))
}
