//! Exact fixed-world-frame display distances, separate from the pocket rotation quotient.

mod canonical;
mod coordinates;
mod goal;
mod table;

use std::fmt;
use std::time::Instant;

pub use goal::{Accept, Cell, FrameGoal, Rule};
pub use table::{GoalTable, Transition};

/// Explicit limits for a deterministic table build. A deadline never changes a successful answer.
#[derive(Debug, Clone, Copy)]
pub struct BuildLimits {
    pub max_memory_bytes: usize,
    pub deadline: Option<Instant>,
}

impl Default for BuildLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1_500_000_000,
            deadline: None,
        }
    }
}

/// Validation, resource or invariant failure; an unsatisfiable goal never has a finite distance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixedError {
    InvalidGoal(String),
    InvalidState(String),
    OverLimit,
    Deadline,
    Unsatisfiable,
    Internal(&'static str),
}

impl fmt::Display for FixedError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGoal(message) | Self::InvalidState(message) => out.write_str(message),
            Self::OverLimit => out.write_str("fixed-frame memory limit exceeded"),
            Self::Deadline => out.write_str("fixed-frame deadline exhausted"),
            Self::Unsatisfiable => out.write_str("display has no reachable goal state"),
            Self::Internal(message) => write!(out, "fixed-frame invariant failure: {message}"),
        }
    }
}

impl std::error::Error for FixedError {}

impl BuildLimits {
    fn check_deadline(self) -> Result<(), FixedError> {
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Err(FixedError::Deadline)
        } else {
            Ok(())
        }
    }
}
