//! Known routes: a state that is a library pattern, in any of the 48 turned
//! and mirrored positions, is solved by that pattern's generator carried over
//! to the same position.
//!
//! The index holds, for every pattern with a generator on one cube size and for
//! each of the 48 symmetries, the sticker colouring of the image. Looking a
//! state up is one hash lookup. A hit gives the moves that solve the state and
//! the moves that build it from the solved cube; both are replayed before they
//! are returned, so a hit is never wrong.

use std::collections::HashMap;

use super::detector::geometry::{image, symmetries, Sym};
use super::Pattern;
use crate::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// What the index holds for one pattern.
struct Entry {
    name: String,
    catalogue: String,
    /// The shortest plain-move generator on this size.
    generator: MoveSequence,
}

/// The index of one cube size.
pub struct RouteIndex {
    kind: PuzzleKind,
    entries: Vec<Entry>,
    /// Image colouring to (entry, symmetry).
    map: HashMap<Vec<u8>, (u32, u8)>,
}

/// A pattern found in a state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownRoute {
    pub name: String,
    pub catalogue: String,
    /// Moves that take the state to the solved cube.
    pub solve: MoveSequence,
    /// Moves that take the solved cube to the state.
    pub build: MoveSequence,
}

const NORMALS: [[i32; 3]; 6] = [
    [0, 1, 0],
    [1, 0, 0],
    [0, 0, 1],
    [0, -1, 0],
    [-1, 0, 0],
    [0, 0, -1],
];

/// The face a symmetry sends `face` to.
fn face_under(sym: Sym, face: Face) -> Face {
    let normal = sym.apply(NORMALS[face as usize]);
    let at = NORMALS
        .iter()
        .position(|n| *n == normal)
        .expect("a symmetry maps a face normal to a face normal");
    Face::ALL[at]
}

fn flipped(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    }
}

/// The moves of `moves` as seen after the symmetry `sym`.
fn carried(sym: Sym, moves: &MoveSequence) -> MoveSequence {
    moves
        .moves()
        .iter()
        .map(|mv| {
            let turn = if sym.is_mirror() {
                flipped(mv.turn)
            } else {
                mv.turn
            };
            Move::new(face_under(sym, mv.face), mv.depth, mv.wide, turn)
        })
        .collect()
}

fn inverse(moves: &MoveSequence) -> MoveSequence {
    moves
        .moves()
        .iter()
        .rev()
        .map(|mv| Move::new(mv.face, mv.depth, mv.wide, flipped(mv.turn)))
        .collect()
}

impl RouteIndex {
    /// The index of `patterns` on `kind`; patterns without a generator or a
    /// state on that size are left out.
    pub fn build<'a>(kind: PuzzleKind, patterns: impl Iterator<Item = &'a Pattern>) -> RouteIndex {
        let mut index = RouteIndex {
            kind,
            entries: Vec::new(),
            map: HashMap::new(),
        };
        let solved = Facelets::solved(kind);
        for pattern in patterns {
            for generator in pattern
                .generators()
                .iter()
                .filter(|g| g.size == kind.size())
                .filter_map(|g| g.sequence().ok())
                .filter(|s| !s.is_empty() && s.fits(kind))
            {
                index.add(pattern, generator, &solved);
            }
        }
        index
    }

    /// One generator's 48 images. The range and the outer generator of a
    /// pattern may end in different frames of a big cube, so each counts.
    fn add(&mut self, pattern: &Pattern, generator: MoveSequence, solved: &Facelets) {
        let kind = self.kind;
        let Ok(state) = solved.apply(&generator) else {
            return;
        };
        let at = self.entries.len() as u32;
        self.entries.push(Entry {
            name: pattern.name.clone(),
            catalogue: pattern.catalogue.clone(),
            generator,
        });
        let colours: Vec<u8> = state.stickers().iter().map(|f| *f as u8).collect();
        for (g, &sym) in symmetries().iter().enumerate() {
            let key = turned_key(kind, &colours, g, sym);
            let shorter = |old: &(u32, u8)| {
                self.entries[at as usize].generator.len()
                    < self.entries[old.0 as usize].generator.len()
            };
            if self.map.get(&key).is_none_or(shorter) {
                self.map.insert(key, (at, g as u8));
            }
        }
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// The number of patterns in the index.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The pattern `state` is, with the moves that solve it.
    pub fn lookup(&self, state: &Facelets) -> Option<KnownRoute> {
        if state.kind() != self.kind || state.is_solved() {
            return None;
        }
        let key: Vec<u8> = state.stickers().iter().map(|f| *f as u8).collect();
        let &(at, g) = self.map.get(&key)?;
        let entry = &self.entries[at as usize];
        let sym = symmetries()[usize::from(g)];
        let build = carried(sym, &entry.generator);
        let solve = inverse(&build);
        // The replay makes a hit certain, whatever the tables say.
        let solved = Facelets::solved(self.kind);
        if state.apply(&solve).ok()? != solved || solved.apply(&build).ok()? != *state {
            return None;
        }
        Some(KnownRoute {
            name: entry.name.clone(),
            catalogue: entry.catalogue.clone(),
            solve,
            build,
        })
    }
}

/// The colouring of the image under `sym`, the colours renamed to follow
/// the faces they sit on (so the solved cube maps to itself).
fn turned_key(kind: PuzzleKind, colours: &[u8], g: usize, sym: Sym) -> Vec<u8> {
    let mut rename = [0u8; 6];
    for face in Face::ALL {
        rename[face as usize] = face_under(sym, face) as u8;
    }
    image(kind, colours, g)
        .into_iter()
        .map(|c| rename[c as usize])
        .collect()
}

#[cfg(test)]
mod tests;
