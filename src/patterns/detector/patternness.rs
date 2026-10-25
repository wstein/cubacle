//! How much a state looks like a pattern, as a number from 0 to 100.
//!
//! It is a measure of regularity, not a probability: a weighted mean of
//!
//! - **symmetry** `S`: `(ln order + ln(colour-keeping) / 2) / ln 2`, at most 1,
//!   where `order` is the number of the 48 operations that fix the state up to
//!   renaming colours and the colour-keeping ones count extra (any symmetry
//!   beyond the identity is already rare: a random cube has none);
//! - **uniformity** `U`: the mean over the piece orbits of how regular the
//!   orbit is: the displaced count, and the twisted or flipped count, are a
//!   simple share of the orbit (none, 1/4, 1/3, 1/2, 2/3, 3/4, 1/6, 5/6 or
//!   all), and the cycles of the pieces, where known, all have one length.
//!
//! `patternness = 100 * (0.6 S + 0.4 U)`, rounded. The solved cube is 100.

use super::fingerprint::OrbitStat;
use super::symmetry::Symmetry;

/// The version of the formula, for stored values.
pub const PATTERNNESS_VERSION: &str = "patternness-v1";

const SYMMETRY_WEIGHT: f64 = 0.6;
const FULL_SYMMETRY: f64 = 2.0;

fn simple_share(count: usize, of: usize) -> bool {
    of == 0 || [1, 2, 3, 4, 6].iter().any(|d| (count * d) % of == 0)
}

fn orbit_regularity(orbit: &OrbitStat) -> f64 {
    let mut parts = vec![simple_share(orbit.displaced, orbit.pieces)];
    if orbit.tag.is_none() {
        parts.push(simple_share(orbit.oriented_wrong, orbit.pieces));
    }
    if let Some(cycles) = &orbit.cycles {
        if !cycles.is_empty() {
            parts.push(cycles.iter().all(|&c| c == cycles[0]));
        }
    }
    parts.iter().filter(|&&p| p).count() as f64 / parts.len() as f64
}

/// The symmetry part, 0 to 1.
pub(crate) fn symmetry_score(symmetry: &Symmetry) -> f64 {
    let order = symmetry.order().max(1) as f64;
    let keep = symmetry.colour_preserving().max(1) as f64;
    ((order.ln() + 0.5 * keep.ln()) / FULL_SYMMETRY.ln()).min(1.0)
}

/// The uniformity part, 0 to 1.
pub(crate) fn uniformity_score(orbits: &[OrbitStat]) -> f64 {
    if orbits.is_empty() {
        return 1.0;
    }
    orbits.iter().map(orbit_regularity).sum::<f64>() / orbits.len() as f64
}

/// The patternness, 0 to 100.
pub(crate) fn patternness(symmetry: &Symmetry, orbits: &[OrbitStat]) -> u8 {
    let score = SYMMETRY_WEIGHT * symmetry_score(symmetry)
        + (1.0 - SYMMETRY_WEIGHT) * uniformity_score(orbits);
    (100.0 * score).round() as u8
}
