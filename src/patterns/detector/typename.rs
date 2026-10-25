//! Names for the classes of subgroups.
//!
//! A subgroup of `M` gets its Schoenflies symbol, read from how many
//! elements of each kind (determinant and trace) it has. Several classes
//! share a symbol when the same abstract group sits in the cube in different
//! orientations (a `C2` about a face axis or about an edge axis); those get a
//! `#1`, `#2`, ... after the symbol, in the order of their element kinds.
//!
//! A subgroup of `M x C2` that contains an antisymmetric element is named
//! after its image `P` in `M` and its symmetric part `K`: `P(K)` when `K` has
//! index 2 in `P`, and `P+1'` when the subgroup is `P x C2` (the cube is its
//! own inverse as well).

use std::collections::BTreeMap;

use super::geometry::symmetries;
use super::group::{classes, members, Set};

const M: usize = 48;
const LOW: Set = (1 << M) - 1;

/// Counts of identity, inversion, mirror, `S4`, `S6`, half turn, quarter
/// turn and third turn elements, and the Schoenflies symbol they spell.
fn schoenflies(set: Set) -> &'static str {
    let (mut inv, mut mir, mut s4, mut s6, mut c2, mut c4, mut c3) = (0, 0, 0, 0, 0, 0, 0);
    let mut order = 0;
    for e in members(set & LOW) {
        order += 1;
        match symmetries()[e].kind_key() {
            (-1, -3) => inv += 1,
            (-1, 1) => mir += 1,
            (-1, -1) => s4 += 1,
            (-1, 0) => s6 += 1,
            (1, -1) => c2 += 1,
            (1, 1) => c4 += 1,
            (1, 0) => c3 += 1,
            _ => {}
        }
    }
    match (order, inv, mir, s4, s6, c2, c4, c3) {
        (1, ..) => "C1",
        (2, 0, 1, ..) => "Cs",
        (2, _, 0, 0, 0, 1, ..) => "C2",
        (2, 1, ..) => "Ci",
        (3, ..) => "C3",
        (4, 0, 2, 0, 0, 1, ..) => "C2v",
        (4, 1, 1, 0, 0, 1, ..) => "C2h",
        (4, 0, 0, 0, 0, 3, ..) => "D2",
        (4, 0, 0, 0, 0, 1, 2, _) => "C4",
        (4, _, _, 2, ..) => "S4",
        (6, 1, ..) => "S6",
        (6, 0, 3, ..) => "C3v",
        (6, 0, 0, ..) => "D3",
        (8, 1, 3, 0, 0, 3, ..) => "D2h",
        (8, 0, 4, ..) => "C4v",
        (8, 0, 2, 2, ..) => "D2d",
        (8, 1, 1, 2, ..) => "C4h",
        (8, 0, 0, ..) => "D4",
        (12, 0, ..) => "T",
        (12, ..) => "D3d",
        (16, ..) => "D4h",
        (24, 1, ..) => "Th",
        (24, 0, _, 6, ..) => "Td",
        (24, ..) => "O",
        _ => "Oh",
    }
}

/// What tells apart two classes with the same symbol: how many elements of
/// each geometric class the subgroup has, per second-factor value.
type Descriptor = Vec<(usize, &'static str, usize)>;

fn descriptor(set: Set) -> Descriptor {
    let mut counts: BTreeMap<(usize, &str), usize> = BTreeMap::new();
    for e in members(set) {
        *counts
            .entry((e / M, symmetries()[e % M].class()))
            .or_default() += 1;
    }
    counts.into_iter().map(|((a, c), n)| (a, c, n)).collect()
}

/// Sorts the classes by order, symbol and element kinds, and returns their
/// names in that order.
pub(super) fn name_all(reps: &mut Vec<Set>, extended: bool) -> Vec<String> {
    let mut keyed: Vec<(usize, String, Descriptor, Set)> = reps
        .iter()
        .map(|&s| {
            let base = if extended && s >> M != 0 {
                extended_base(s)
            } else {
                schoenflies(s).to_owned()
            };
            (s.count_ones() as usize, base, descriptor(s), s)
        })
        .collect();
    keyed.sort();
    let mut names = Vec::new();
    for (i, (_, base, _, _)) in keyed.iter().enumerate() {
        let same = keyed.iter().filter(|k| k.1 == *base).count();
        let before = keyed[..i].iter().filter(|k| k.1 == *base).count();
        names.push(if same > 1 {
            format!("{base}#{}", before + 1)
        } else {
            base.clone()
        });
    }
    *reps = keyed.into_iter().map(|k| k.3).collect();
    names
}

/// `P(K)` or `P+1'` for a subgroup with antisymmetric elements.
fn extended_base(set: Set) -> String {
    let symmetric = set & LOW;
    let image = symmetric | set >> M;
    let plain = classes(false);
    let name = |s: Set| plain.names[plain.index_of(s)].clone();
    if set & (1 << M) != 0 {
        format!("{}+1'", name(image))
    } else {
        format!("{}({})", name(image), name(symmetric))
    }
}

/// The name of a subgroup of `M` or `M x C2` (`set` any, not only a
/// representative).
pub(super) fn name_of(set: Set) -> String {
    if set >> M == 0 {
        let c = classes(false);
        return c.names[c.index_of(set)].clone();
    }
    let c = classes(true);
    c.names[c.index_of(set)].clone()
}
