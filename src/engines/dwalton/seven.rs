//! The 7x7x7 center stages in a row.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube777.py (`reduce_555` and the stages it
//! calls), MIT, Copyright (c) 2019 Daniel Walton; this file is
//! AGPL-3.0-or-later, see NOTICE.md. The stages that the reference hands to
//! its 5x5x5 center stages are searched here over the same tables; the moves
//! of the fake 5x5x5 are the 7x7x7 turns of its first, third (inner) or
//! second (outer) layer.

use std::time::Instant;

use super::geometry::{bit, catalog, from_ref_all, MoveDef, D, L, R, U};
use super::pattern::{Group, Table};
use super::seven_final_tables as fin;
use super::seven_lr_tables as lr;
use super::seven_stage::{self as ss, build, Obliques, RefTable};
use super::seven_ud_tables as ud;
use super::six::Done;
use super::six_stage2::ud_stage_moves;
use super::stage::{select, solve, GroupSpec, PatStage};
use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};
use crate::reduction::cache::Cache;

const FORMAT: u32 = 1;
const LR: u8 = bit(L) | bit(R);
const UD: u8 = bit(U) | bit(D);

const INNER_T: [usize; 4] = [18, 24, 26, 32];
const INNER_X: [usize; 4] = [17, 19, 31, 33];
const OUTER_T: [usize; 4] = [11, 23, 27, 39];
const OUTER_X: [usize; 4] = [9, 13, 37, 41];
const LEFT: [usize; 4] = [10, 30, 20, 40];
const MIDDLE: [usize; 4] = [11, 23, 27, 39];
const RIGHT: [usize; 4] = [12, 16, 34, 38];

/// Reference faces U L F R B D, as sides 0 to 5.
const ALL: [usize; 6] = [0, 1, 2, 3, 4, 5];
/// U F B D.
const UFBD: [usize; 4] = [0, 2, 4, 5];

/// Reference stickers of the local positions on the sides, side by side.
fn on(local: &[usize], sides: &[usize]) -> Vec<usize> {
    from_ref_all(
        7,
        &sides
            .iter()
            .flat_map(|&s| local.iter().map(move |&c| 49 * s + c))
            .collect::<Vec<_>>(),
    )
}

fn lr_goal() -> u32 {
    (4..8).chain(12..16).map(|i| 1u32 << i).sum()
}

fn ud_goal() -> u32 {
    (0..4).chain(12..16).map(|i| 1u32 << i).sum()
}

pub(super) struct Seven {
    defs: Vec<MoveDef>,
    cache: Cache,
    threads: usize,
}

fn node_of(stage: &PatStage, cube: &Facelets) -> super::stage::Node {
    stage.node_of(|s| cube.stickers()[s] as u8)
}

impl Seven {
    pub(super) fn new(threads: usize) -> Seven {
        Seven {
            defs: catalog(PuzzleKind::Cube7),
            cache: Cache::new(crate::reduction::cache::default_dir(), FORMAT),
            threads,
        }
    }

    fn apply(cube: &Facelets, moves: &[String]) -> Facelets {
        let seq: MoveSequence = moves.join(" ").parse().expect("moves of the catalog");
        cube.apply(&seq).expect("moves fit the cube")
    }

    /// A 5x5x5 center stage: t-centers and x-centers, 8 of 24 marked each.
    #[allow(clippy::too_many_arguments)]
    fn five_stage(
        &self,
        cube: &Facelets,
        names: [&str; 2],
        local: [&[usize]; 2],
        sides: &[usize],
        marks: u8,
        goal: u32,
        allow: fn(&str) -> bool,
        deadline: Option<Instant>,
    ) -> Option<Vec<String>> {
        let used = select(&self.defs, allow);
        let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
        let specs: Vec<GroupSpec> = local
            .iter()
            .map(|l| GroupSpec {
                cells: on(l, sides),
                marks,
            })
            .collect();
        let tables: Vec<Table> = (0..2)
            .map(|i| {
                Table::new(
                    names[i],
                    vec![Group {
                        slot: i,
                        cells: specs[i].cells.clone(),
                        k: 8,
                    }],
                    &[vec![goal]],
                    &dest,
                    &self.cache,
                    self.threads,
                )
            })
            .collect();
        let stage = PatStage::new(specs, &used, tables.iter().collect(), None);
        solve(&stage, node_of(&stage, cube), self.threads, deadline)
    }

