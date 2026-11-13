//! The cube as pieces for the target-set search: the 18 turns as permutation
//! tables and a compact position that applies them.
//!
//! The tables are read off the engine's own cubie model (a turn of the solved
//! cube), so the two agree by construction; a test replays answers through
//! the facelet model as well.

use super::super::cubie::Cubie;
use crate::puzzle::{CenterTwists, Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// Face turns: clockwise, half and counterclockwise of U R F D L B.
pub(super) const TURNS: usize = 18;

/// How each turn moves the pieces: after turn `m`, slot `t` holds what slot
/// `from[m][t]` held, and its orientation grew by `add[m][t]`.
pub(super) struct Turns {
    pub(super) cfrom: [[u8; 8]; TURNS],
    pub(super) cadd: [[u8; 8]; TURNS],
    pub(super) efrom: [[u8; 12]; TURNS],
    pub(super) eadd: [[u8; 12]; TURNS],
    pub(super) moves: [Move; TURNS],
}

impl Turns {
    pub(super) fn new() -> Turns {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        let mut turns = Turns {
            cfrom: [[0; 8]; TURNS],
            cadd: [[0; 8]; TURNS],
            efrom: [[0; 12]; TURNS],
            eadd: [[0; 12]; TURNS],
            moves: [Move::new(Face::U, 1, false, Turn::Clockwise); TURNS],
        };
        for (f, face) in Face::ALL.into_iter().enumerate() {
            for (q, turn) in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise]
                .into_iter()
                .enumerate()
            {
                let m = f * 3 + q;
                let mv = Move::new(face, 1, false, turn);
                let turned = solved
                    .apply(&MoveSequence::new(vec![mv]))
                    .expect("a face turn fits");
                let mut cube = Cubie::new();
                assert_eq!(cube.set_from_facelet(&turned.to_string()), 0);
                for t in 0..8 {
                    turns.cfrom[m][t] = cube.ca[t] & 7;
                    turns.cadd[m][t] = cube.ca[t] >> 3;
                }
                for t in 0..12 {
                    turns.efrom[m][t] = cube.ea[t] >> 1;
                    turns.eadd[m][t] = cube.ea[t] & 1;
                }
                turns.moves[m] = mv;
            }
        }
        turns
    }
}

/// The pieces: for every slot, the piece in it and its orientation (corner
/// twist 0 to 2, edge flip 0 or 1), in the numbering of the cubie model.
#[derive(Clone, Copy)]
pub(super) struct Pos {
    pub(super) cp: [u8; 8],
    pub(super) co: [u8; 8],
    pub(super) ep: [u8; 12],
    pub(super) eo: [u8; 12],
    /// The center twists packed two bits per face, in the order `URFDLB`.
    pub(super) tw: u16,
}

impl Pos {
    pub(super) fn from_cubie(cube: &Cubie) -> Pos {
        let mut pos = Pos {
            cp: [0; 8],
            co: [0; 8],
            ep: [0; 12],
            eo: [0; 12],
            tw: 0,
        };
        for s in 0..8 {
            pos.cp[s] = cube.ca[s] & 7;
            pos.co[s] = cube.ca[s] >> 3;
        }
        for s in 0..12 {
            pos.ep[s] = cube.ea[s] >> 1;
            pos.eo[s] = cube.ea[s] & 1;
        }
        pos
    }

    #[inline]
    pub(super) fn step(&self, turns: &Turns, m: usize) -> Pos {
        let mut next = *self;
        for t in 0..8 {
            let from = usize::from(turns.cfrom[m][t]);
            next.cp[t] = self.cp[from];
            next.co[t] = (self.co[from] + turns.cadd[m][t]) % 3;
        }
        for t in 0..12 {
            let from = usize::from(turns.efrom[m][t]);
            next.ep[t] = self.ep[from];
            next.eo[t] = self.eo[from] ^ turns.eadd[m][t];
        }
        next.tw = twist_step(self.tw, m);
        next
    }

    /// The same pieces with the center twists `twist`.
    pub(super) fn with_twist(mut self, twist: CenterTwists) -> Pos {
        self.tw = pack(Face::ALL.map(|f| twist.twist(f)));
        self
    }
}

/// The twists packed two bits per face.
pub(super) fn pack(t: [u8; 6]) -> u16 {
    t.iter()
        .enumerate()
        .fold(0, |acc, (g, &k)| acc | u16::from(k & 3) << (2 * g))
}

/// The twist of face `g` in a packed vector.
#[inline]
pub(super) fn twist_of(tw: u16, g: usize) -> u8 {
    (tw >> (2 * g) & 3) as u8
}

/// The packed twists after turn `m`: face `m / 3` gains `m % 3 + 1` quarters.
#[inline]
pub(super) fn twist_step(tw: u16, m: usize) -> u16 {
    let shift = 2 * (m / 3);
    let turned = ((tw >> shift & 3) + (m % 3) as u16 + 1) & 3;
    tw & !(3 << shift) | turned << shift
}
