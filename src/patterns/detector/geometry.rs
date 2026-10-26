//! The 48 symmetries of the cube as sticker permutations.
//!
//! A symmetry is a signed permutation matrix (24 rotations and their 24
//! mirror images). It sends every sticker to another sticker of the same
//! cube, so for each size it is a permutation of the sticker indices, built
//! once from the positions that `puzzle::locate` gives.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::puzzle::{locate, PuzzleKind};

pub(crate) type Vec3 = [i32; 3];

/// A signed permutation matrix, row-major.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Sym {
    m: [[i32; 3]; 3],
}

impl Sym {
    pub(crate) fn apply(self, v: Vec3) -> Vec3 {
        let mut out = [0; 3];
        for (o, row) in out.iter_mut().zip(self.m) {
            *o = row.iter().zip(v).map(|(a, b)| a * b).sum();
        }
        out
    }

    fn det(self) -> i32 {
        let m = self.m;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    fn trace(self) -> i32 {
        self.m[0][0] + self.m[1][1] + self.m[2][2]
    }

    /// The product `self * other` (apply `other` first).
    pub(crate) fn times(self, other: Sym) -> Sym {
        let mut m = [[0; 3]; 3];
        for (i, row) in m.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| self.m[i][k] * other.m[k][j]).sum();
            }
        }
        Sym { m }
    }

    /// `(determinant, trace)`: together they tell the kind of operation.
    pub(crate) fn kind_key(self) -> (i32, i32) {
        (self.det(), self.trace())
    }

    pub(crate) fn is_mirror(self) -> bool {
        self.det() < 0
    }

    /// The class of the operation: its name without the axis.
    pub(crate) fn class(self) -> &'static str {
        match (self.det(), self.trace()) {
            (1, 3) => "identity",
            (1, 1) => "quarter turn about a face axis",
            (1, -1) => {
                if self.m.iter().enumerate().all(|(i, row)| row[i] != 0) {
                    "half turn about a face axis"
                } else {
                    "half turn about an edge axis"
                }
            }
            (1, _) => "third turn about a corner axis",
            (_, -3) => "inversion through the center",
            (_, 1) => {
                if self.m.iter().enumerate().all(|(i, row)| row[i] != 0) {
                    "mirror parallel to a face"
                } else {
                    "mirror through opposite edges"
                }
            }
            (_, -1) => "quarter turn and mirror",
            _ => "sixth turn and mirror",
        }
    }
}

/// All 48 signed permutation matrices, the identity first.
pub(crate) fn symmetries() -> &'static [Sym; 48] {
    static ALL: OnceLock<[Sym; 48]> = OnceLock::new();
    ALL.get_or_init(|| {
        const PERMS: [[usize; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let mut out = Vec::with_capacity(48);
        for perm in PERMS {
            for signs in 0..8 {
                let mut m = [[0; 3]; 3];
                for (row, &column) in perm.iter().enumerate() {
                    m[row][column] = if signs >> row & 1 == 1 { -1 } else { 1 };
                }
                out.push(Sym { m });
            }
        }
        let identity = out
            .iter()
            .position(|s| s.m == [[1, 0, 0], [0, 1, 0], [0, 0, 1]])
            .expect("the identity is a signed permutation");
        out.swap(0, identity);
        out.try_into().expect("48 signed permutation matrices")
    })
}

/// A cube size's symmetries as permutations of sticker indices.
pub(crate) struct Tables {
    /// `perm[g][i]` is the sticker that sticker `i` goes to under symmetry `g`.
    pub(crate) perm: Vec<Vec<u16>>,
}

/// The tables of a size, built on first use.
pub(crate) fn tables(kind: PuzzleKind) -> &'static Tables {
    static CELLS: [OnceLock<Tables>; 6] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    CELLS[kind.size() - 2].get_or_init(|| build(kind))
}

fn build(kind: PuzzleKind) -> Tables {
    let count = kind.facelet_count();
    let located: Vec<(Vec3, Vec3)> = (0..count).map(|i| locate(kind, i)).collect();
    let at: HashMap<(Vec3, Vec3), usize> = located
        .iter()
        .enumerate()
        .map(|(i, place)| (*place, i))
        .collect();
    let mut perm = Vec::with_capacity(48);
    for sym in symmetries() {
        perm.push(
            located
                .iter()
                .map(|&(p, n)| at[&(sym.apply(p), sym.apply(n))] as u16)
                .collect(),
        );
    }
    Tables { perm }
}

