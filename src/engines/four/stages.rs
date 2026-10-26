//! The stages of a 4x4x4 answer, from the phases of the reduction.
//!
//! New code. The reduction has three phases (see `phase1` to `phase3`) and the
//! 3x3x3 solver follows. Their ends are known on the model cube's moves; here
//! they are moved to the real moves (the two extra turns after phase 1 are
//! not part of the solution) and then carried through the normalizing and the
//! final rotation to the answer.
//!
//! | Stage | What it does |
//! | --- | --- |
//! | `centers staged` | phase 1: every pair of opposite colors has its centers on one axis |
//! | `centers sorted and wings split` | phase 2: the centers sorted far enough and every wing pair split for the last phase |
//! | `centers and wings paired` | phase 3: the centers solved and the wings paired |
//! | `3x3 stage` | the 3x3x3 solver |

use super::reduce::Reduction;
use super::rotation::Rotations;
use crate::puzzle::{stage_ends, Facelets, Move, SolveStage};

/// The names of the stages, in order.
const NAMES: [&str; 4] = [
    "centers staged",
    "centers sorted and wings split",
    "centers and wings paired",
    "3x3 stage",
];

/// The stages of an answer. `raw` is the real moves before any rewriting
/// (the reduction, then the 3x3x3 tail if `three`), `fin` the answer.
pub(super) fn stages(
    rotations: &Rotations,
    start: &Facelets,
    reduction: &Reduction,
    raw: &[Move],
    fin: &[Move],
    three: bool,
) -> Vec<SolveStage> {
    let (length1, add1) = (usize::from(reduction.cand.length1), reduction.cand.add1);
    // model index -> real index: the two extra turns of the model are skipped
    let real = |at: usize| match (add1, at >= length1 + 2) {
        (false, _) => at,
        (true, true) => at - 2,
        (true, false) => at.min(length1),
    };
    let model_end = reduction.cand.moves().len() + reduction.move3.len();
    let mut ends = vec![
        real(length1),
        real(reduction.cand.moves().len()),
        real(model_end),
    ];
    let mut names = NAMES[..3].to_vec();
    if three {
        names.push(NAMES[3]);
        ends.push(raw.len());
    } else if let Some(last) = ends.last_mut() {
        *last = raw.len();
    }
    let mapped = stage_ends(start, raw, &ends, fin, rotations.all());
    names
        .into_iter()
        .zip(mapped)
        .map(|(name, end)| SolveStage { name, end })
        .collect()
}