    /// Pairs the oblique centers of `sides` for the colors `marks`.
    fn pair_stage(
        &self,
        cube: &Facelets,
        sides: &[usize],
        marks: u8,
        cost: fn(i32) -> i32,
        allow: fn(&str) -> bool,
        deadline: Option<Instant>,
    ) -> Option<Vec<String>> {
        let used = select(&self.defs, allow);
        let specs: Vec<GroupSpec> = [&LEFT, &MIDDLE, &RIGHT]
            .iter()
            .map(|l| GroupSpec {
                cells: on(&l[..], sides),
                marks,
            })
            .collect();
        let extra = Obliques { cost };
        let stage = PatStage::new(specs, &used, Vec::new(), Some(&extra));
        solve(&stage, node_of(&stage, cube), self.threads, deadline)
    }

    /// A stage over the tables of the reference.
    fn table_stage(
        &self,
        cube: &Facelets,
        refs: &[RefTable],
        allow: fn(&str) -> bool,
        deadline: Option<Instant>,
    ) -> Option<Vec<String>> {
        let used = select(&self.defs, allow);
        let dest: Vec<&[usize]> = used.iter().map(|d| &d.dest[..]).collect();
        let built = build(refs, &dest);
        let stage = PatStage::new(built.specs, &used, built.tables.iter().collect(), None);
        solve(&stage, node_of(&stage, cube), self.threads, deadline)
    }

    /// All center stages. Returns the cube after them and the moves of each.
    pub(super) fn centers(
        &self,
        cube: &Facelets,
        deadline: Option<Instant>,
    ) -> Option<(Facelets, Vec<Done>)> {
        let mut cube = cube.clone();
        let mut done: Vec<Done> = Vec::new();
        let mut run = |cube: &mut Facelets, name: &'static str, m: Option<Vec<String>>| {
            let m = m?;
            *cube = Self::apply(cube, &m);
            done.push((name, m));
            Some(())
        };

        let m = self.five_stage(
            &cube,
            ["dw7-t-lr-in", "dw7-x-lr-in"],
            [&INNER_T, &INNER_X],
            &ALL,
            LR,
            lr_goal(),
            ss::inner_moves,
            deadline,
        );
        run(&mut cube, "LR inner centers staged", m)?;
        let m = self.pair_stage(&cube, &ALL, LR, ss::lr_cost, ss::lr_pair_moves, deadline);
        run(&mut cube, "LR obliques paired", m)?;
        let m = self.five_stage(
            &cube,
            ["dw7-t-lr-out", "dw7-x-lr-out"],
            [&OUTER_T, &OUTER_X],
            &ALL,
            LR,
            lr_goal(),
            |n| !n.starts_with('3'),
            deadline,
        );
        run(&mut cube, "LR centers staged", m)?;

        let m = self.five_stage(
            &cube,
            ["dw7-t-ud-in", "dw7-x-ud-in"],
            [&INNER_T, &INNER_X],
            &UFBD,
            UD,
            ud_goal(),
            ss::inner_ud_moves,
            deadline,
        );
        run(&mut cube, "UD inner centers staged", m)?;
        let m = self.pair_stage(&cube, &UFBD, UD, ss::ud_cost, ss::ud_pair_moves, deadline);
        run(&mut cube, "UD obliques paired", m)?;
        let m = self.five_stage(
            &cube,
            ["dw7-t-ud-out", "dw7-x-ud-out"],
            [&OUTER_T, &OUTER_X],
            &UFBD,
            UD,
            ud_goal(),
            ud_stage_moves,
            deadline,
        );
        run(&mut cube, "UD centers staged", m)?;

