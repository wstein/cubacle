//! 6x6x6 stages 2 to 4: stage the L/R centers (a 5x5x5 stage on the paired
//! obliques and the outer x-centers), stage and pair the U/D inner x-centers
//! and obliques, stage the U/D centers.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube666.py (`LookupTable666UDInnerXCentersStage`,
//! `LookupTable666UDObliquEdgeStage`) and RubiksCube555.py
//! (`LookupTableIDA555LRCenterStage`, `LookupTableIDA555FBCentersStage`),
//! MIT, Copyright (c) 2019 Daniel Walton; this file is AGPL-3.0-or-later, see
//! NOTICE.md. The tables are built by search here.

use super::geometry::*;
use super::pattern::{Group, Table};
use super::stage::{Extra, GroupSpec};
use crate::reduction::cache::Cache;

const LR: u8 = bit(L) | bit(R);
const UD: u8 = bit(U) | bit(D);

/// The 24 stickers of a set that lie on the faces `faces` (cubacle indices),
/// in the order of the set.
fn on(list: &[usize], faces: &[usize]) -> Vec<usize> {
    from_ref_all(6, list)
        .into_iter()
        .filter(|s| faces.contains(&(s / 36)))
        .collect()
}

/// Stage 2: t-centers (left obliques as representatives) and x-centers, both
/// by color L/R.
pub(super) fn lr_groups() -> Vec<GroupSpec> {
    vec![
        GroupSpec {
            cells: from_ref_all(6, &LEFT_OBLIQUE_666),
            marks: LR,
        },
        GroupSpec {
            cells: from_ref_all(6, &OUTER_X_666),
            marks: LR,
        },
    ]
}

/// Mask of the goal `L` and `R` faces in a 24-sticker set of faces
/// U L F R B D.
fn lr_goal() -> u32 {
    (4..8).chain(12..16).map(|i| 1u32 << i).sum()
}

pub(super) fn lr_tables(used: &[&MoveDef], cache: &Cache, threads: usize) -> Vec<Table> {
    let g = lr_groups();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    ["dw6-t-lr", "dw6-x-lr"]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            Table::new(
                name,
                vec![Group {
                    slot: i,
                    cells: g[i].cells.clone(),
                    k: 8,
                }],
                &[vec![lr_goal()]],
                &dest,
                cache,
                threads,
            )
        })
        .collect()
}

/// Faces U F B D of the cubacle order, for stages 3 and 4.
const UFBD: [usize; 4] = [U, F, D, B];

/// Stage 3: UFBD inner x-centers (0), left obliques (1), right obliques (2),
/// by color U/D.
pub(super) fn ud_groups() -> Vec<GroupSpec> {
    [&INNER_X_666, &LEFT_OBLIQUE_666, &RIGHT_OBLIQUE_666]
        .iter()
        .map(|list| GroupSpec {
            cells: on(&list[..], &UFBD),
            marks: UD,
        })
        .collect()
}

/// In the 16-sticker sets of faces U F B D (the order of the reference) the
/// U/D colors belong on the first and last four.
fn ud_goal() -> u32 {
    (0..4).chain(12..16).map(|i| 1u32 << i).sum()
}

/// The U/D obliques are paired when both sets show the same stickers.
pub(super) struct UdPaired;

impl Extra for UdPaired {
    fn eval(&self, m: &[u32]) -> i32 {
        if m[1] == m[2] {
            0
        } else {
            (8 - (m[1] & m[2]).count_ones() as i32).max(1)
        }
    }
}

pub(super) fn ud_inner_table(used: &[&MoveDef], cache: &Cache, threads: usize) -> Table {
    let g = ud_groups();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    Table::new(
        "dw6-inner-x-ud",
        vec![Group {
            slot: 0,
            cells: g[0].cells.clone(),
            k: 8,
        }],
        &[vec![ud_goal()]],
        &dest,
        cache,
        threads,
    )
}

/// Stage 4: UFBD left obliques (0) and outer x-centers (1) by color U/D.
pub(super) fn ud_stage_groups() -> Vec<GroupSpec> {
    [&LEFT_OBLIQUE_666, &OUTER_X_666]
        .iter()
        .map(|list| GroupSpec {
            cells: on(&list[..], &UFBD),
            marks: UD,
        })
        .collect()
}

pub(super) fn ud_stage_tables(used: &[&MoveDef], cache: &Cache, threads: usize) -> Vec<Table> {
    let g = ud_stage_groups();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    ["dw6-t-ud", "dw6-x-ud"]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            Table::new(
                name,
                vec![Group {
                    slot: i,
                    cells: g[i].cells.clone(),
                    k: 8,
                }],
                &[vec![ud_goal()]],
                &dest,
                cache,
                threads,
            )
        })
        .collect()
}

/// Move filters of the stages.
pub(super) fn five_moves(name: &str) -> bool {
    !name.starts_with('3')
}

pub(super) fn ud_pair_moves(name: &str) -> bool {
    // No turns of L and R, no quarter turns of the wide moves of U D F B,
    // no three-layer quarter turns of U D F B.
    let face = name.trim_start_matches('3').chars().next().unwrap_or(' ');
    let quarter = !name.ends_with('2');
    let wide = name.contains('w');
    let three = name.starts_with('3');
    match face {
        'L' | 'R' => wide,
        _ => !quarter || !(wide || three),
    }
}

pub(super) fn ud_stage_moves(name: &str) -> bool {
    if name.starts_with('3') {
        return false;
    }
    let face = name.chars().next().unwrap_or(' ');
    let wide = name.contains('w');
    let quarter = !name.ends_with('2');
    !(wide && quarter && matches!(face, 'U' | 'F' | 'B' | 'D'))
}