/// The image of `state` under symmetry `g`: the sticker at `i` goes to `perm[i]`.
pub(crate) fn image(kind: PuzzleKind, state: &[u8], g: usize) -> Vec<u8> {
    let mut out = vec![0; state.len()];
    for (&to, &color) in tables(kind).perm[g].iter().zip(state) {
        out[to as usize] = color;
    }
    out
}

const FACE_NAMES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

/// The axis of a symmetry about which it turns or in whose normal it mirrors,
/// written as the pair of opposite cube features it joins (`U/D`, `UF/DB`,
/// `UFR/DBL`); empty for the identity and the inversion.
pub(crate) fn axis_name(sym: Sym) -> String {
    // The axis of a rotation is fixed by it; a mirror's normal is reversed.
    let rotation = if sym.is_mirror() { negate(sym) } else { sym };
    let fixed: Vec<Vec3> = (-1..=1)
        .flat_map(|x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| [x, y, z])))
        .filter(|v| *v != [0, 0, 0] && rotation.apply(*v) == *v)
        .collect();
    // Of the fixed directions take the longest (the axis, not a plane point).
    let Some(best) = fixed
        .iter()
        .max_by_key(|v| v.iter().filter(|c| **c != 0).count())
    else {
        return String::new();
    };
    if fixed.len() > 2 {
        // Identity or inversion: every direction is fixed (or reversed).
        return String::new();
    }
    let name = |v: Vec3| -> String {
        let mut text = String::new();
        // Letters in U/D, F/B, R/L order.
        for (axis, positive, negative) in [(1, 'U', 'D'), (2, 'F', 'B'), (0, 'R', 'L')] {
            match v[axis] {
                1 => text.push(positive),
                -1 => text.push(negative),
                _ => {}
            }
        }
        text
    };
    let mut plus = *best;
    let first = [1, 2, 0].into_iter().find(|&a| plus[a] != 0).unwrap_or(0);
    if plus[first] < 0 {
        plus = plus.map(|c| -c);
    }
    let minus = plus.map(|c| -c);
    format!("{}/{}", name(plus), name(minus))
}

fn negate(sym: Sym) -> Sym {
    Sym {
        m: sym.m.map(|row| row.map(|c| -c)),
    }
}

/// The letter of a face index.
pub(crate) fn face_letter(face: usize) -> char {
    FACE_NAMES[face]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_48_distinct_symmetries_with_24_rotations() {
        let all = symmetries();
        for (i, a) in all.iter().enumerate() {
            assert!(all[i + 1..].iter().all(|b| a != b));
        }
        assert_eq!(all.iter().filter(|s| !s.is_mirror()).count(), 24);
        assert_eq!(all[0].class(), "identity");
    }

    #[test]
    fn symmetries_permute_the_stickers_of_every_size() {
        for kind in PuzzleKind::ALL {
            for perm in &tables(kind).perm {
                let mut seen = vec![false; kind.facelet_count()];
                for &to in perm {
                    assert!(!seen[to as usize]);
                    seen[to as usize] = true;
                }
            }
        }
    }

    #[test]
    fn the_class_sizes_are_the_known_ones() {
        let mut counts = std::collections::BTreeMap::new();
        for sym in symmetries() {
            *counts.entry(sym.class()).or_insert(0) += 1;
        }
        assert_eq!(counts["identity"], 1);
        assert_eq!(counts["quarter turn about a face axis"], 6);
        assert_eq!(counts["half turn about a face axis"], 3);
        assert_eq!(counts["half turn about an edge axis"], 6);
        assert_eq!(counts["third turn about a corner axis"], 8);
        assert_eq!(counts["inversion through the center"], 1);
        assert_eq!(counts["mirror parallel to a face"], 3);
        assert_eq!(counts["mirror through opposite edges"], 6);
        assert_eq!(counts["quarter turn and mirror"], 6);
        assert_eq!(counts["sixth turn and mirror"], 8);
    }
}
