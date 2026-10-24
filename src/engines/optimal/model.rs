//! The cube as pieces: where each of the 12 edges and 8 corners is and how it
//! is turned.
//!
//! A piece is stored with its place and its turn in one byte: an edge as
//! `place * 2 + flip`, a corner as `place * 3 + twist`. A move acts on that
//! byte alone, so one table of 24 entries per move and kind describes it, and
//! the pieces of a pruning table can be looked at without searching for them.
//! The tables are not typed in: each is read from the facelets after a single
//! turn of the solved cube.

#![allow(clippy::needless_range_loop)]

use std::sync::OnceLock;

use crate::puzzle::{Face, Facelets, PuzzleKind};

/// The moves: six faces in the order U R F D L B, three turns each (quarter,
/// half, quarter back). Move `m` turns face `m / 3`.
pub(crate) const MOVES: usize = 18;
pub(crate) const FACE_NAMES: [&str; 6] = ["U", "R", "F", "D", "L", "B"];
pub(crate) const TURN_NAMES: [&str; 3] = ["", "2", "'"];

/// The facelets of each corner, the U or D sticker first, then clockwise.
const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];

/// The facelets of each edge: UR UF UL UB DR DF DL DB FR FL BL BR.
const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

/// The pieces of a cube, one byte each; lanes past the last piece stay zero.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Cube {
    pub edges: [u8; 16],
    pub corners: [u8; 16],
    /// The same for the inverse cube: its pieces are the places of this one,
    /// so it is kept by moving places (see `MoveTables::place_edges`).
    pub inv_edges: [u8; 16],
    pub inv_corners: [u8; 16],
}

/// What a move does to a piece byte, for edges and for corners.
pub(crate) struct MoveTables {
    pub edges: [[u8; 32]; MOVES],
    pub corners: [[u8; 32]; MOVES],
    /// For the inverse cube: the place each place takes its piece from (255
    /// past the last place, which a vector lookup turns into zero), and the
    /// turn added there (edges: flip, corners: twist taken away).
    pub from_edges: [[u8; 16]; MOVES],
    pub flip_edges: [[u8; 16]; MOVES],
    pub from_corners: [[u8; 16]; MOVES],
    pub turn_corners: [[u8; 16]; MOVES],
    /// A corner byte with its twist reduced by 0, 1 or 2.
    pub untwist: [[u8; 3]; 24],
}

impl Cube {
    pub(crate) const SOLVED: Cube = {
        let mut cube = Cube {
            edges: [0; 16],
            corners: [0; 16],
            inv_edges: [0; 16],
            inv_corners: [0; 16],
        };
        let mut p = 0;
        while p < 12 {
            cube.edges[p] = (2 * p) as u8;
            cube.inv_edges[p] = (2 * p) as u8;
            p += 1;
        }
        let mut p = 0;
        while p < 8 {
            cube.corners[p] = (3 * p) as u8;
            cube.inv_corners[p] = (3 * p) as u8;
            p += 1;
        }
        cube
    };

    pub(crate) fn is_solved(&self) -> bool {
        self.edges == Cube::SOLVED.edges && self.corners == Cube::SOLVED.corners
    }

    /// Fills `inv_edges` and `inv_corners` from `edges` and `corners`.
    fn invert(&mut self) {
        for p in 0..12 {
            let v = self.edges[p];
            self.inv_edges[usize::from(v >> 1)] = (2 * p) as u8 + (v & 1);
        }
        for p in 0..8 {
            let v = self.corners[p];
            self.inv_corners[usize::from(v / 3)] = (3 * p) as u8 + (3 - v % 3) % 3;
        }
    }

    /// Reads a 3x3x3 given as facelets; `None` when the stickers do not form
    /// the pieces of one cube.
    pub(crate) fn from_facelets(facelets: &Facelets) -> Option<Cube> {
        if facelets.kind() != PuzzleKind::Cube3 {
            return None;
        }
        let colour = |i: usize| face_number(facelets.stickers()[i]);
        let mut cube = Cube::SOLVED;
        let mut seen = [false; 12];
        for (place, slots) in CORNER_FACELETS.iter().enumerate() {
            let colours = slots.map(colour);
            let piece = (0..8).find(|&p| same_set(&colours, &corner_colours(p)))?;
            if std::mem::replace(&mut seen[piece], true) {
                return None;
            }
            let twist = colours.iter().position(|&c| c == 0 || c == 3)?;
            cube.corners[piece] = (3 * place + twist) as u8;
        }
        seen = [false; 12];
        for (place, slots) in EDGE_FACELETS.iter().enumerate() {
            let colours = slots.map(colour);
            let piece = (0..12).find(|&p| same_set(&colours, &edge_colours(p)))?;
            if std::mem::replace(&mut seen[piece], true) {
                return None;
            }
            // A piece with a U or D colour is turned right when that colour
            // is on the first facelet, a middle-layer piece with F or B.
            let own = edge_colours(piece);
            let reference = own
                .into_iter()
                .find(|&c| c == 0 || c == 3)
                .or_else(|| own.into_iter().find(|&c| c == 2 || c == 5))?;
            cube.edges[piece] = (2 * place + usize::from(colours[0] != reference)) as u8;
        }
        cube.invert();
        Some(cube)
    }

