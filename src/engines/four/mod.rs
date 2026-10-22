//! The 4x4x4 reduction engine: a port of Tsai's three-phase solver
//! cs0x7f/TPR-4x4x4-Solver.
//!
//! Three table-driven searches bring the centres and wings into place,
//! leaving a 3x3x3 that an injected solver finishes. Files, from the bottom
//! up: `moves` (move numbers and the rules for sequences), `cube` (pieces and
//! their moves), `facelets` (reading and writing stickers), `sym` (the 48
//! symmetries), `center1`, `center2`, `center3`, `edge3` and `edge3_prune`
//! (the coordinates and tables of the phases), `tables` (all of them in one
//! value), `candidates` (partial solutions and the reference's queue),
//! `phase1` to `phase3` (the searches), `reduce` (the phases in a row),
//! `output`, `rotation` and `swap` (the answer in real moves, the final
//! rotation and the wide-turn swaps that shorten it) and
//! `solver` (the [`crate::puzzle::Solver`] adapter, the only public item).

mod candidates;
mod center1;
mod center2;
mod center3;
mod control;
mod cube;
mod edge3;
mod edge3_prune;
mod facelets;
mod finish;
mod moves;
mod output;
mod phase1;
mod phase2;
mod phase3;
mod reduce;
mod rotation;
mod solver;
#[cfg(test)]
mod solver_tests;
mod swap;
mod sym;
mod tables;
#[cfg(test)]
mod testing;

pub use solver::FourSolver;
