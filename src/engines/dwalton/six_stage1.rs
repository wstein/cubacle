//! 6x6x6 stage 1: stage the L/R inner x-centres and pair the L/R oblique
//! centres anywhere.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube666.py (`LookupTable666LRObliquEdgeStageInnerXStage`,
//! `LookupTable666LRInnerXCentersStage`) and ida_search_666.c (the unpaired
//! obliques heuristic), MIT, Copyright (c) 2019 Daniel Walton; this file is
//! AGPL-3.0-or-later, see NOTICE.md. The table is built by search here, not
//! read from the reference's files.

use super::geometry::*;
use super::pattern::{Group, Table};
#[cfg(test)]
use super::stage::PatStage;
use super::stage::{Extra, GroupSpec};
use crate::reduction::cache::Cache;

const LR: u8 = bit(L) | bit(R);

/// Groups of the stage: inner x (0), left obliques (1), right obliques (2).
pub(super) fn groups() -> Vec<GroupSpec> {
    [&INNER_X_666, &LEFT_OBLIQUE_666, &RIGHT_OBLIQUE_666]
        .iter()
        .map(|list| GroupSpec {
            cells: from_ref_all(6, &list[..]),
            marks: LR,
        })
        .collect()
}

/// Eight oblique pairs of L/R colours must be paired; counts unpaired ones.
pub(super) struct Unpaired;

impl Extra for Unpaired {
    fn eval(&self, m: &[u32]) -> i32 {
        if m[1] == m[2] {
            0
        } else {
            (8 - (m[1] & m[2]).count_ones() as i32).max(1)
        }
    }
}

pub(super) fn table(used: &[&MoveDef], cache: &Cache, threads: usize) -> Table {
    let g = groups();
    let goal: u32 = (4..8).chain(12..16).map(|i| 1u32 << i).sum();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    Table::new(
        "dw6-inner-x-lr",
        vec![Group {
            slot: 0,
            cells: g[0].cells.clone(),
            k: 8,
        }],
        &[vec![goal]],
        &dest,
        cache,
        threads,
    )
}

#[cfg(test)]
mod tests {
    use super::super::stage::{select, solve};
    use super::*;
    use crate::generate::scramble;
    use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};
    use crate::reduction::par;

    #[test]
    fn stage_one_solves_scrambles() {
        let defs = catalogue(PuzzleKind::Cube6);
        let used = select(&defs, |_| true);
        let cache = Cache::new(None, 1);
        let threads = par::threads();
        let table = table(&used, &cache, threads);
        let stage = PatStage::new(groups(), &used, vec![&table], Some(&Unpaired));
        for _ in 0..4 {
            let cube = Facelets::solved(PuzzleKind::Cube6)
                .apply(&scramble(PuzzleKind::Cube6, 80))
                .unwrap();
            let node = stage.node_of(|s| cube.stickers()[s] as u8);
            let started = std::time::Instant::now();
            let sol = solve(&stage, node, threads).expect("solution");
            println!("{} moves {:?} in {:?}", sol.len(), sol, started.elapsed());
            let seq: MoveSequence = sol.join(" ").parse().unwrap();
            let after = cube.apply(&seq).unwrap();
            let n2 = stage.node_of(|s| after.stickers()[s] as u8);
            assert_eq!(Unpaired.eval(&n2.0), 0);
            assert_eq!(table.depth(&n2.0), 0);
        }
    }
}
