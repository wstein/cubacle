//! A cube on its way through the phases: the pieces, the symmetry the search
//! currently looks at it through, and the moves made so far.
//!
//! Ported from cs0x7f/cube555 src/SolvingCube.java, MIT, Copyright (c) 2023
//! Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.

use super::cubie::Cubie;
use super::cubie_sym::Syms;

#[derive(Clone)]
pub(super) struct Solving {
    pub cube: Cubie,
    conj: usize,
    /// The moves made, in the orientation of the original cube.
    pub moves: Vec<u8>,
    /// The number of moves made when each phase ended, in order.
    pub ends: Vec<usize>,
}

impl Solving {
    pub(super) fn new(cube: Cubie) -> Solving {
        Solving {
            cube,
            conj: 0,
            moves: Vec::new(),
            ends: Vec::new(),
        }
    }

    /// Moves made so far.
    pub(super) fn cost(&self) -> i32 {
        self.moves.len() as i32
    }

    pub(super) fn do_moves(&mut self, syms: &Syms, moves: &[u8]) {
        for &m in moves {
            self.cube.do_move(m);
            self.moves.push(syms.mv[self.conj][usize::from(m)]);
        }
    }

    pub(super) fn do_conj(&mut self, syms: &Syms, idx: usize) {
        syms.conj(&mut self.cube, idx);
        self.conj = usize::from(syms.mult[self.conj][idx]);
    }
}
