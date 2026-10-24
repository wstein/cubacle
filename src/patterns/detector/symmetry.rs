//! Which of the 48 cube symmetries leave a state unchanged, up to a
//! permutation of the six colours.
//!
//! Turning (or mirroring) the cube moves every sticker, and the colour of a
//! sticker may be renamed on the way. A symmetry `g` fixes a state `s` when
//! some colour permutation `t` gives `s[g(i)] = t(s[i])` for every sticker
//! `i`. When `t` is the identity the state is fixed literally; otherwise it
//! is fixed only after the colours are renamed (a superflip stays a superflip
//! when the cube is turned, but the sticker that was F-coloured on U is
//! R-coloured after a quarter turn about U/D).

use super::geometry::{axis_name, face_letter, symmetries, tables};
use crate::puzzle::PuzzleKind;

/// A colour permutation: `t[c]` is the colour that `c` becomes.
pub(crate) type Colours = [u8; 6];

/// `t` with `a[perm[i]] == t[b[i]]` for every sticker, if there is one.
pub(crate) fn colour_map(a: &[u8], b: &[u8], perm: &[u16]) -> Option<Colours> {
    let mut map = [u8::MAX; 6];
    for (i, &to) in perm.iter().enumerate() {
        let (from, into) = (b[i] as usize, a[to as usize]);
        if map[from] == u8::MAX {
            map[from] = into;
        } else if map[from] != into {
            return None;
        }
    }
    let mut used = [false; 6];
    for &c in &map {
        if c == u8::MAX || std::mem::replace(&mut used[c as usize], true) {
            return None;
        }
    }
    Some(map)
}

/// One symmetry that fixes the state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// The kind of operation, such as `half turn about a face axis`.
    pub class: &'static str,
    /// The axis it turns about or mirrors across (`U/D`, `UF/DB`, `UFR/DBL`),
    /// empty for the identity and the inversion.
    pub axis: String,
    /// Whether the operation is a mirror image.
    pub mirror: bool,
    /// The renaming of colours that makes the operation fix the state, as
    /// cycles such as `(U D)(R L)`; empty if no colour changes.
    pub colour_swap: String,
}

impl Operation {
    /// Whether the state is fixed without renaming any colour.
    pub fn keeps_colours(&self) -> bool {
        self.colour_swap.is_empty()
    }
}

/// The symmetries that fix a state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symmetry {
    pub operations: Vec<Operation>,
}

impl Symmetry {
    /// The order of the symmetry group of the state (1 to 48).
    pub fn order(&self) -> usize {
        self.operations.len()
    }

    /// How many of the operations are rotations (the others are mirrored).
    pub fn rotations(&self) -> usize {
        self.operations.iter().filter(|o| !o.mirror).count()
    }

    /// How many fix the state with no colour renamed.
    pub fn colour_preserving(&self) -> usize {
        self.operations.iter().filter(|o| o.keeps_colours()).count()
    }
}

/// The cycles of a colour permutation, `(U D)(R L)`; empty for the identity.
fn cycles(map: &Colours) -> String {
    let mut seen = [false; 6];
    let mut text = String::new();
    for start in 0..6 {
        if seen[start] || map[start] as usize == start {
            continue;
        }
        let mut at = start;
        text.push('(');
        while !seen[at] {
            seen[at] = true;
            text.push(face_letter(at));
            at = map[at] as usize;
            if !seen[at] {
                text.push(' ');
            }
        }
        text.push(')');
    }
    text
}

/// The symmetries of `state` (colours 0 to 5 in URFDLB order).
pub(crate) fn symmetry_of(kind: PuzzleKind, state: &[u8]) -> Symmetry {
    let tables = tables(kind);
    let mut operations = Vec::new();
    for (g, sym) in symmetries().iter().enumerate() {
        let Some(map) = colour_map(state, state, &tables.perm[g]) else {
            continue;
        };
        operations.push(Operation {
            class: sym.class(),
            axis: axis_name(*sym),
            mirror: sym.is_mirror(),
            colour_swap: cycles(&map),
        });
    }
    Symmetry { operations }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Facelets;

    fn colours(cube: &Facelets) -> Vec<u8> {
        cube.stickers().iter().map(|f| *f as u8).collect()
    }

    #[test]
    fn the_solved_cube_has_all_48_symmetries_and_only_the_identity_keeps_colours() {
        for kind in PuzzleKind::ALL {
            let s = symmetry_of(kind, &colours(&Facelets::solved(kind)));
            assert_eq!(s.order(), 48);
            assert_eq!(s.colour_preserving(), 1);
            assert_eq!(s.rotations(), 24);
        }
    }

    #[test]
    fn the_group_is_the_same_for_every_turn_and_renaming_of_the_cube() {
        use super::super::geometry::image;
        for kind in PuzzleKind::ALL {
            let moves = if kind == PuzzleKind::Cube2 {
                "R U F'"
            } else {
                "R U F' L2 D 2R"
            };
            let cube = Facelets::solved(kind)
                .apply(&moves.parse().unwrap())
                .unwrap();
            let state = colours(&cube);
            let order = symmetry_of(kind, &state).order();
            for g in 0..48 {
                let turned = image(kind, &state, g);
                assert_eq!(symmetry_of(kind, &turned).order(), order);
                let renamed: Vec<u8> = turned.iter().map(|&c| (c * 5 + 1) % 6).collect();
                assert_eq!(symmetry_of(kind, &renamed).order(), order);
            }
        }
    }

    #[test]
    fn a_single_turn_has_a_smaller_group() {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R".parse().unwrap())
            .unwrap();
        let s = symmetry_of(PuzzleKind::Cube3, &colours(&cube));
        assert!(s.order() < 48);
        assert!(48 % s.order() == 0);
    }
}
