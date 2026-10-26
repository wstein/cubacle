//! Synthetic cube face photos: a byte-deterministic renderer that turns a grid
//! of sticker letters into an RGBA image (`docs/specs/synthetic-face.md`).
//!
//! The module is pure and depends on nothing else in the crate.

// Used by the renderer, which lands in the next commit.
#![allow(dead_code)]

mod color;
mod rng;

pub use color::Rgb;
