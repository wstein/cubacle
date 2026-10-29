//! A shorter end for the engine's own center sorting.
//!
//! The engine's solution of a target brings its centers to uniform colors
//! with a prefix of `L` moves, but the last moves of that prefix are not the
//! shortest way to finish: the engine works toward the whole solved cube, not
//! toward uniform centers alone. A beam search that starts a few moves before
//! the end of the prefix, on the state the engine reached there, often
//! finishes in about half the moves (measured on the 5x5x5: the last 20 moves
//! in 8). The beam from the very start does not get there, because the sum of
//! its table distances stalls with several colors half sorted; from a state the
//! engine has prepared it does not.
//!
//! So the prefix is cut at the point where cut plus beam is shortest. The
//! work is counted (beam width and depth), not timed, so the result is the
//! same on every machine; only an explicit time budget stops the scan early.

use std::time::{Duration, Instant};

use crate::puzzle::{Facelets, MoveSequence};

use super::search::{Aim, Model};

/// Beam width of the finishing searches. 1000 keeps most of the gain at a
/// third of the time, 16000 gains about one move more at three times it.
const WIDTH: usize = 4000;
/// The largest number of moves cut from the end of the prefix.
const MOST_CUT: usize = 40;
/// Steps between the tried cut points.
const STEP: usize = 2;
/// Cuts this far from the end or nearer cannot be expected to find a shorter
/// finish (the finish has to be shorter than the cut part), so a miss there
/// does not count toward giving up.
const WARM_UP: usize = 12;
/// Cut points in a row without a shorter finish after which the scan stops.
const GIVE_UP: usize = 3;

/// A cut of the engine's prefix: its first `keep` moves, then `finish`.
pub(super) struct Cut {
    pub keep: usize,
    pub finish: MoveSequence,
}

/// The best cut of a prefix that ends in a goal, given the `states` after
/// each of its moves (`states[0]` is where it starts, the last one is a goal);
/// `None` when no finish beats the prefix. The finishes aim the colors at
/// `aim`, and `done` says whether the end of a finish is a goal (for the
/// fix-up: uniform centers in the frame of the start). The scan walks back
/// from the end, stops after a stretch of cuts that find nothing shorter, and
/// stops when `over` says the time is up.
pub(super) fn shorten(
    model: &Model,
    states: &[Facelets],
    aim: Aim,
    done: &dyn Fn(&Facelets) -> bool,
    over: &dyn Fn() -> bool,
) -> Option<Cut> {
    let len = states.len() - 1;
    let mut best: Option<Cut> = None;
    let mut best_total = len;
    let mut misses = 0;
    for back in (STEP..=len.min(MOST_CUT)).step_by(STEP) {
        if over() {
            break;
        }
        let keep = len - back;
        // strictly shorter than the part it replaces
        let finish = model
            .beam(&states[keep], aim, WIDTH, back - 1)
            .filter(|finish| states[keep].apply(finish).is_ok_and(|end| done(&end)));
        match finish {
            Some(finish) if keep + finish.len() < best_total => {
                best_total = keep + finish.len();
                best = Some(Cut { keep, finish });
                misses = 0;
            }
            _ => {
                if back > WARM_UP {
                    misses += 1;
                }
                if misses >= GIVE_UP {
                    break;
                }
            }
        }
    }
    best
}

/// Whether the explicit time `limit`, if any, has passed since `started`.
pub(super) fn out_of_time(started: Instant, limit: Option<Duration>) -> bool {
    limit.is_some_and(|limit| started.elapsed() >= limit)
}
