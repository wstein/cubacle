//! The 5x5x5 reduction engine: a port of the five-phase solver cs0x7f/cube555.
//!
//! Five table-driven searches bring the centers and wings into place, leaving
//! a 3x3x3 that an injected solver finishes. Files, from the bottom up:
//! `mv` (move numbers), `cubie` and `cubie_sym` (the pieces and the 48
//! symmetries), `p1` to `p5` (one phase each: coordinates, tables, search
//! node; `p4_model` and `p5_model` hold their piece models), `spaces` (search
//! spaces shared by tables), `build` and `tables` (making the pruning tables,
//! or reading them from the cache), `solving` and `reduction` (the phases in a
//! row) and `solver` (the [`crate::puzzle::Solver`] adapter, the only public
//! item). The searches themselves are the generic ones of `crate::reduction`.

mod build;
mod cubie;
mod cubie_sym;
mod finish;
mod mv;
mod p1;
mod p2;
mod p3;
mod p4;
mod p4_model;
mod p5;
mod p5_model;
mod reduction;
mod solver;
mod solving;
mod spaces;
mod tables;
#[cfg(test)]
mod testing;

pub use solver::FiveSolver;
