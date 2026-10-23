//! The macros of the pattern shortcut: classic 3x3x3 pattern algorithms, each
//! turned into a family for a bigger cube.
//!
//! A macro is a base algorithm with every face turn replaced by turns of the
//! same face on a chosen set of layers (layers 1 to 3 counted from that face;
//! layer 1 alone is the 3x3x3 algorithm itself), seen from one of the 24
//! orientations of the cube. Macros that move the stickers the same way are
//! kept once, the shorter one.

use std::collections::HashMap;

use crate::puzzle::{destinations, Face, Move, MoveSequence, PuzzleKind, Turn};

/// Names and algorithms of the bases (3x3x3 notation, outer turns only).
pub(super) const BASES: [(&str, &str); 6] = [
    ("checkerboard", "R2 L2 U2 D2 F2 B2"),
    ("six spot", "U D' R L' F B' U D'"),
    ("pons asinorum", "F2 B2 U D' R2 L2 U D'"),
    ("tetris", "L R F B U' D' L' R'"),
    ("cube in cube", "F L F U' R U F2 L2 U' L' B D' B' L2 U"),
    (
        "cube in cube, long",
        "U' L' U' F' R2 B' R F U B2 U B' L U' F U R F'",
    ),
];

/// How many layers from a face the macros may use.
const MAX_LAYER: usize = 3;

/// One macro: the plain moves and where they send every sticker.
pub(super) struct Macro {
    pub name: String,
    pub moves: Vec<Move>,
    /// Sticker `i` ends up at `dest[i]`.
    pub dest: Vec<u16>,
    /// The macro that undoes this one.
    pub inverse: usize,
}

pub(super) struct Library {
    pub kind: PuzzleKind,
    pub macros: Vec<Macro>,
}

fn invert(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    }
}

fn inverse_moves(moves: &[Move]) -> Vec<Move> {
    moves
        .iter()
        .rev()
        .map(|mv| Move::new(mv.face, mv.depth, mv.wide, invert(mv.turn)))
        .collect()
}

/// The 24 rotations of the cube as maps from face index to face index.
fn rotations() -> Vec<[usize; 6]> {
    // Face order U R F D L B: a quarter turn about each of two axes.
    let about_x: [usize; 6] = [2, 1, 3, 5, 4, 0];
    let about_y: [usize; 6] = [0, 5, 1, 3, 2, 4];
    let mut all: Vec<[usize; 6]> = vec![[0, 1, 2, 3, 4, 5]];
    let mut next = 0;
    while next < all.len() {
        let current = all[next];
        for step in [about_x, about_y] {
            let mut turned = [0; 6];
            for (face, slot) in turned.iter_mut().enumerate() {
                *slot = step[current[face]];
            }
            if !all.contains(&turned) {
                all.push(turned);
            }
        }
        next += 1;
    }
    all
}

fn tokens(text: &str) -> Vec<(Face, Turn)> {
    text.split_whitespace()
        .map(|token| {
            let face = Face::from_letter(token.chars().next().expect("a token"))
                .expect("base algorithms use face letters");
            let turn = Turn::from_suffix(&token[1..]).expect("base algorithms use valid turns");
            (face, turn)
        })
        .collect()
}

/// Where the moves send every sticker, in order.
fn destination_of(kind: PuzzleKind, moves: &[Move]) -> Vec<u16> {
    let mut total: Vec<u16> = (0..kind.facelet_count() as u16).collect();
    for mv in moves {
        let step = destinations(kind, mv);
        for slot in total.iter_mut() {
            *slot = step[usize::from(*slot)] as u16;
        }
    }
    total
}

impl Library {
    pub(super) fn build(kind: PuzzleKind) -> Library {
        let layers = MAX_LAYER.min(kind.size() - 1);
        let mut macros: Vec<Macro> = Vec::new();
        let mut seen: HashMap<Vec<u16>, usize> = HashMap::new();
        let mut add = |name: String, moves: Vec<Move>, macros: &mut Vec<Macro>| {
            let dest = destination_of(kind, &moves);
            match seen.get(&dest) {
                Some(&index) if macros[index].moves.len() <= moves.len() => {}
                Some(&index) => {
                    macros[index].name = name;
                    macros[index].moves = moves;
                }
                None => {
                    seen.insert(dest.clone(), macros.len());
                    macros.push(Macro {
                        name,
                        moves,
                        dest,
                        inverse: usize::MAX,
                    });
                }
            }
        };
        let orientations = rotations();
        for (name, text) in BASES {
            let base = tokens(text);
            for set in 1u32..(1 << layers) {
                let used: Vec<u8> = (0..layers as u8)
                    .filter(|layer| set >> layer & 1 == 1)
                    .map(|layer| layer + 1)
                    .collect();
                for turned in &orientations {
                    let forward: Vec<Move> = base
                        .iter()
                        .flat_map(|&(face, turn)| {
                            let face = Face::ALL[turned[face.index()]];
                            used.iter().map(move |&l| Move::new(face, l, false, turn))
                        })
                        .collect();
                    let label = format!("{name} on layers {used:?}");
                    add(
                        format!("{label}, undone"),
                        inverse_moves(&forward),
                        &mut macros,
                    );
                    add(label, forward, &mut macros);
                }
            }
        }
        let index: HashMap<Vec<u16>, usize> = macros
            .iter()
            .enumerate()
            .map(|(i, m)| (m.dest.clone(), i))
            .collect();
        for entry in macros.iter_mut() {
            let dest = destination_of(kind, &inverse_moves(&entry.moves));
            entry.inverse = *index.get(&dest).expect("every macro has its inverse");
        }
        Library { kind, macros }
    }

    /// The plain moves of a product of macros, shortened by merging turns of
    /// one axis (see [`crate::ranges`]).
    pub(super) fn sequence(&self, path: &[usize]) -> MoveSequence {
        let raw: Vec<Move> = path
            .iter()
            .flat_map(|&m| self.macros[m].moves.iter().copied())
            .collect();
        let packed = crate::ranges::compress(self.kind, &MoveSequence::new(raw));
        crate::ranges::expand(&packed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_24_rotations() {
        assert_eq!(rotations().len(), 24);
    }

    #[test]
    fn every_macro_has_an_inverse() {
        let library = Library::build(PuzzleKind::Cube5);
        assert!(library.macros.len() > 100);
        let identity: Vec<u16> = (0..PuzzleKind::Cube5.facelet_count() as u16).collect();
        for (i, m) in library.macros.iter().enumerate() {
            let there_and_back: Vec<u16> = m
                .dest
                .iter()
                .map(|&d| library.macros[m.inverse].dest[usize::from(d)])
                .collect();
            assert_eq!(there_and_back, identity, "macro {i}");
        }
    }
}
