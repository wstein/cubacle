//! Searching the part of the solution a person points to first.
//!
//! A person who stopped at move `N` ("I did move `N` and the cube no longer
//! matches") made the mistake at move `N` or before it, so only the moves
//! `1..=N` of the solution are candidates and what follows is ignored. The
//! search starts with the narrowest window and widens it only when nothing in
//! it explains the cube; the last stage is the search over the whole solution.

use std::time::{Duration, Instant};

use super::search::Window;
use super::{diagnose_in, Diagnosis, Finding, Limits, RecoverError};
use crate::puzzle::{Facelets, MoveSequence};

/// Printed moves that a mistake itself may add or drop, besides the moves up
/// to the one the person stopped at (a skipped move, an extra move).
const SLACK: usize = 2;

/// What the person knows about where the mistake is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// The move (counting from 1) after which the person noticed the mismatch.
    pub at_move: usize,
}

/// How far the search had to widen before it found the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Widening {
    /// The moves up to the one the person stopped at.
    Prefix,
    /// The whole solution, as without a scope.
    Full,
}

impl Widening {
    pub fn describe(self) -> &'static str {
        match self {
            Widening::Prefix => "the moves up to the stop",
            Widening::Full => "the whole solution",
        }
    }
}

/// A diagnosis and the stage that found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    pub diagnosis: Diagnosis,
    pub stage: Widening,
    /// The stages tried, the last being the one that answered.
    pub tried: Vec<Widening>,
}

/// The windowed stages for `scope`, narrowest first, each with the first
/// printed position (0-based) a mistake may have. The whole-solution stage
/// that always follows is not listed.
pub(super) fn plan(_scope: &Scope) -> Vec<(Widening, usize)> {
    vec![(Widening::Prefix, 0)]
}

pub(super) fn window(scope: &Scope, first: usize) -> Window {
    Window {
        first,
        last: scope.at_move - 1,
        done_lo: scope.at_move.saturating_sub(SLACK),
        done_hi: scope.at_move + SLACK,
    }
}

fn explained(finding: &Finding) -> bool {
    matches!(finding, Finding::Mistakes { .. } | Finding::OnPath { .. })
}

/// `limits` with what is left of the time budget since `started`.
fn remaining(limits: &Limits, started: Instant) -> Limits {
    Limits {
        max_time: limits.max_time.map(|t| {
            t.saturating_sub(started.elapsed())
                .max(Duration::from_millis(1))
        }),
        ..*limits
    }
}

/// [`super::diagnose`], held to the windows of `scope` first. Without a scope
/// it is the plain search.
pub fn diagnose_staged(
    original: &Facelets,
    current: &Facelets,
    solution: &MoveSequence,
    limits: &Limits,
    scope: Option<&Scope>,
) -> Result<Staged, RecoverError> {
    let started = Instant::now();
    let mut tried = Vec::new();
    if let Some(scope) = scope {
        if scope.at_move == 0 || scope.at_move > solution.len() {
            return Err(RecoverError::AtMoveOutOfRange(
                scope.at_move,
                solution.len(),
            ));
        }
        for (stage, first) in plan(scope) {
            tried.push(stage);
            let found = diagnose_in(
                original,
                current,
                solution,
                &remaining(limits, started),
                Some(window(scope, first)),
            )?;
            if explained(&found.finding) {
                return Ok(Staged {
                    diagnosis: found,
                    stage,
                    tried,
                });
            }
        }
    }
    tried.push(Widening::Full);
    let diagnosis = diagnose_in(
        original,
        current,
        solution,
        &remaining(limits, started),
        None,
    )?;
    Ok(Staged {
        diagnosis,
        stage: Widening::Full,
        tried,
    })
}

#[cfg(test)]
mod tests;
