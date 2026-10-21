//! A size-independent facade over the cube solvers.
//!
//! Every engine, whatever the cube size, is described by the same vocabulary:
//! a [`PuzzleKind`], a [`Facelets`] state, a [`MoveSequence`] and a [`Solver`].

mod facelets;
mod input;
mod kind;
mod min2phase;
mod moves;
pub mod orbit64;
mod pocket;
mod scheme;
mod solver;

pub use facelets::{Face, Facelets, FaceletsError};
pub use input::{parse_cube, InputError};
pub use kind::PuzzleKind;
pub use min2phase::Min2PhaseSolver;
pub use moves::{Move, MoveSequence, NotationError, Turn};
pub use pocket::PocketSolver;
pub use scheme::{ColourScheme, SchemeError};
pub use solver::{Failure, SolveMetrics, SolveOptions, SolveResult, Solver};
