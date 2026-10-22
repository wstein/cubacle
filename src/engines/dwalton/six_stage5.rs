//! 6x6x6 stages 5 and 6: solve the L/R inner x-centres and obliques so that
//! the L/R centres are reducible to a 5x5x5, then the same for U/D and F/B.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube666.py (`LookupTable666Step50LRCenters`,
//! `LookupTable666UDInnerXCenterAndObliqueEdges`,
//! `LookupTable666FBInnerXCenterAndObliqueEdges`,
//! `LookupTable666LRObliqueEdges`), MIT, Copyright (c) 2019 Daniel Walton;
//! this file is AGPL-3.0-or-later, see NOTICE.md. The tables are built by
//! search here. The 4x4x4 edge tables of the reference's step 5 are not used:
//! the 4x4x4 engine pairs the inner wings.

use super::geometry::*;
use super::pattern::{Group, Table};
use super::stage::GroupSpec;
use crate::reduction::cache::Cache;

/// Targets of step 50 over `STEP50_CELLS`, `L` for the L colour.
const STEP50_TARGETS: [&str; 36] = [
    "LLLLLLLLLLLLxxxxxxxxxxxx",
    "LLLLLLLLLLxxLLxxxxxxxxxx",
    "LLLLLLLLLLxxxxxxxxxxxxLL",
    "LLLLLxLLLxLLxxLxxxLxxxxx",
    "LLLLLxLLLxLLxxxxxLxxxLxx",
    "LLLLLxLLLxxxLLLxxxLxxxxx",
    "LLLLLxLLLxxxLLxxxLxxxLxx",
    "LLLLLxLLLxxxxxLxxxLxxxLL",
    "LLLLLxLLLxxxxxxxxLxxxLLL",
    "LLxLLLxLLLLLxxLxxxLxxxxx",
    "LLxLLLxLLLLLxxxxxLxxxLxx",
    "LLxLLLxLLLxxLLLxxxLxxxxx",
    "LLxLLLxLLLxxLLxxxLxxxLxx",
    "LLxLLLxLLLxxxxLxxxLxxxLL",
    "LLxLLLxLLLxxxxxxxLxxxLLL",
    "LLxLLxxLLxLLxxLxxLLxxLxx",
    "LLxLLxxLLxxxLLLxxLLxxLxx",
    "LLxLLxxLLxxxxxLxxLLxxLLL",
    "xxLLLLLLLLLLLLxxxxxxxxxx",
    "xxLLLLLLLLLLxxxxxxxxxxLL",
    "xxLLLLLLLLxxLLxxxxxxxxLL",
    "xxLLLxLLLxLLLLLxxxLxxxxx",
    "xxLLLxLLLxLLLLxxxLxxxLxx",
    "xxLLLxLLLxLLxxLxxxLxxxLL",
    "xxLLLxLLLxLLxxxxxLxxxLLL",
    "xxLLLxLLLxxxLLLxxxLxxxLL",
    "xxLLLxLLLxxxLLxxxLxxxLLL",
    "xxxLLLxLLLLLLLLxxxLxxxxx",
    "xxxLLLxLLLLLLLxxxLxxxLxx",
    "xxxLLLxLLLLLxxLxxxLxxxLL",
    "xxxLLLxLLLLLxxxxxLxxxLLL",
    "xxxLLLxLLLxxLLLxxxLxxxLL",
    "xxxLLLxLLLxxLLxxxLxxxLLL",
    "xxxLLxxLLxLLLLLxxLLxxLxx",
    "xxxLLxxLLxLLxxLxxLLxxLLL",
    "xxxLLxxLLxxxLLLxxLLxxLLL",
];

const STEP50_CELLS: [usize; 24] = [
    45, 46, 50, 51, 52, 53, 56, 57, 58, 59, 63, 64, 117, 118, 122, 123, 124, 125, 128, 129, 130,
    131, 135, 136,
];

/// The cells of `list` (reference numbers) that are also in `of`.
fn within(list: &[usize], of: &[usize]) -> Vec<usize> {
    list.iter().copied().filter(|i| of.contains(i)).collect()
}

/// Splits a list of reference stickers into inner x-centres, left and right
/// obliques (cubacle indices each).
fn split3(list: &[usize]) -> [Vec<usize>; 3] {
    [
        from_ref_all(6, &within(list, &INNER_X_666)),
        from_ref_all(6, &within(list, &LEFT_OBLIQUE_666)),
        from_ref_all(6, &within(list, &RIGHT_OBLIQUE_666)),
    ]
}

/// The masks of a target string over `list`, one per group of `split3`, for
/// the character `mark`.
fn masks_of(list: &[usize], target: &str, mark: char) -> Vec<u32> {
    let parts = [
        &INNER_X_666[..],
        &LEFT_OBLIQUE_666[..],
        &RIGHT_OBLIQUE_666[..],
    ];
    parts
        .iter()
        .map(|part| {
            let mut mask = 0;
            let mut bit = 0;
            for (i, &cell) in list.iter().enumerate() {
                if part.contains(&cell) {
                    if target.as_bytes()[i] as char == mark {
                        mask |= 1u32 << bit;
                    }
                    bit += 1;
                }
            }
            mask
        })
        .collect()
}

fn specs(list: &[usize], marks: u8) -> Vec<GroupSpec> {
    split3(list)
        .into_iter()
        .map(|cells| GroupSpec { cells, marks })
        .collect()
}

pub(super) fn step50_groups() -> Vec<GroupSpec> {
    specs(&STEP50_CELLS, bit(L))
}

