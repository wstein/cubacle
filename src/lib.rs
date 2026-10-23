//! Rubik's cube solvers and generators, 2x2x2 to 7x7x7.
//!
//! The crate is one crate today and is laid out so that each module can become
//! a workspace crate later with little more than a `Cargo.toml`. See
//! `ARCHITECTURE.md` for the dependency rules, which `tests/architecture_test.rs`
//! enforces.
//!
//! | Module | Role | Depends on |
//! | --- | --- | --- |
//! | [`puzzle`] | cube kinds, facelets, moves, the [`puzzle::Solver`] trait | nothing |
//! | [`notation`] | Orbit64, colour schemes, input detection | `puzzle` |
//! | [`generate`] | random states and scrambles | `puzzle` |
//! | [`engines`] | the 2x2x2 and 3x3x3 solvers | `puzzle` (and `notation` for the 2x2x2) |
//! | [`phases`] | the phases of a solution, from the engine's stages or the cube state | `puzzle` |
//! | [`recover`] | where a cube went wrong in a printed solution, and the cheapest way back | `puzzle` |
//! | [`ranges`] | opt-in compression of a solution into layer ranges | `puzzle`, `notation` |
//! | [`patterns`] | well-known patterns defined by structure, with searched generators | `puzzle` |
//! | [`render`] | move icons as SVG, pixels, Kitty escapes and text | `puzzle` |
//!
//! ```
//! use cubacle::engines::three::Min2PhaseSolver;
//! use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};
//!
//! let scramble = "R U R' U'".parse().unwrap();
//! let cube = Facelets::solved(PuzzleKind::Cube3).apply(&scramble).unwrap();
//!
//! let solver = Min2PhaseSolver::shared();
//! let result = solver.solve(&cube, &SolveOptions::default().with_max_moves(20));
//! let solution = result.solution().expect("solvable");
//! assert!(cube.apply(solution).unwrap().is_solved());
//! ```

pub mod engines;
pub mod generate;
pub mod notation;
pub mod patterns;
pub mod phases;
pub mod puzzle;
pub mod ranges;
pub mod recover;
mod reduction;
pub mod render;
