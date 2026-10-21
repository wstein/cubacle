//! A size-independent facade over the cube solvers.
//!
//! Every engine, whatever the cube size, is described by the same vocabulary:
//! a [`PuzzleKind`], a [`Facelets`] state, a [`MoveSequence`] and a [`Solver`].

mod facelets;
mod kind;
mod moves;

pub use facelets::{Face, Facelets, FaceletsError};
pub use kind::PuzzleKind;
pub use moves::{Move, MoveSequence, NotationError, Turn};
