//! The cubies of a cube: the stickers that sit on one small cube.

use std::collections::BTreeMap;

use super::{locate, PuzzleKind};

/// One small cube and the stickers it carries.
///
/// `stickers` are indices into the facelet string. A cubie with two stickers
/// off the middle of its edge (a wing) lists them in a fixed handedness, so
/// that turning layers, which are rotations, maps "first" to "first". The
/// other cubies list their stickers by index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cubie {
    /// Doubled cube coordinates; the outermost layer is `+-(size - 1)`.
    pub position: [i32; 3],
    pub stickers: Vec<usize>,
}

/// Every cubie of the cube, ordered by position.
pub(crate) fn cubies(kind: PuzzleKind) -> Vec<Cubie> {
    let mut by_position: BTreeMap<[i32; 3], Vec<(usize, [i32; 3])>> = BTreeMap::new();
    for index in 0..kind.facelet_count() {
        let (position, normal) = locate(kind, index);
        by_position
            .entry(position)
            .or_default()
            .push((index, normal));
    }
    by_position
        .into_iter()
        .map(|(position, mut stickers)| {
            if let [(_, a), (_, b)] = stickers[..] {
                let cross = [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ];
                let handed: i32 = (0..3).map(|k| cross[k] * position[k]).sum();
                if handed < 0 {
                    stickers.swap(0, 1);
                }
            }
            Cubie {
                position,
                stickers: stickers.into_iter().map(|(index, _)| index).collect(),
            }
        })
        .collect()
}

/// The movable pieces of a 5x5x5 by kind, as sticker indices.
///
/// With the six face centres fixed (only turns of depth 1 and 2), these are
/// the orbits the pieces stay in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FiveOrbits {
    /// The corners of each face's 3x3 centre block.
    pub x: Vec<usize>,
    /// The edge cells of each face's 3x3 centre block.
    pub t: Vec<usize>,
    /// The two wings on each edge, with stickers in the handedness of [`Cubie`].
    pub wings: Vec<[usize; 2]>,
    /// The middle edges.
    pub mids: Vec<[usize; 2]>,
}

pub(crate) fn five_orbits() -> FiveOrbits {
    let mut orbits = FiveOrbits {
        x: Vec::new(),
        t: Vec::new(),
        wings: Vec::new(),
        mids: Vec::new(),
    };
    for cubie in cubies(PuzzleKind::Cube5) {
        let outer = cubie.position.iter().filter(|c| c.abs() == 4).count();
        let mut inner: Vec<i32> = cubie
            .position
            .iter()
            .map(|c| c.abs())
            .filter(|&c| c != 4)
            .collect();
        inner.sort_unstable();
        let pair = |stickers: &[usize]| [stickers[0], stickers[1]];
        match (outer, inner.as_slice()) {
            (1, [2, 2]) => orbits.x.push(cubie.stickers[0]),
            (1, [0, 2]) => orbits.t.push(cubie.stickers[0]),
            (2, [2]) => orbits.wings.push(pair(&cubie.stickers)),
            (2, [0]) => orbits.mids.push(pair(&cubie.stickers)),
            _ => {}
        }
    }
    orbits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_5x5_has_24_pieces_per_orbit_and_12_middle_edges() {
        let orbits = five_orbits();
        assert_eq!(orbits.x.len(), 24);
        assert_eq!(orbits.t.len(), 24);
        assert_eq!(orbits.wings.len(), 24);
        assert_eq!(orbits.mids.len(), 12);
    }

    #[test]
    fn a_5x5_has_the_known_cubie_counts() {
        let all = cubies(PuzzleKind::Cube5);
        let count = |stickers: usize| all.iter().filter(|c| c.stickers.len() == stickers).count();
        assert_eq!(count(3), 8);
        assert_eq!(count(2), 36);
        assert_eq!(count(1), 54);
        assert_eq!(all.iter().map(|c| c.stickers.len()).sum::<usize>(), 150);
    }
}