    /// The cube after move `m`.
    #[inline]
    pub(crate) fn turned(&self, tables: &MoveTables, m: usize) -> Cube {
        super::vector::turn(self, tables, m)
    }

    /// The cube after move `m`, plain Rust.
    #[inline]
    pub(crate) fn turned_portable(&self, tables: &MoveTables, m: usize) -> Cube {
        let mut out = *self;
        let edge = &tables.edges[m];
        for p in 0..12 {
            out.edges[p] = edge[usize::from(self.edges[p])];
        }
        let corner = &tables.corners[m];
        for p in 0..8 {
            out.corners[p] = corner[usize::from(self.corners[p])];
        }
        for x in 0..12 {
            out.inv_edges[x] =
                self.inv_edges[usize::from(tables.from_edges[m][x])] ^ tables.flip_edges[m][x];
        }
        for x in 0..8 {
            let v = self.inv_corners[usize::from(tables.from_corners[m][x])];
            out.inv_corners[x] =
                tables.untwist[usize::from(v)][usize::from(tables.turn_corners[m][x])];
        }
        out
    }
}

fn face_number(face: Face) -> u8 {
    "URFDLB".find(face.letter()).unwrap_or(0) as u8
}

fn same_set(a: &[u8], b: &[u8]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

fn corner_colours(piece: usize) -> [u8; 3] {
    CORNER_FACELETS[piece].map(|i| (i / 9) as u8)
}

fn edge_colours(piece: usize) -> [u8; 2] {
    EDGE_FACELETS[piece].map(|i| (i / 9) as u8)
}

/// The tables, read once from the facelets.
pub(crate) fn tables() -> &'static MoveTables {
    static TABLES: OnceLock<MoveTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = MoveTables {
            edges: [[0; 32]; MOVES],
            corners: [[0; 32]; MOVES],
            from_edges: [[0xFF; 16]; MOVES],
            flip_edges: [[0; 16]; MOVES],
            from_corners: [[0xFF; 16]; MOVES],
            turn_corners: [[0; 16]; MOVES],
            untwist: [[0; 3]; 24],
        };
        for v in 0..24 {
            for d in 0..3 {
                tables.untwist[v][d] = (v / 3 * 3 + (v % 3 + 3 - d) % 3) as u8;
            }
        }
        for m in 0..MOVES {
            let text = format!("{}{}", FACE_NAMES[m / 3], TURN_NAMES[m % 3]);
            let turn = text.parse().expect("a face turn");
            let after = Facelets::solved(PuzzleKind::Cube3)
                .apply(&turn)
                .expect("a turn of the 3x3x3");
            let cube = Cube::from_facelets(&after).expect("a turned solved cube");
            for p in 0..12 {
                let (to, flip) = (usize::from(cube.edges[p] >> 1), cube.edges[p] & 1);
                for o in 0..2u8 {
                    tables.edges[m][2 * p + usize::from(o)] = (2 * to) as u8 + (o ^ flip);
                }
                tables.from_edges[m][to] = p as u8;
                tables.flip_edges[m][to] = flip;
            }
            for p in 0..8 {
                let (to, twist) = (usize::from(cube.corners[p] / 3), cube.corners[p] % 3);
                for o in 0..3u8 {
                    tables.corners[m][3 * p + usize::from(o)] = (3 * to) as u8 + (o + twist) % 3;
                }
                tables.from_corners[m][to] = p as u8;
                tables.turn_corners[m][to] = twist;
            }
        }
        tables
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn the_solved_cube_reads_as_solved() {
        let cube = Cube::from_facelets(&Facelets::solved(PuzzleKind::Cube3)).unwrap();
        assert!(cube.is_solved());
    }

    #[test]
    fn tables_agree_with_facelets_on_random_scrambles() {
        let tables = tables();
        let mut rng = rand::rng();
        for _ in 0..200 {
            let mut cube = Cube::SOLVED;
            let mut text = String::new();
            for _ in 0..rng.random_range(1..30) {
                let m = rng.random_range(0..MOVES);
                cube = cube.turned_portable(tables, m);
                text += &format!("{}{} ", FACE_NAMES[m / 3], TURN_NAMES[m % 3]);
            }
            let facelets = Facelets::solved(PuzzleKind::Cube3)
                .apply(&text.parse().unwrap())
                .unwrap();
            assert_eq!(Cube::from_facelets(&facelets), Some(cube), "{text}");
            // The inverse part is the cube of the inverse scramble.
            let back: Vec<String> = text
                .split_whitespace()
                .rev()
                .map(|m| match m.strip_suffix('\'') {
                    Some(face) => face.to_owned(),
                    None if m.ends_with('2') => m.to_owned(),
                    None => format!("{m}'"),
                })
                .collect();
            let inverse = Facelets::solved(PuzzleKind::Cube3)
                .apply(&back.join(" ").parse().unwrap())
                .unwrap();
            let inverse = Cube::from_facelets(&inverse).unwrap();
            assert_eq!(cube.inv_edges, inverse.edges, "{text}");
            assert_eq!(cube.inv_corners, inverse.corners, "{text}");
        }
    }
}