        let refs = |cells, targets, mark| RefTable {
            cells,
            targets,
            mark,
        };
        let m = self.table_stage(
            &cube,
            &[
                refs(&lr::CELLS_41, &lr::TARGETS_41, 'L'),
                refs(&lr::CELLS_42, &lr::TARGETS_42, 'L'),
                refs(&lr::CELLS_43, &lr::TARGETS_43, 'L'),
                refs(&lr::CELLS_44, &lr::TARGETS_44, 'L'),
            ],
            ss::step40_moves,
            deadline,
        );
        run(&mut cube, "LR centers vertical bars", m)?;
        let m = self.table_stage(
            &cube,
            &[
                refs(&ud::CELLS_51, &ud::TARGETS_51, 'U'),
                refs(&ud::CELLS_52, &ud::TARGETS_52, 'U'),
                refs(&ud::CELLS_53, &ud::TARGETS_53, 'U'),
                refs(&ud::CELLS_54, &ud::TARGETS_54, 'U'),
                refs(&ud::CELLS_55, &ud::TARGETS_55, 'L'),
            ],
            ss::step50_moves,
            deadline,
        );
        run(&mut cube, "LR solved, UD vertical bars", m)?;
        let m = self.table_stage(
            &cube,
            &[
                refs(&fin::CELLS_61, &fin::TARGETS_61, 'U'),
                refs(&fin::CELLS_62, &fin::TARGETS_62, 'L'),
                refs(&fin::CELLS_65, &fin::TARGETS_65, 'F'),
                refs(&fin::CELLS_66, &fin::TARGETS_66, 'F'),
            ],
            ss::step60_moves,
            deadline,
        );
        run(&mut cube, "centers daisy solved", m)?;
        Some((cube, done))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::scramble;

    #[test]
    fn the_center_stages_run() {
        let seven = Seven::new(crate::reduction::par::threads());
        for _ in 0..2 {
            let cube = Facelets::solved(PuzzleKind::Cube7)
                .apply(&scramble(PuzzleKind::Cube7, 100))
                .unwrap();
            let started = std::time::Instant::now();
            let (_, done) = seven.centers(&cube, None).expect("stages");
            let lens: Vec<(&str, usize)> = done.iter().map(|d| (d.0, d.1.len())).collect();
            println!("{lens:?} in {:?}", started.elapsed());
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    fn clump(i: usize, orbit: usize) -> usize {
        // 0-based row or column of a 7x7 -> clump number of the fake 5x5.
        match (orbit, i) {
            (_, 0) => 0,
            (0, 1) => 1,
            (0, 2..=4) => 2,
            (0, 5) => 3,
            (1, 1 | 2) => 1,
            (1, 3) => 2,
            (1, 4 | 5) => 3,
            _ => 4,
        }
    }

    #[test]
    fn lifted_moves_keep_blocks_together() {
        for orbit in 0..2usize {
            let defs = catalog(PuzzleKind::Cube7);
            let allowed = |n: &str| {
                if orbit == 0 {
                    !n.starts_with('3')
                } else {
                    !n.contains('w') || n.starts_with('3')
                }
            };
            let used = select(&defs, allowed);
            let mut origin: Vec<usize> = (0..294).collect();
            let mut x = 99u64;
            for _ in 0..25 {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let d = used[(x >> 33) as usize % used.len()];
                let mut next = origin.clone();
                for (s, &to) in d.dest.iter().enumerate() {
                    next[to] = origin[s];
                }
                origin = next;
            }
            let block = |s: usize| (s / 49, clump(s % 49 / 7, orbit), clump(s % 7, orbit));
            let mut map = std::collections::HashMap::new();
            for (s, &from) in origin.iter().enumerate() {
                let b = block(s);
                let o = block(from);
                if let Some(prev) = map.insert(b, o) {
                    assert_eq!(prev, o, "orbit {orbit}: block {b:?} is split");
                }
            }
        }
    }
}
