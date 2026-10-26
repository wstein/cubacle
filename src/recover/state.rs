//! Cube states as plain color arrays, and the move and rotation permutations
//! that act on them.
//!
//! A state is the color of every sticker (0..6, the [`Face`] it shows). Each
//! move is a sticker permutation, so one step is a table walk and works the
//! same for every cube size.

use crate::puzzle::{destinations, Face, Facelets, Move, PuzzleKind, Turn};

pub(super) type State = Vec<u8>;

/// Index of a move in [`Perms`].
pub(super) type MoveId = usize;

pub(super) fn state_of(facelets: &Facelets) -> State {
    facelets.stickers().iter().map(|f| *f as u8).collect()
}

/// One step: `out[dest[i]] = from[i]`.
pub(super) fn permute(out: &mut [u8], from: &[u8], dest: &[u16]) {
    for (i, &d) in dest.iter().enumerate() {
        out[usize::from(d)] = from[i];
    }
}

/// A 64-bit fingerprint of a state at a position of the solution.
pub(super) fn hash(state: &[u8], position: usize) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ (position as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for chunk in state.chunks(8) {
        let mut word = 0u64;
        for &byte in chunk {
            word = (word << 8) | u64::from(byte);
        }
        h = (h ^ word).wrapping_mul(0x0100_0000_01b3);
        h ^= h >> 29;
    }
    h
}

pub(super) fn inverse(mv: &Move) -> Move {
    let turn = match mv.turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    };
    Move::new(mv.face, mv.depth, mv.wide, turn)
}

/// Every move of the universe as a sticker permutation, closed under inverse.
pub(super) struct Perms {
    moves: Vec<Move>,
    perms: Vec<Vec<u16>>,
    inverse: Vec<MoveId>,
}

impl Perms {
    /// The universe: every move the size has when `inner` is set (single inner
    /// layers and wide turns), else the outer face turns only.
    pub(super) fn new(kind: PuzzleKind, inner: bool) -> Perms {
        let mut moves = Vec::new();
        for face in Face::ALL {
            for wide in [false, true] {
                for depth in 1..kind.size() as u8 {
                    if (wide && depth < 2) || (!inner && (wide || depth > 1)) {
                        continue;
                    }
                    for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
                        moves.push(Move::new(face, depth, wide, turn));
                    }
                }
            }
        }
        let perms = moves.iter().map(|m| table(kind, m)).collect();
        let inverse = moves
            .iter()
            .map(|m| {
                let inv = inverse(m);
                moves.iter().position(|x| *x == inv).expect("closed")
            })
            .collect();
        Perms {
            moves,
            perms,
            inverse,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.moves.len()
    }

    pub(super) fn id(&self, mv: &Move) -> Option<MoveId> {
        self.moves.iter().position(|m| m == mv)
    }

    pub(super) fn mv(&self, id: MoveId) -> Move {
        self.moves[id]
    }

    pub(super) fn inv(&self, id: MoveId) -> MoveId {
        self.inverse[id]
    }

    pub(super) fn apply_into(&self, out: &mut [u8], from: &[u8], id: MoveId) {
        permute(out, from, &self.perms[id]);
    }

    pub(super) fn apply(&self, from: &[u8], id: MoveId) -> State {
        let mut out = from.to_vec();
        self.apply_into(&mut out, from, id);
        out
    }

    pub(super) fn apply_all(&self, from: &[u8], ids: &[MoveId]) -> State {
        ids.iter().fold(from.to_vec(), |s, &id| self.apply(&s, id))
    }

    /// The undo of `ids`: the inverse moves in reverse order.
    pub(super) fn undo_of(&self, ids: &[MoveId]) -> Vec<MoveId> {
        ids.iter().rev().map(|&id| self.inv(id)).collect()
    }
}

fn table(kind: PuzzleKind, mv: &Move) -> Vec<u16> {
    destinations(kind, mv)
        .into_iter()
        .map(|d| d as u16)
        .collect()
}

/// A whole-cube rotation: the permutation and the `x y z` moves that make it.
pub(super) struct Rotation {
    pub(super) perm: Vec<u16>,
    pub(super) word: String,
}

/// The 24 rotations of the cube, the identity first, each with a shortest word
/// of `x`, `y` and `z` turns.
pub(super) fn rotations(kind: PuzzleKind) -> Vec<Rotation> {
    let n = kind.size() as u8;
    let mut generators: Vec<(char, Turn, Vec<u16>)> = Vec::new();
    for (letter, face) in [('x', Face::R), ('y', Face::U), ('z', Face::F)] {
        for turn in [Turn::Clockwise, Turn::CounterClockwise] {
            let perm = table(kind, &Move::new(face, n, true, turn));
            generators.push((letter, turn, perm));
        }
    }
    let identity: Vec<u16> = (0..kind.facelet_count() as u16).collect();
    let mut found = vec![(identity, Vec::<(char, Turn)>::new())];
    let mut i = 0;
    while i < found.len() {
        for (letter, turn, g) in &generators {
            let perm: Vec<u16> = found[i].0.iter().map(|&p| g[usize::from(p)]).collect();
            if !found.iter().any(|(p, _)| *p == perm) {
                let mut word = found[i].1.clone();
                word.push((*letter, *turn));
                found.push((perm, word));
            }
        }
        i += 1;
    }
    found
        .into_iter()
        .map(|(perm, word)| Rotation {
            perm,
            word: spell(&word),
        })
        .collect()
}

/// `x x` becomes `x2`, a counter-clockwise turn `x'`.
fn spell(word: &[(char, Turn)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < word.len() {
        let (letter, turn) = word[i];
        if word.get(i + 1) == Some(&(letter, turn)) {
            parts.push(format!("{letter}2"));
            i += 2;
        } else {
            let suffix = if turn == Turn::CounterClockwise {
                "'"
            } else {
                ""
            };
            parts.push(format!("{letter}{suffix}"));
            i += 1;
        }
    }
    parts.join(" ")
}
