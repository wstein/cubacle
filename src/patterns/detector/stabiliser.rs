//! The symmetry type of a state: which conjugacy class of subgroups of `M`
//! (symmetry) and of `M x C2` (symmetry and antisymmetry) its stabiliser is.
//!
//! A symmetry `g` fixes a state `s` when `s` after `g` is `s` with the colours
//! renamed. It is an antisymmetry when `s` after `g` is the inverse cube of
//! `s`, again up to a renaming of the colours. The inverse is defined by the
//! colours on the 2x2x2 and the 3x3x3, where every piece can be told apart;
//! on bigger cubes equal pieces make it ambiguous, so only symmetry is typed.

use super::geometry::{symmetries, tables};
use super::group::{closure, members, Set};
use super::symmetry::colour_map;
use super::typename::name_of;
use crate::puzzle::{Facelets, PuzzleKind};

/// The symmetry type of a state, such as `D4h` of order 16.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymmetryType {
    /// The Schoenflies symbol, with `#1`, `#2`, ... when one symbol covers
    /// several orientations in the cube.
    pub name: String,
    /// The number of cube symmetries that fix the state.
    pub order: usize,
}

/// The symmetry and antisymmetry type of a state with antisymmetries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Antisymmetry {
    /// `P(K)`: the group `P` of symmetries and antisymmetries, over its
    /// symmetric half `K`; `P+1'` if the cube is also its own inverse.
    pub name: String,
    /// The number of operations (symmetries and antisymmetries together).
    pub order: usize,
    /// How many of them are symmetries.
    pub symmetries: usize,
}

/// The inverse of a 3x3x3 as sticker colours, if the colours are a cube's.
pub(crate) fn inverse_colours(cube: &Facelets) -> Option<Vec<u8>> {
    let kind = cube.kind();
    if kind.size() != 3 {
        return None;
    }
    let pi = super::matching::of_state(cube).ok()?;
    let face_len = kind.size() * kind.size();
    Some(pi.iter().map(|&p| (p / face_len) as u8).collect())
}

/// The elements of `M x C2` that fix the state: `g` for a symmetry, `g + 48`
/// for an antisymmetry (when the inverse colours are given).
pub(crate) fn stabiliser(kind: PuzzleKind, state: &[u8], inverse: Option<&[u8]>) -> Set {
    let mut set: Set = 0;
    for (g, perm) in tables(kind).perm.iter().enumerate() {
        if colour_map(state, state, perm).is_some() {
            set |= 1 << g;
        }
        if let Some(inv) = inverse {
            if colour_map(state, inv, perm).is_some() {
                set |= 1 << (g + symmetries().len());
            }
        }
    }
    set
}

/// Whether the set is closed under products (a subgroup of `M x C2`).
fn is_subgroup(set: Set) -> bool {
    let gens: Vec<usize> = members(set).collect();
    closure(&gens) == set
}

/// The types, from the stabiliser (symmetry only if it is not trivial).
pub(crate) fn types(set: Set) -> (Option<SymmetryType>, Option<Antisymmetry>) {
    let low = set & ((1 << 48) - 1);
    let symmetry = (low.count_ones() > 1).then(|| SymmetryType {
        name: name_of(low),
        order: low.count_ones() as usize,
    });
    let anti = (set >> 48 != 0 && is_subgroup(set)).then(|| Antisymmetry {
        name: name_of(set),
        order: set.count_ones() as usize,
        symmetries: low.count_ones() as usize,
    });
    (symmetry, anti)
}

#[cfg(test)]
mod tests {
    use super::super::geometry::image;
    use super::*;
    use crate::patterns::detector::colours;

    fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
        Facelets::solved(kind)
            .apply(&moves.parse().unwrap())
            .unwrap()
    }

    fn of(cube: &Facelets) -> (Option<SymmetryType>, Option<Antisymmetry>) {
        let state = colours(cube);
        let inverse = inverse_colours(cube);
        types(stabiliser(cube.kind(), &state, inverse.as_deref()))
    }

    #[test]
    fn the_solved_cube_has_the_whole_group_and_is_its_own_inverse() {
        let (s, a) = of(&Facelets::solved(PuzzleKind::Cube3));
        assert_eq!(s.unwrap().order, 48);
        let a = a.unwrap();
        assert_eq!((a.order, a.symmetries), (96, 48));
        assert_eq!(a.name, "Oh+1'");
    }

    #[test]
    fn a_random_looking_cube_has_a_trivial_stabiliser() {
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3] {
            let c = cube(kind, "R U2 F' L D R' B U F2 L'");
            let state = colours(&c);
            let set = stabiliser(kind, &state, inverse_colours(&c).as_deref());
            // Only the identity can fix such a state.
            assert_eq!(types(set), (None, None), "{kind:?}");
        }
    }

    #[test]
    fn library_patterns_have_their_known_types() {
        use crate::patterns;
        let of_pattern = |id: &str| {
            let c = patterns::find(id)
                .unwrap()
                .state(PuzzleKind::Cube3)
                .unwrap();
            of(&c)
        };
        let named = |t: (Option<SymmetryType>, Option<Antisymmetry>)| {
            (
                t.0.map(|s| (s.name, s.order)),
                t.1.map(|a| (a.name, a.order)),
            )
        };
        let (s, a) = named(of_pattern("checkerboard"));
        assert_eq!(s, Some(("Oh".to_owned(), 48)));
        assert_eq!(a, Some(("Oh+1'".to_owned(), 96)));
        let (s, a) = named(of_pattern("four-spots"));
        assert_eq!(s, Some(("D4h".to_owned(), 16)));
        assert_eq!(a, Some(("D4h+1'".to_owned(), 32)));
        let (s, a) = named(of_pattern("six-spots"));
        assert_eq!(s, Some(("S6".to_owned(), 6)));
        assert_eq!(a, Some(("D3d(S6)".to_owned(), 12)));
    }

    #[test]
    fn the_stabiliser_is_a_subgroup_and_conjugating_keeps_its_type() {
        for kind in [PuzzleKind::Cube3] {
            for moves in [
                "R",
                "R2 L2",
                "U2 D2 R2 L2 F2 B2",
                "R U R' U'",
                "F2 B2",
                "R L'",
            ] {
                let c = cube(kind, moves);
                let state = colours(&c);
                let inverse = inverse_colours(&c).unwrap();
                let set = stabiliser(kind, &state, Some(&inverse));
                assert!(is_subgroup(set), "{moves}");
                let expect = types(set);
                for g in 0..48 {
                    // Conjugating a cube by a symmetry turns its picture and
                    // its inverse's picture the same way.
                    let turned = image(kind, &state, g);
                    let turned_inverse = image(kind, &inverse, g);
                    let again = types(stabiliser(kind, &turned, Some(&turned_inverse)));
                    assert_eq!(again, expect, "{moves} after symmetry {g}");
                }
            }
        }
    }

    #[test]
    fn big_cubes_get_a_symmetry_type_that_survives_turning_and_no_antisymmetry() {
        for kind in [
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            let c = cube(kind, "R2 L2");
            let state = colours(&c);
            let (s, a) = types(stabiliser(kind, &state, None));
            assert_eq!(a, None);
            let s = s.expect("R2 L2 is symmetric");
            assert_eq!((s.name.as_str(), s.order), ("D4h", 16), "{kind:?}");
            for g in 0..48 {
                let turned = image(kind, &state, g);
                assert_eq!(types(stabiliser(kind, &turned, None)).0, Some(s.clone()));
            }
        }
    }
}
