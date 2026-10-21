//! A size-independent facade over the cube solvers.
//!
//! Every engine, whatever the cube size, is described by the same vocabulary:
//! a [`PuzzleKind`], a [`Facelets`] state, a [`MoveSequence`] and a [`Solver`].

mod kind;

pub use kind::PuzzleKind;
