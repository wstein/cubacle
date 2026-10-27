//! The direct route: start and target are close.
//!
//! A target with mixed centers is normally reached through a uniform nearby
//! goal and a fix-up (see the parent module). When the target is only a few
//! moves away from the start, that detour is far longer than the few moves,
//! so a meet-in-the-middle search over the layer moves looks for the route
//! between the two states first. It is the search of `solve --within`, with
//! the single inner layers in its alphabet; a route it finds is the shortest
//! one in that alphabet, replayed and checked.

use std::time::Duration;

use crate::puzzle::{Facelets, MoveSequence};
use crate::within::{self, Kit, Limits, Outcome};

/// The longest direct route looked for, in moves. Measured on the 7x7x7: six
/// moves take 0.2 s and 9 MB when there is no route, seven take 6 s and
/// 700 MB.
const DIRECT_MOVES: usize = 6;
/// Memory for the stored side, in megabytes. The stored depth follows from it
/// and from the bound, so the search is the same on every machine.
const DIRECT_MEMORY_MB: usize = 100;
/// A safety stop, not a budget: the search is bounded by moves and memory,
/// and a stop by this limit only means that the normal route is used.
const DIRECT_TIME: Duration = Duration::from_secs(20);

/// The shortest route of at most [`DIRECT_MOVES`] moves from `start` to
/// `target`, if there is one. States that are further apart are excluded by
/// the lower bound of the search without any work.
pub(super) fn route(start: &Facelets, target: &Facelets) -> Option<MoveSequence> {
    let limits = Limits {
        max_moves: DIRECT_MOVES,
        max_memory_mb: DIRECT_MEMORY_MB,
        max_time: DIRECT_TIME,
    };
    match within::search(start, target, Kit::Slices, &limits).outcome {
        Outcome::Found(route) if !route.is_empty() => Some(route),
        _ => None,
    }
}
