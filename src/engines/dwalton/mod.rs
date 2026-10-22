//! 6x6x6 and 7x7x7 reduction engine, after dwalton76/rubiks-cube-NxNxN-solver.
//!
//! The stages follow the reference: centres are staged and solved in steps
//! (each an iterative-deepening search over small pruning tables), the wings
//! are paired by the 4x4x4 and 5x5x5 engines, and a 3x3x3 solver finishes.
//! Files: `geometry` (sticker sets of the reference), `pattern` (pruning
//! tables over colour patterns of sticker sets), `stage` (the search over
//! them).

mod fake;
mod geometry;
mod pattern;
mod seven;
mod seven_final_tables;
mod seven_lr_tables;
mod seven_stage;
mod seven_ud_tables;
mod six;
mod six_stage1;
mod six_stage2;
mod six_stage5;
mod solver;
mod stage;
mod wings;

pub use solver::{Account, DwaltonSolver};
