//! The 5x5x5 reduction engine.
//!
//! Files, from the bottom up: `model` (the cube as sticker permutations of
//! the layer turns that keep the centres fixed), `library` (pure 3-cycles of
//! centre and wing pieces), `orbit` (ordering one orbit with them), `reduce`
//! (centres, then wings, and the parity of the wings) and `solver` (the
//! [`crate::puzzle::Solver`] adapter, the only public item, which finishes
//! with an injected 3x3x3 solver).

mod library;
mod model;
mod orbit;
mod reduce;
mod solver;

pub use solver::FiveSolver;
#[allow(dead_code)]
mod cubie;
#[allow(dead_code)]
mod cubie_sym;
mod mv;
