//! All tables of the 4x4x4 solver, owned by one value.
//!
//! New code; the contents follow cs0x7f/TPR-4x4x4-Solver `Search.init`.

use super::center1::Center1;
use super::center2::Center2;
use super::center3::Center3;
use super::cube::CornerMoves;
use super::edge3::Edge3;
use super::edge3_prune::WingPrun;
use super::sym::Syms;

pub(super) struct Tables {
    pub syms: Syms,
    pub corner_moves: CornerMoves,
    pub c1: Center1,
    pub c2: Center2,
    pub c3: Center3,
    pub e3: Edge3,
    pub wings: WingPrun,
}

impl Tables {
    /// Builds the tables on `threads` threads. `layers` is the depth of the
    /// wing table (9 for the real one; tests use less).
    pub(super) fn build(threads: usize, layers: u32) -> Tables {
        let syms = Syms::new();
        let c1 = Center1::build(&syms);
        let e3 = Edge3::new();
        let wings = WingPrun::build(&e3, threads.max(1), layers);
        Tables {
            syms,
            corner_moves: CornerMoves::new(),
            c1,
            c2: Center2::build(),
            c3: Center3::build(),
            e3,
            wings,
        }
    }

    /// Bytes held by the big tables.
    pub(super) fn bytes(&self) -> usize {
        self.c1.ctsmv.len() * 4
            + self.c1.csprun.len()
            + self.c1.sym2raw.len() * 4
            + self.c2.ctmv.len() * 2
            + self.c2.rlmv.len()
            + self.c2.ctprun.len()
            + self.c3.ctmove.len() * 2
            + self.c3.prun.len()
            + self.wings.bytes()
    }
}
