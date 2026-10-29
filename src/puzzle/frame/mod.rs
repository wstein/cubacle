//! Fixed-world-frame, identity-preserving 2x2x2 corner states.
//!
//! This is the typed geometry contract for fixed-frame proofs, not a replacement
//! for color targets or the existing rotation-equivalent pocket solver.

mod geometry;
mod keys;
mod printing;
mod state;

use std::fmt;

use super::{Face, Move, Turn};
use geometry::Rotation;

pub use keys::Projection;

/// An integer position, normal or artwork-up vector in the world frame.
pub type Vector = [i8; 3];

/// One immutable physical sticker, or its current rigidly transformed frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sticker {
    pub id: u8,
    pub cubie: u8,
    pub p: Vector,
    pub n: Vector,
    pub up: Vector,
    pub image: Option<String>,
}

/// A complete reachable pose, indexed by physical body identity.
///
/// Pose codes are relative to their reference [`Print`]. Exchange or persist them
/// with that print (or its hash); equality has meaning within the same print context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    poses: [u8; 8],
}

impl State {
    pub fn poses(&self) -> &[u8; 8] {
        &self.poses
    }
}

/// A validated reference print with its 24 proper placements per body.
#[derive(Debug, Clone)]
pub struct Print {
    stickers: [Sticker; 24],
    positions: [Vector; 8],
    rotations: [[Rotation; 24]; 8],
    move_table: [[[u8; 18]; 24]; 8],
}

/// Invalid print, rigid state, projection or unsupported layer move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameError(String);

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FrameError {}

fn invalid(message: impl Into<String>) -> FrameError {
    FrameError(message.into())
}

/// All 18 HTM moves in the canonical certificate tie-breaking order.
pub fn moves() -> [Move; 18] {
    std::array::from_fn(|i| {
        Move::new(
            Face::ALL[i / 3],
            1,
            false,
            [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][i % 3],
        )
    })
}

fn move_index(mv: Move) -> Result<usize, FrameError> {
    if mv.depth != 1 || mv.wide {
        return Err(invalid("fixed-frame moves must be outer-face turns"));
    }
    Ok(mv.face as usize * 3
        + match mv.turn {
            Turn::Clockwise => 0,
            Turn::Half => 1,
            Turn::CounterClockwise => 2,
        })
}

fn face_axis(face: Face) -> (usize, i8) {
    match face {
        Face::U => (1, 1),
        Face::R => (0, 1),
        Face::F => (2, 1),
        Face::D => (1, -1),
        Face::L => (0, -1),
        Face::B => (2, -1),
    }
}

#[cfg(test)]
mod tests;
