//! The pattern detector: reads the structure of a cube state and says what
//! it is, without searching.
//!
//! [`detect`] reports the symmetries that fix the state (with color swaps),
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
mod center_names;
mod cycles;
mod fingerprint;
pub(super) mod geometry;
mod group;
mod halfturn;
mod kind;
mod library;
mod matching;
mod measures;
mod patternness;
mod pieces;
mod similarity;
mod stabilizer;
mod subgroup;
mod symmetry;
mod text;
mod typename;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
pub(crate) use canon::mirrored;
pub(crate) use canon::{canonical, canonical_cube};
pub use cycles::{Cycle, Cycles, OrbitCycles, Order};
pub use fingerprint::{Fingerprint, OrbitStat};
pub use kind::{kind, PatternKind};
pub use library::Match;
pub use measures::{measures, Measures};
pub use patternness::PATTERNNESS_VERSION;
pub use similarity::{Partial, Similar};
pub use stabilizer::{Antisymmetry, SymmetryType};
pub use subgroup::{Membership, Verdict};
#[cfg(test)]
pub(crate) use symmetry::symmetry_of;
pub use symmetry::{Operation, Symmetry};
pub use text::Detailed;

use crate::patterns::Pattern;
use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

/// What the detector found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The symmetries that fix the state, up to a color swap.
    pub symmetry: Symmetry,
    /// The conjugacy class of the symmetry group in the cube's 48 symmetries
    /// (`D4h`, `C2v#1`, ...); `None` when only the identity fixes the state.
    pub symmetry_type: Option<SymmetryType>,
    /// The class of the group of symmetries and antisymmetries (operations
    /// that give the inverse cube) in the 96 of `M x C2`; `None` without
    /// antisymmetry. Told on the 3x3x3 only (it has fixed centers).
    pub antisymmetry: Option<Antisymmetry>,
    /// The smallest sticker string in the state's orbit under the 48
    /// symmetries and every renaming of the colors, as a cube: equal for
    /// states that differ only by turning, mirroring or recoloring.
    pub canonical: Facelets,
    /// The library patterns the state is (up to turning, mirroring and
    /// recoloring); empty if it is none of them.
    pub matches: Vec<Match>,
    /// Per piece orbit: how many pieces are displaced, twisted or flipped,
    /// and the cycle type where pieces can be told apart.
    pub fingerprint: Fingerprint,
    /// Well-known subgroups the state is, or may be, in (see `subgroup`).
    pub subgroups: Vec<Membership>,
    /// How much the state looks like a pattern, 0 to 100 (see `patternness`).
    pub patternness: u8,
    /// The library patterns (three unless asked otherwise) the state is closest to (the solved cube
    /// included), closest first.
    pub closest: Vec<Similar>,
    /// The orbits in which the state equals the closest pattern that matches
    /// in some orbits and not in others.
    pub partial: Option<Partial>,
}

/// The sticker colors of a cube as numbers 0 to 5 in `URFDLB` order.
pub(crate) fn colors(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| *f as u8).collect()
}

/// Reads the structure of a cube state, any size from 2 to 7, against the
/// built-in library.
pub fn detect(cube: &Facelets) -> Report {
    detect_with(cube, &[], 3)
}

/// Like [`detect`], against the built-in library and `extra` (imported
/// patterns, named by their catalog id), and with the `closest` nearest
/// patterns listed.
pub fn detect_with(cube: &Facelets, extra: &[Pattern], closest: usize) -> Report {
    let kind = cube.kind();
    let state = colors(cube);
    let canon = canon::canonical(kind, &state);
    let symmetry = symmetry::symmetry_of(kind, &state);
    let inverse = stabilizer::inverse_colors(cube);
    let (symmetry_type, antisymmetry) =
        stabilizer::types(stabilizer::stabilizer(kind, &state, inverse.as_deref()));
    let fingerprint = fingerprint::fingerprint(kind, &state);
    let extra = library::extra_entries(kind, extra);
    let (closest, partial) = similarity::closest(kind, &state, &extra, closest);
    Report {
        closest,
        partial,
        patternness: patternness::patternness(&symmetry, &fingerprint.orbits),
        symmetry,
        symmetry_type,
        antisymmetry,
        matches: library::matches(kind, &state, &canon, &extra),
        canonical: canon::facelets(&canon),
        fingerprint,
        subgroups: subgroup::memberships(kind, &state),
    }
}

/// How much a state looks like a pattern, 0 (irregular) to 100 (the solved
/// cube). A measure of symmetry and uniformity, not a probability.
pub fn patternness(cube: &Facelets) -> u8 {
    let kind = cube.kind();
    let state = colors(cube);
    let symmetry = symmetry::symmetry_of(kind, &state);
    patternness::patternness(&symmetry, &fingerprint::fingerprint(kind, &state).orbits)
}

/// The cycles in which a move sequence carries the pieces of a cube of `kind`,
/// with the order of the permutation. Exact: pieces are followed through the
/// moves.
pub fn cycles_of_moves(kind: PuzzleKind, moves: &MoveSequence) -> Result<Cycles, String> {
    let pi = matching::of_moves(kind, moves)
        .ok_or_else(|| "a move does not exist on this cube".to_owned())?;
    Ok(cycles::analyze(kind, &pi, true))
}

/// The cycles of a cube given by its colors. On the 2x2x2 and 3x3x3 every
/// piece is told apart and the order is given; on bigger cubes equal pieces
/// are matched in index order, which reproduces the colors but defines no
/// order.
pub fn cycles_of_state(cube: &Facelets) -> Result<Cycles, String> {
    let pi = matching::of_state(cube)?;
    Ok(cycles::analyze(cube.kind(), &pi, cube.kind().size() <= 3))
}
