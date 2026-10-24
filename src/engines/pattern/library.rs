//! The macros of the pattern search: the base algorithms of the pattern
//! registry (`crate::patterns::bases`), each turned into a family for a bigger
//! cube.
//!
//! A macro is a base algorithm with every face turn replaced by turns of the
//! same face on a chosen set of layers (layers 1 to 3 counted from that face;
//! layer 1 alone is the 3x3x3 algorithm itself), seen from one of the 24
//! orientations of the cube. Macros that move the stickers the same way are
//! kept once, the shorter one. Every macro is a [`Step`], so the macro search
//! and the generator search of this module move stickers the same way.

use std::collections::HashMap;

use super::bidir::Step;
use super::steps::step_of;
use crate::patterns::bases;
use crate::puzzle::{Face, LayerMove, Move, MoveSequence, PuzzleKind, Turn};

/// How many layers from a face the macros may use.
const MAX_LAYER: usize = 3;

/// One macro: the plain moves and the sticker permutation they make.
pub(super) struct Macro {
    pub name: String,
    pub moves: Vec<Move>,
    /// The permutation (`step.forward[i]` is the sticker that lands on `i`).
    pub step: Step,
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

/// The permutation the moves make on a cube of `kind`.
fn step_of_moves(kind: PuzzleKind, moves: &[Move]) -> Step {
    let layers: Vec<LayerMove> = moves.iter().copied().map(LayerMove::from).collect();
    step_of(kind.size(), &layers)
}

impl Library {
    pub(super) fn build(kind: PuzzleKind) -> Library {
        let layers = MAX_LAYER.min(kind.size() - 1);
        let mut macros: Vec<Macro> = Vec::new();
        let mut seen: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut add = |name: String, moves: Vec<Move>, macros: &mut Vec<Macro>| {
            let step = step_of_moves(kind, &moves);
            match seen.get(&step.forward) {
                Some(&index) if macros[index].moves.len() <= moves.len() => {}
                Some(&index) => {
                    macros[index].name = name;
                    macros[index].moves = moves;
                }
                None => {
                    seen.insert(step.forward.clone(), macros.len());
                    macros.push(Macro {
                        name,
                        moves,
                        step,
                        inverse: usize::MAX,
                    });
                }
            }
        };
        let orientations = rotations();
        for base in bases() {
            let (name, text) = (base.name, base.moves);
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
        let index: HashMap<Vec<usize>, usize> = macros
            .iter()
            .enumerate()
            .map(|(i, m)| (m.step.forward.clone(), i))
            .collect();
        for entry in macros.iter_mut() {
            entry.inverse = *index
                .get(&entry.step.backward)
                .expect("every macro has its inverse");
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
    use crate::puzzle::Facelets;

    #[test]
    fn there_are_24_rotations() {
        assert_eq!(rotations().len(), 24);
    }

    #[test]
    fn every_macro_has_an_inverse() {
        let library = Library::build(PuzzleKind::Cube5);
        assert!(library.macros.len() > 100);
        let identity: Vec<usize> = (0..PuzzleKind::Cube5.facelet_count()).collect();
        for (i, m) in library.macros.iter().enumerate() {
            let there_and_back: Vec<usize> = m
                .step
                .forward
                .iter()
                .map(|&s| library.macros[m.inverse].step.forward[s])
                .collect();
            assert_eq!(there_and_back, identity, "macro {i}");
        }
    }

    #[test]
    fn a_macro_permutes_like_the_facelet_model() {
        let library = Library::build(PuzzleKind::Cube4);
        for m in library.macros.iter().step_by(37) {
            let solved = Facelets::solved(PuzzleKind::Cube4);
            let moved = solved.apply(&MoveSequence::new(m.moves.clone())).unwrap();
            let by_step: Vec<Face> = m
                .step
                .forward
                .iter()
                .map(|&s| solved.stickers()[s])
                .collect();
            assert_eq!(moved.stickers(), &by_step[..], "{}", m.name);
        }
    }
}
