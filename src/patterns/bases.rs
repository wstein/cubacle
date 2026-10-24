//! The base macros of the library: classic 3x3x3 pattern algorithms, written
//! with outer face turns only. The pattern search lifts each of them to the
//! bigger cubes (see `engines::pattern`); the registry owns them so that the
//! patterns and the algorithms that build them live in one place.

/// A named 3x3x3 algorithm that builds a classic pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base {
    pub name: &'static str,
    /// Outer face turns in WCA notation, separated by spaces.
    pub moves: &'static str,
}

const BASES: [Base; 6] = [
    Base {
        name: "checkerboard",
        moves: "R2 L2 U2 D2 F2 B2",
    },
    Base {
        name: "six spot",
        moves: "U D' R L' F B' U D'",
    },
    Base {
        name: "pons asinorum",
        moves: "F2 B2 U D' R2 L2 U D'",
    },
    Base {
        name: "tetris",
        moves: "L R F B U' D' L' R'",
    },
    Base {
        name: "cube in cube",
        moves: "F L F U' R U F2 L2 U' L' B D' B' L2 U",
    },
    Base {
        name: "cube in cube, long",
        moves: "U' L' U' F' R2 B' R F U B2 U B' L U' F U R F'",
    },
];

/// The base macros, in the order the search enumerates them.
pub fn bases() -> &'static [Base] {
    &BASES
}
