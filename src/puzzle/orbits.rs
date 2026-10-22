//! The orbits of the movable pieces of the larger cubes.
//!
//! A turn of a layer keeps a centre sticker among the stickers that the
//! rotations of the whole cube map it to (24 of them, four on each face), and
//! a wing among the 24 wings of the same depth along its edge. These sets are
//! the orbits; a piece never leaves its orbit, so each one can be shuffled on
//! its own to make a random state, and searched on its own.

use std::collections::HashMap;

use super::cubies::cubies;
use super::{locate, PuzzleKind};

type Vec3 = [i32; 3];

fn quarter(axis: usize, v: Vec3) -> Vec3 {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// The 24 centre-sticker orbits are found by turning a sticker through the 24
/// rotations of the cube: apply the quarter turns about two axes in every
/// combination that reaches a new orientation.
fn rotations(position: Vec3, normal: Vec3) -> Vec<(Vec3, Vec3)> {
    let mut seen = vec![(position, normal)];
    let mut at = 0;
    while at < seen.len() {
        let (p, n) = seen[at];
        for axis in 0..3 {
            let next = (quarter(axis, p), quarter(axis, n));
            if !seen.contains(&next) {
                seen.push(next);
            }
        }
        at += 1;
    }
    seen
}

/// The orbits of the centre stickers, each of 24 sticker indices in
/// increasing order, the orbits ordered by their first sticker. The sticker in
/// the middle of each face of an odd cube is fixed and belongs to none. A cube
/// of size `n` has `(n - 2)^2 / 4` of them, rounded down.
pub(crate) fn centre_orbits(kind: PuzzleKind) -> Vec<Vec<usize>> {
    let n = kind.size();
    let mut at: HashMap<(Vec3, Vec3), usize> = HashMap::new();
    let mut centres = Vec::new();
    for index in 0..kind.facelet_count() {
        let (position, normal) = locate(kind, index);
        if position.iter().filter(|c| c.abs() == n as i32 - 1).count() == 1
            && position.iter().filter(|&&c| c == 0).count() < 2
        {
            at.insert((position, normal), index);
            centres.push(index);
        }
    }
    let mut orbits: Vec<Vec<usize>> = Vec::new();
    let mut done = vec![false; kind.facelet_count()];
    for &index in &centres {
        if done[index] {
            continue;
        }
        let (position, normal) = locate(kind, index);
        let mut orbit: Vec<usize> = rotations(position, normal)
            .into_iter()
            .map(|image| at[&image])
            .collect();
        orbit.sort_unstable();
        for &member in &orbit {
            done[member] = true;
        }
        orbits.push(orbit);
    }
    orbits
}

/// The wing orbits: the wings of one depth along their edges, 24 each, as
/// the two stickers of the wing in the fixed handedness of [`cubies`]. The
/// middle edge of an odd cube is no wing. A cube of size `n` has `(n - 2) / 2`
/// orbits, rounded down.
pub(crate) fn wing_orbits(kind: PuzzleKind) -> Vec<Vec<[usize; 2]>> {
    let n = kind.size() as i32;
    let mut orbits: Vec<Vec<[usize; 2]>> = vec![Vec::new(); (kind.size() - 2) / 2];
    for cubie in cubies(kind) {
        let outer = cubie.position.iter().filter(|c| c.abs() == n - 1).count();
        if outer != 2 || cubie.stickers.len() != 2 {
            continue;
        }
        let along = cubie
            .position
            .iter()
            .find(|c| c.abs() != n - 1)
            .map_or(0, |c| c.abs());
        if along == 0 {
            continue;
        }
        let depth = ((n - 1 - along) / 2) as usize - 1;
        orbits[depth].push([cubie.stickers[0], cubie.stickers[1]]);
    }
    orbits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_orbits_of_the_large_cubes_have_the_known_sizes() {
        for (kind, centres, wings) in [
            (PuzzleKind::Cube4, 1, 1),
            (PuzzleKind::Cube5, 2, 1),
            (PuzzleKind::Cube6, 4, 2),
            (PuzzleKind::Cube7, 6, 2),
        ] {
            let c = centre_orbits(kind);
            assert_eq!(c.len(), centres, "{kind:?}");
            assert!(c.iter().all(|o| o.len() == 24));
            let w = wing_orbits(kind);
            assert_eq!(w.len(), wings, "{kind:?}");
            assert!(w.iter().all(|o| o.len() == 24));
        }
    }

    #[test]
    fn every_centre_sticker_is_in_one_orbit_with_four_per_face() {
        for kind in [PuzzleKind::Cube6, PuzzleKind::Cube7] {
            let mut all: Vec<usize> = centre_orbits(kind).concat();
            all.sort_unstable();
            all.dedup();
            let per_face = (kind.size() - 2).pow(2) - kind.size() % 2;
            assert_eq!(all.len(), 6 * per_face);
            for orbit in centre_orbits(kind) {
                for face in 0..6 {
                    let on = orbit
                        .iter()
                        .filter(|&&i| i / kind.face_len() == face)
                        .count();
                    assert_eq!(on, 4);
                }
            }
        }
    }
}
