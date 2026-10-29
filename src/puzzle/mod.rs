//! The vocabulary every engine and every application shares.
//!
//! Every engine, whatever the cube size, is described by the same types: a
//! [`PuzzleKind`], a [`Facelets`] state, a [`MoveSequence`] and a [`Solver`].

mod alphabet;
mod apply;
mod axis;
mod centers;
mod cubies;
mod facelets;
mod kind;
mod layer_move;
mod mask;
mod moves;
mod orbits;
mod order;
mod relative;
mod rotation;
mod solver;
mod stages;
mod target;
mod threads;

pub use alphabet::Alphabet;
pub(crate) use alphabet::{inner_layer_set, layer_set};
pub use apply::ApplyError;
pub(crate) use apply::{destinations, locate};
pub(crate) use axis::normalize;
pub(crate) use centers::centers_off_face;
pub use centers::{oriented_by_centers, solves_by_color};
pub(crate) use cubies::five_orbits;
pub use facelets::{Face, Facelets, FaceletsError};
pub use kind::PuzzleKind;
pub use layer_move::LayerMove;
pub use mask::Mask;
pub use moves::{Move, MoveSequence, NotationError, Turn};
pub(crate) use orbits::{center_orbits, wing_orbits};
pub use order::{sticker_cycles, OrbitCycles, StickerCycles, StickerOrbit};
#[cfg(test)]
pub(crate) use relative::relative_by;
pub use relative::{reaches, relative_state, RelativeError};
pub(crate) use rotation::rotation_maps;
pub use solver::{Failure, SolveMetrics, SolveOptions, SolveResult, Solver};
pub use stages::SolveStage;
pub(crate) use stages::{interned, stage_ends, stage_parts, PHASE_MOVES};
pub use target::{Target, TargetError};
pub(crate) use threads::threads;
