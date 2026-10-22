//! Stopping a search: the time budget, and why a solve ended without a
//! solution.
//!
//! New code.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// Why a solve ended without a solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stop {
    /// The time budget ran out.
    Timeout,
    /// The searches ran through their limits without a solution.
    Exhausted(&'static str),
    /// A state that cannot happen did: a bug, never the caller's fault.
    Internal(&'static str),
}

/// The deadline of one solve, shared by the threads that work on it.
pub(super) struct Control {
    deadline: Option<Instant>,
    expired: AtomicBool,
}

/// Nodes between two looks at the clock.
pub(super) const TICK: u32 = 0xfff;

impl Control {
    pub(super) fn new(deadline: Option<Instant>) -> Control {
        Control {
            deadline,
            expired: AtomicBool::new(false),
        }
    }

    /// Looks at the clock: true once the deadline has passed.
    pub(super) fn check(&self) -> bool {
        if self.expired.load(Ordering::Relaxed) {
            return true;
        }
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.expired.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }

    pub(super) fn expired(&self) -> bool {
        self.expired.load(Ordering::Relaxed)
    }
}