pub(super) fn step50_table(used: &[&MoveDef], cache: &Cache, threads: usize) -> Table {
    let g = step50_groups();
    let goals: Vec<Vec<u32>> = STEP50_TARGETS
        .iter()
        .map(|t| masks_of(&STEP50_CELLS, t, 'L'))
        .collect();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    Table::new(
        "dw6-step50",
        g.iter()
            .enumerate()
            .map(|(i, s)| Group {
                slot: i,
                cells: s.cells.clone(),
                k: 4,
            })
            .collect(),
        &goals,
        &dest,
        cache,
        threads,
    )
}

/// Moves of steps 50 and 60: faces, and wide half turns.
pub(super) fn step50_moves(name: &str) -> bool {
    !name.contains('w') || name.ends_with('2')
}

const UD_CELLS: [usize; 24] = [
    9, 10, 14, 15, 16, 17, 20, 21, 22, 23, 27, 28, 189, 190, 194, 195, 196, 197, 200, 201, 202,
    203, 207, 208,
];
const FB_CELLS: [usize; 24] = [
    81, 82, 86, 87, 88, 89, 92, 93, 94, 95, 99, 100, 153, 154, 158, 159, 160, 161, 164, 165, 166,
    167, 171, 172,
];
const UD_TARGETS: [&str; 2] = ["DDDUUDDUUDDDUUUDDUUDDUUU", "UUUUUUUUUUUUDDDDDDDDDDDD"];
const FB_TARGETS: [&str; 2] = ["BBBFFBBFFBBBFFFBBFFBBFFF", "FFFFFFFFFFFFBBBBBBBBBBBB"];
const LR_OBL_INNER: [usize; 32] = [
    15, 16, 21, 22, 45, 46, 50, 53, 56, 59, 63, 64, 87, 88, 93, 94, 117, 118, 122, 125, 128, 131,
    135, 136, 159, 160, 165, 166, 195, 196, 201, 202,
];
const LR_OBL_TARGETS: [&str; 2] = [
    "UUUULLLLLLLLFFFFRRRRRRRRBBBBDDDD",
    "UUUURRRRRRRRFFFFLLLLLLLLBBBBDDDD",
];

/// Step 60 groups: 0..3 U/D (mark U), 3..6 F/B (mark F), 6 and 7 the L/R
/// left and right obliques (mark L), 8 to 10 the inner x-centres of U F B D
/// (marks U, F, B).
pub(super) fn step60_groups() -> Vec<GroupSpec> {
    let mut out = specs(&UD_CELLS, bit(U));
    out.extend(specs(&FB_CELLS, bit(F)));
    let lr = |part: &[usize]| from_ref_all(6, &within(&LR_OBL_INNER, part));
    for part in [&LEFT_OBLIQUE_666, &RIGHT_OBLIQUE_666] {
        out.push(GroupSpec {
            cells: lr(&part[..]),
            marks: bit(L),
        });
    }
    let inner = from_ref_all(6, &within(&LR_OBL_INNER, &INNER_X_666));
    for face in [U, F, B] {
        out.push(GroupSpec {
            cells: inner.clone(),
            marks: bit(face),
        });
    }
    out
}

pub(super) fn step60_tables(used: &[&MoveDef]) -> Vec<Table> {
    let g = step60_groups();
    let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
    let grp = |slots: &[usize]| -> Vec<Group> {
        slots
            .iter()
            .map(|&slot| Group {
                slot,
                cells: g[slot].cells.clone(),
                k: 0,
            })
            .collect()
    };
    let ud: Vec<Vec<u32>> = UD_TARGETS
        .iter()
        .map(|t| masks_of(&UD_CELLS, t, 'U'))
        .collect();
    let fb: Vec<Vec<u32>> = FB_TARGETS
        .iter()
        .map(|t| masks_of(&FB_CELLS, t, 'F'))
        .collect();
    // Third table: the LR obliques (left, right) and the inner x-centres.
    let lr: Vec<Vec<u32>> = LR_OBL_TARGETS
        .iter()
        .map(|t| {
            let m = masks_of(&LR_OBL_INNER, t, 'L');
            let inner = |mark: char| {
                let mut mask = 0u32;
                let mut bit = 0;
                for (i, &cell) in LR_OBL_INNER.iter().enumerate() {
                    if INNER_X_666.contains(&cell) {
                        if t.as_bytes()[i] as char == mark {
                            mask |= 1 << bit;
                        }
                        bit += 1;
                    }
                }
                mask
            };
            vec![m[1], m[2], inner('U'), inner('F'), inner('B')]
        })
        .collect();
    vec![
        Table::reachable(grp(&[0, 1, 2]), &ud, &dest),
        Table::reachable(grp(&[3, 4, 5]), &fb, &dest),
        Table::reachable(grp(&[6, 7, 8, 9, 10]), &lr, &dest),
    ]
}

/// Moves of step 60: no quarter turns of L and R, no three-layer half turns
/// of U D F B.
pub(super) fn step60_moves(name: &str) -> bool {
    step50_moves(name)
        && !matches!(
            name,
            "L" | "L'" | "R" | "R'" | "3Uw2" | "3Dw2" | "3Fw2" | "3Bw2"
        )
}

/// The inner x-centres show the colour of their face and the obliques are
/// paired (the reference's `reduced_to_555`, centre part).
#[cfg(test)]
pub(super) fn centres_reduced(cube: &crate::puzzle::Facelets) -> bool {
    let st = cube.stickers();
    let face_of = |i: usize| st[from_ref(6, i)] as usize;
    let inner_ok = INNER_X_666
        .iter()
        .all(|&i| face_of(i) == from_ref(6, i) / 36);
    let pairs_ok = LEFT_OBLIQUE_666
        .iter()
        .zip(&RIGHT_OBLIQUE_666)
        .all(|(&a, &b)| face_of(a) == face_of(b));
    inner_ok && pairs_ok
}
