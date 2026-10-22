//! The three phases in a row: from a cube to a 3x3x3 with the model's moves.
//!
//! New code; the order of the steps follows `Search.doSearch` of
//! cs0x7f/TPR-4x4x4-Solver.

use super::candidates::Cand;
use super::control::{Control, Stop};
use super::cube::Cube;
use super::tables::Tables;
use super::{phase1, phase2, phase3};

/// The outcome of the three phases.
pub(super) struct Reduction {
    /// The partial solution of phases 1 and 2 (it holds the first-phase moves
    /// and the two extra turns of the model, if any).
    pub cand: Cand,
    /// The moves of phase 3, as move numbers of the model.
    pub move3: Vec<usize>,
}

impl Reduction {
    /// All moves of the model so far.
    pub(super) fn model_moves(&self) -> Vec<u8> {
        let mut all = self.cand.moves().to_vec();
        all.extend(self.move3.iter().map(|&m| m as u8));
        all
    }
}

/// The model cube after `moves`.
pub(super) fn after(t: &Tables, cube: &Cube, moves: &[u8]) -> Cube {
    let mut out = *cube;
    for &m in moves {
        out.apply(&t.corner_moves, usize::from(m));
    }
    out
}

pub(super) fn reduce(
    t: &Tables,
    cube: &Cube,
    threads: usize,
    ctl: &Control,
) -> Result<Reduction, Stop> {
    let cands = phase1::run(t, cube, ctl)?;
    let cubes: Vec<Cube> = cands.iter().map(|c| after(t, cube, c.moves())).collect();
    let mut results = phase2::run(t, &cands, &cubes, threads, ctl)?;
    // The reference sorts the results by value, ties in the order found.
    results.sort_by_key(|c| c.value);
    let (index, move3) = phase3::run(t, &results, threads, ctl)?;
    Ok(Reduction {
        cand: results[index],
        move3,
    })
}
