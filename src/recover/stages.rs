//! Searching the part of the solution a person points to first.
//!
//! A person who stopped at move `N` ("I did move `N` and the cube no longer
//! matches") made the mistake at move `N` or before it, so only the moves
//! `1..=N` of the solution are candidates and what follows is ignored. The
//! search starts with the narrowest window and widens it only when nothing in
//! it explains the cube; the last stage is the search over the whole solution.

use std::time::{Duration, Instant};

use super::search::Window;
use super::{diagnose_in, Diagnosis, Finding, Limits, RecoverError, Search};
use crate::puzzle::{Facelets, MoveSequence};

/// Printed moves that a mistake itself may add or drop, besides the moves up
/// to the one the person stopped at (a skipped move, an extra move).
const SLACK: usize = 2;

/// What the person knows about where the mistake is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// The move (counting from 1) after which the person noticed the mismatch.
    pub at_move: usize,
    /// The move after which the person is sure the cube was still right. Only
    /// a hint: people check big cubes unreliably, so the search widens past it.
    pub sure_until: Option<usize>,
    /// The first move (0-based) of every phase of the solution, ascending; may
    /// be empty when the phases are not known.
    pub phase_starts: Vec<usize>,
}

impl Scope {
    /// A scope with only the stop known.
    pub fn at(at_move: usize) -> Scope {
        Scope {
            at_move,
            sure_until: None,
            phase_starts: Vec::new(),
        }
    }
}

/// How far the search had to widen before it found the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Widening {
    /// From the move the person is sure of up to the stop.
    Sure,
    /// The phase that contains the stop, up to the stop.
    Phase,
    /// The phase before that one, too.
    PreviousPhase,
    /// All the moves up to the stop.
    Prefix,
    /// The whole solution, as without a scope.
    Full,
    /// One mistake and moves made by hand after it: the last resort, and the
    /// least certain.
    Strays,
}

impl Widening {
    pub fn describe(self) -> &'static str {
        match self {
            Widening::Sure => "the window from the move you are sure of",
            Widening::Phase => "the phase of the stop",
            Widening::PreviousPhase => "the previous phase as well",
            Widening::Prefix => "all moves up to the stop",
            Widening::Full => "the whole solution",
            Widening::Strays => "stray moves after the mistake",
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

/// The index of the phase that contains printed position `pos`.
pub fn phase_index(starts: &[usize], pos: usize) -> usize {
    starts.iter().rposition(|&s| s <= pos).unwrap_or(0)
}

/// The windowed stages for `scope`, narrowest first, each with the first
/// printed position (0-based) a mistake may have. A stage that would not reach
/// back further than the one before it is left out. The whole-solution stage
/// that always follows is not listed.
pub(super) fn plan(scope: &Scope) -> Vec<(Widening, usize)> {
    let stop = scope.at_move - 1;
    let mut out: Vec<(Widening, usize)> = Vec::new();
    let mut add = |stage: Widening, first: usize| {
        if out.last().is_none_or(|&(_, reach)| first < reach) {
            out.push((stage, first));
        }
    };
    if let Some(m) = scope.sure_until.filter(|&m| m < scope.at_move) {
        add(Widening::Sure, m);
    }
    if !scope.phase_starts.is_empty() {
        let at = phase_index(&scope.phase_starts, stop);
        add(Widening::Phase, scope.phase_starts[at]);
        if at > 0 {
            add(Widening::PreviousPhase, scope.phase_starts[at - 1]);
        }
    }
    add(Widening::Prefix, 0);
    out
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
        if let Some(sure) = scope.sure_until.filter(|&m| m > scope.at_move) {
            return Err(RecoverError::SureUntilAfterStop(sure, scope.at_move));
        }
        for (stage, first) in plan(scope) {
            tried.push(stage);
            let found = diagnose_in(
                original,
                current,
                solution,
                &remaining(limits, started),
                Search::In(window(scope, first)),
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
        Search::Whole,
    )?;
    let strays = (!explained(&diagnosis.finding) && limits.strays > 0).then(|| {
        tried.push(Widening::Strays);
        // After the move the person stopped at, the stray moves follow.
        let window = scope.map(|s| window(s, 0));
        diagnose_in(
            original,
            current,
            solution,
            &remaining(limits, started),
            Search::Strays(window),
        )
    });
    if let Some(found) = strays {
        let found = found?;
        if explained(&found.finding) {
            return Ok(Staged {
                diagnosis: found,
                stage: Widening::Strays,
                tried,
            });
        }
    }
    Ok(Staged {
        diagnosis,
        stage: Widening::Full,
        tried,
    })
}

#[cfg(test)]
mod tests;
