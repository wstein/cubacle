//! The cube's symmetry group `M` (48 elements) and `M x C2` (96), their
//! subgroups and the conjugacy classes of those.
//!
//! An element is an index `g` below 48 (the position in
//! [`symmetries`](super::geometry::symmetries)) or `g + 48`, the same
//! symmetry paired with the second factor `C2`. A subgroup is a set of
//! elements, one bit each in a `u128`. Subgroups are enumerated by joining
//! with cyclic subgroups, one representative per conjugacy class (conjugating
//! by `M`; `C2` is central), so the whole work is a few thousand closures.

use std::sync::OnceLock;

use super::geometry::symmetries;
use super::typename;

/// A subgroup as a bit set over the 96 elements.
pub(super) type Set = u128;

const M: usize = 48;

/// Multiplication and inverses of `M`.
struct Table {
    mul: Vec<[u8; M]>,
    inv: [u8; M],
    /// `conj[c][e]` is `c e c^-1` for `e` in `M x C2`.
    conj: Vec<[u8; 2 * M]>,
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| {
        let all = symmetries();
        let at = |s| {
            all.iter()
                .position(|t| *t == s)
                .expect("the group is closed")
        };
        let mut mul = vec![[0u8; M]; M];
        let mut inv = [0u8; M];
        for (a, row) in mul.iter_mut().enumerate() {
            for (b, cell) in row.iter_mut().enumerate() {
                *cell = at(all[a].times(all[b])) as u8;
                if *cell == 0 {
                    inv[a] = b as u8;
                }
            }
        }
        let at = |c: usize, e: usize| {
            let by =
                |x: usize, y: usize| usize::from(mul[x % M][y % M]) + M * ((x / M + y / M) % 2);
            by(by(c, e), usize::from(inv[c])) as u8
        };
        let conj = (0..M).map(|c| std::array::from_fn(|e| at(c, e))).collect();
        Table { mul, inv, conj }
    })
}

/// The product of two elements of `M x C2`.
pub(super) fn times(x: usize, y: usize) -> usize {
    let t = table();
    usize::from(t.mul[x % M][y % M]) + M * ((x / M + y / M) % 2)
}

/// The smallest subgroup containing the generators.
pub(super) fn closure(generators: &[usize]) -> Set {
    let mut set: Set = 1;
    let mut queue = vec![0usize];
    while let Some(x) = queue.pop() {
        for &g in generators {
            let y = times(x, g);
            if set >> y & 1 == 0 {
                set |= 1 << y;
                queue.push(y);
            }
        }
    }
    set
}

pub(super) fn members(set: Set) -> impl Iterator<Item = usize> {
    (0..2 * M).filter(move |e| set >> e & 1 == 1)
}

/// `c e c^-1` for every element of the set, `c` in `M`.
fn conjugate(set: Set, c: usize) -> Set {
    let row = &table().conj[c];
    members(set).fold(0, |acc, e| acc | 1 << row[e])
}

/// The least bit set among the conjugates of the subgroup, and a `c` giving it.
fn canonical(set: Set) -> (Set, usize) {
    (0..M)
        .map(|c| (conjugate(set, c), c))
        .min()
        .expect("48 conjugators")
}

/// The conjugacy classes of subgroups of `M` or of `M x C2`.
pub(super) struct Classes {
    /// The least member of every class, in listing order.
    pub(super) reps: Vec<Set>,
    pub(super) names: Vec<String>,
}

impl Classes {
    /// The position of the class a subgroup is in.
    pub(super) fn index_of(&self, set: Set) -> usize {
        let (rep, _) = canonical(set);
        self.reps
            .iter()
            .position(|r| *r == rep)
            .expect("every subgroup is in a listed class")
    }
}

/// All classes of `M` (`extended` false: 33) or `M x C2` (true: 131), named.
pub(super) fn classes(extended: bool) -> &'static Classes {
    static PLAIN: OnceLock<Classes> = OnceLock::new();
    static EXTENDED: OnceLock<Classes> = OnceLock::new();
    if extended {
        EXTENDED.get_or_init(|| build(true))
    } else {
        PLAIN.get_or_init(|| build(false))
    }
}

fn build(extended: bool) -> Classes {
    let count = if extended { 2 * M } else { M };
    // One generator per cyclic subgroup.
    let mut cyclic: Vec<(Set, usize)> = Vec::new();
    for g in 1..count {
        let c = closure(&[g]);
        if cyclic.iter().all(|(s, _)| *s != c) {
            cyclic.push((c, g));
        }
    }
    // Each class is held as (representative, generators of it).
    let mut found: Vec<(Set, Vec<usize>)> = vec![(1, Vec::new())];
    let mut visited: std::collections::HashSet<Set> = std::collections::HashSet::new();
    let mut next = 0;
    while next < found.len() {
        let (h, gens) = found[next].clone();
        next += 1;
        for &(c, g) in &cyclic {
            if c & !h == 0 {
                continue;
            }
            let mut joined = gens.clone();
            joined.push(g);
            let raw = closure(&joined);
            if !visited.insert(raw) {
                continue;
            }
            let (rep, by) = canonical(raw);
            if found.iter().all(|(s, _)| *s != rep) {
                let inv = usize::from(table().inv[by]);
                let moved = joined.iter().map(|&e| times(times(by, e), inv)).collect();
                found.push((rep, moved));
            }
        }
    }
    let mut reps: Vec<Set> = found.into_iter().map(|(s, _)| s).collect();
    let names = typename::name_all(&mut reps, extended);
    Classes { reps, names }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m_has_33_classes_of_subgroups_and_m_times_c2_has_131() {
        assert_eq!(classes(false).reps.len(), 33);
        assert_eq!(classes(true).reps.len(), 131);
    }

    #[test]
    fn the_33_symmetry_types_and_their_orders() {
        let c = classes(false);
        let list: Vec<String> = c
            .names
            .iter()
            .zip(&c.reps)
            .map(|(n, r)| format!("{n}:{}", r.count_ones()))
            .collect();
        assert_eq!(
            list.join(" "),
            "C1:1 C2#1:2 C2#2:2 Ci:2 Cs#1:2 Cs#2:2 C3:3 C2h#1:4 C2h#2:4 C2v#1:4 C2v#2:4 \
             C2v#3:4 C4:4 D2#1:4 D2#2:4 S4:4 C3v:6 D3:6 S6:6 C4h:8 C4v:8 D2d#1:8 D2d#2:8 \
             D2h#1:8 D2h#2:8 D4:8 D3d:12 T:12 D4h:16 O:24 Td:24 Th:24 Oh:48"
        );
    }

    #[test]
    fn the_131_extended_types_hold_the_33_and_have_orders_that_divide_96() {
        let c = classes(true);
        let plain = classes(false);
        for name in &plain.names {
            assert!(c.names.contains(name), "{name}");
        }
        assert!(c.reps.iter().all(|r| 96 % r.count_ones() == 0));
        assert_eq!(c.names.last().unwrap(), "Oh+1'");
    }

    #[test]
    fn the_class_names_are_distinct() {
        for extended in [false, true] {
            let mut names = classes(extended).names.clone();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), classes(extended).reps.len());
        }
    }

    #[test]
    fn the_listed_classes_are_subgroups_and_pairwise_not_conjugate() {
        for extended in [false, true] {
            let all = classes(extended);
            for (i, &r) in all.reps.iter().enumerate() {
                let gens: Vec<usize> = members(r).collect();
                assert_eq!(closure(&gens), r);
                assert_eq!(canonical(r).0, r);
                assert_eq!(all.index_of(r), i);
            }
        }
    }
}
