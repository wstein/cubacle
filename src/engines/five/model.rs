//! The 5x5x5 as sticker permutations.
//!
//! Only layer turns of depth 1 and 2 are used. They never move the six fixed
//! centre stickers, so the solved cube is the solved facelet string and no
//! whole-cube rotation is ever needed.

use crate::puzzle::{
    destinations, five_orbits, Face, Facelets, FiveOrbits, Move, MoveSequence, PuzzleKind, Turn,
};

pub(super) const KIND: PuzzleKind = PuzzleKind::Cube5;
/// Stickers on a 5x5x5.
pub(super) const STICKERS: usize = 150;
/// Six faces, two layer depths, three turn amounts.
pub(super) const MOVES: usize = 36;

/// Sticker colours, as the index of the face that shows them.
pub(super) type Colours = [u8; STICKERS];

/// A move by number: `(face * 2 + depth - 1) * 3 + turn`, turn 0, 1, 2 being
/// clockwise, half and counter-clockwise.
pub(super) type MoveId = u8;

pub(super) struct Model {
    /// `dest[m][i]` is where move `m` takes the sticker at `i`.
    dest: Vec<[u8; STICKERS]>,
    /// Sticker index of every x-centre slot.
    pub x: Vec<usize>,
    /// Sticker index of every t-centre slot.
    pub t: Vec<usize>,
    /// The two stickers of every wing slot, in a handedness that turns keep.
    pub wings: Vec<[usize; 2]>,
    /// For every wing slot, the middle-edge stickers on the same two faces,
    /// in the order of the wing's own stickers.
    pub beside: Vec<[usize; 2]>,
    /// The stickers that make up the 3x3x3 hidden inside (rows and columns 0, 2, 4).
    pub three: Vec<usize>,
}

impl Model {
    pub(super) fn new() -> Model {
        let mut dest = Vec::with_capacity(MOVES);
        for id in 0..MOVES as MoveId {
            let mut table = [0u8; STICKERS];
            for (slot, &to) in destinations(KIND, &move_of(id)).iter().enumerate() {
                table[slot] = to as u8;
            }
            dest.push(table);
        }
        let FiveOrbits { x, t, wings, mids } = five_orbits();
        let beside = wings
            .iter()
            .map(|wing: &[usize; 2]| {
                let on = |face: usize, other: usize| {
                    mids.iter()
                        .find(|mid: &&[usize; 2]| {
                            mid.iter().any(|m| m / 25 == face)
                                && mid.iter().any(|m| m / 25 == other)
                        })
                        .and_then(|mid| mid.iter().copied().find(|m| m / 25 == face))
                        .expect("a middle edge on the same two faces")
                };
                [
                    on(wing[0] / 25, wing[1] / 25),
                    on(wing[1] / 25, wing[0] / 25),
                ]
            })
            .collect();
        let three = (0..STICKERS)
            .filter(|i| {
                let local = i % 25;
                (local / 5) % 2 == 0 && (local % 5) % 2 == 0
            })
            .collect();
        Model {
            dest,
            x,
            t,
            wings,
            beside,
            three,
        }
    }

    pub(super) fn apply(&self, colours: &mut Colours, id: MoveId) {
        let table = &self.dest[usize::from(id)];
        let old = *colours;
        for (slot, &to) in table.iter().enumerate() {
            colours[usize::from(to)] = old[slot];
        }
    }

    pub(super) fn apply_all(&self, colours: &mut Colours, ids: &[MoveId]) {
        for &id in ids {
            self.apply(colours, id);
        }
    }

    pub(super) fn table(&self, id: MoveId) -> &[u8; STICKERS] {
        &self.dest[usize::from(id)]
    }

    /// The 3x3x3 facelets seen through rows and columns 0, 2 and 4.
    pub(super) fn three_by_three(&self, colours: &Colours) -> Facelets {
        let stickers = self
            .three
            .iter()
            .map(|&i| Face::ALL[usize::from(colours[i])])
            .collect();
        Facelets::from_stickers(stickers).expect("a 3x3x3 sample keeps nine stickers per colour")
    }
}

pub(super) fn colours_of(facelets: &Facelets) -> Colours {
    let mut colours = [0u8; STICKERS];
    for (slot, face) in colours.iter_mut().zip(facelets.stickers()) {
        *slot = face.index() as u8;
    }
    colours
}

pub(super) fn move_of(id: MoveId) -> Move {
    let group = usize::from(id) / 3;
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][usize::from(id) % 3];
    Move::new(Face::ALL[group / 2], (group % 2) as u8 + 1, false, turn)
}

pub(super) fn id_of(mv: &Move) -> Option<MoveId> {
    if mv.wide || !(1..=2).contains(&mv.depth) {
        return None;
    }
    let turn = match mv.turn {
        Turn::Clockwise => 0,
        Turn::Half => 1,
        Turn::CounterClockwise => 2,
    };
    Some(((mv.face as u8 * 2 + mv.depth - 1) * 3) + turn)
}

/// The same turn undone: quarter turns swap direction, half turns stay.
pub(super) fn inverse(id: MoveId) -> MoveId {
    id - id % 3 + (2 - id % 3)
}

pub(super) fn invert(ids: &[MoveId]) -> Vec<MoveId> {
    ids.iter().rev().map(|&id| inverse(id)).collect()
}

/// The turn group `(face, depth)` a move belongs to.
pub(super) fn group(id: MoveId) -> u8 {
    id / 3
}

pub(super) fn parse(text: &str) -> Vec<MoveId> {
    let moves: MoveSequence = text.parse().expect("a valid move sequence");
    moves
        .moves()
        .iter()
        .map(|mv| id_of(mv).expect("a depth 1 or 2 turn"))
        .collect()
}

pub(super) fn sequence(ids: &[MoveId]) -> MoveSequence {
    MoveSequence::new(ids.iter().map(|&id| move_of(id)).collect())
}

/// Deterministic scrambles for the tests of every stage.
#[cfg(test)]
pub(super) mod support {
    use super::*;

    /// A deterministic stream of numbers below a bound.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self, bound: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) % bound
        }
    }

    /// `length` random turns of depth 1 or 2, which keep the centres fixed.
    pub(in crate::engines::five) fn scramble(seed: u64, length: usize) -> Vec<MoveId> {
        let mut rng = Lcg(seed);
        (0..length)
            .map(|_| rng.next(MOVES as u64) as MoveId)
            .collect()
    }

    pub(in crate::engines::five) fn scrambled(seed: u64, length: usize) -> Facelets {
        Facelets::solved(PuzzleKind::Cube5)
            .apply(&sequence(&scramble(seed, length)))
            .unwrap()
    }
}
