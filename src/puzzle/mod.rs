//! The vocabulary every engine and every application shares.
//!
//! Every engine, whatever the cube size, is described by the same types: a
//! [`PuzzleKind`], a [`Facelets`] state, a [`MoveSequence`] and a [`Solver`].

mod apply;
mod axis;
mod cubies;
mod facelets;
mod kind;
mod layer_move;
mod moves;
mod orbits;
mod solver;
mod stages;

pub use apply::ApplyError;
pub(crate) use apply::{destinations, locate};
pub(crate) use axis::normalise;
pub(crate) use cubies::five_orbits;
pub use facelets::{Face, Facelets, FaceletsError};
pub use kind::PuzzleKind;
pub use layer_move::LayerMove;
pub use moves::{Move, MoveSequence, NotationError, Turn};
pub(crate) use orbits::{centre_orbits, wing_orbits};
pub use solver::{Failure, SolveMetrics, SolveOptions, SolveResult, Solver};
pub(crate) use stages::stage_ends;
pub use stages::SolveStage;
