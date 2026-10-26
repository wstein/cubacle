//! Building blocks of the 7x7x7 center stages: pruning tables read off the
//! reference's sticker lists and targets, the oblique pairing heuristics,
//! and the move filters of the steps.
//!
//! Ported from dwalton76/rubiks-cube-NxNxN-solver
//! rubikscubennnsolver/RubiksCube777.py (the illegal move lists of the steps)
//! and ida_search_777.c (the unpaired obliques heuristics), MIT, Copyright (c)
//! 2019 Daniel Walton; this file is AGPL-3.0-or-later, see NOTICE.md. The
//! tables are built by search here, not read from the reference's files.

use super::geometry::*;
use super::pattern::{Group, Table};
use super::stage::{Extra, GroupSpec};

/// A table of the reference: the stickers it reads and the states it solves.
pub(super) struct RefTable {
    pub cells: &'static [usize],
    pub targets: &'static [&'static str],
    /// The character of the targets that marks a sticker.
    pub mark: char,
}

/// Tables with the sticker groups they read.
pub(super) struct Built {
    pub specs: Vec<GroupSpec>,
    pub tables: Vec<Table>,
}

fn color_of(mark: char) -> usize {
    match mark {
        'U' => U,
        'R' => R,
        'F' => F,
        'D' => D,
        'L' => L,
        _ => B,
    }
}

/// The cells of a table cut into closed groups of at most 32: the orbits of
/// the moves, packed together.
fn groups_of(cells: &[usize], dest: &[&[usize]]) -> Vec<Vec<usize>> {
    let n = cells.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for d in dest {
        for (i, &c) in cells.iter().enumerate() {
            let j = cells
                .iter()
                .position(|&t| t == d[c])
                .expect("the sticker set is closed under the moves");
            let (a, b) = (find(&mut parent, i), find(&mut parent, j));
            parent[a.max(b)] = a.min(b);
        }
    }
    let mut orbits: Vec<Vec<usize>> = Vec::new();
    let mut root_of = vec![usize::MAX; n];
    for i in 0..n {
        let r = find(&mut parent, i);
        if root_of[r] == usize::MAX {
            root_of[r] = orbits.len();
            orbits.push(Vec::new());
        }
        orbits[root_of[r]].push(i);
    }
    let mut packed: Vec<Vec<usize>> = Vec::new();
    for orbit in orbits {
        assert!(orbit.len() <= 32, "an orbit of {} stickers", orbit.len());
        match packed.iter_mut().find(|g| g.len() + orbit.len() <= 32) {
            Some(group) => group.extend(orbit),
            None => packed.push(orbit),
        }
    }
    for g in &mut packed {
        g.sort_unstable();
    }
    packed
}

/// Builds the tables of `defs` for the moves `dest`; groups that two tables
/// share are one group of the search.
pub(super) fn build(defs: &[RefTable], dest: &[&[usize]]) -> Built {
    let mut specs: Vec<GroupSpec> = Vec::new();
    let mut tables = Vec::new();
    for def in defs {
        let cells = from_ref_all(7, def.cells);
        let marks = bit(color_of(def.mark));
        let mut groups = Vec::new();
        let mut parts = Vec::new();
        for part in groups_of(&cells, dest) {
            let of: Vec<usize> = part.iter().map(|&i| cells[i]).collect();
            let slot = match specs.iter().position(|s| s.cells == of && s.marks == marks) {
                Some(slot) => slot,
                None => {
                    specs.push(GroupSpec {
                        cells: of.clone(),
                        marks,
                    });
                    specs.len() - 1
                }
            };
            groups.push(Group {
                slot,
                cells: of,
                k: 0,
            });
            parts.push(part);
        }
        let goals: Vec<Vec<u32>> = def
            .targets
            .iter()
            .map(|t| {
                parts
                    .iter()
                    .map(|part| {
                        part.iter()
                            .enumerate()
                            .filter(|&(_, &i)| t.as_bytes()[i] as char == def.mark)
                            .map(|(bit, _)| 1u32 << bit)
                            .sum()
                    })
                    .collect()
            })
            .collect();
        tables.push(Table::reachable(groups, &goals, dest));
    }
    Built { specs, tables }
}

/// The oblique pairs: groups 0, 1, 2 are the left, middle and right
/// obliques; every marked middle oblique needs both neighbors marked.
pub(super) struct Obliques {
    /// Estimated moves for a number of unpaired obliques.
    pub cost: fn(i32) -> i32,
}

impl Extra for Obliques {
    fn eval(&self, m: &[u32]) -> i32 {
        if m[1] & !(m[0] & m[2]) == 0 {
            return 0;
        }
        let paired = (m[1] & m[0]).count_ones() + (m[1] & m[2]).count_ones();
        (self.cost)(16 - paired as i32).max(1)
    }
}

/// The reference's table from the number of unpaired L/R obliques to moves.
pub(super) fn lr_cost(unpaired: i32) -> i32 {
    match unpaired {
        0 => 0,
        1 | 2 => 1,
        3..=8 => unpaired + 1,
        9..=11 => unpaired,
        12 => 11,
        _ => 12,
    }
}

/// The same for the U/D obliques.
pub(super) fn ud_cost(unpaired: i32) -> i32 {
    match unpaired {
        0 => 0,
        1 | 2 => 1,
        3 => 3,
        4..=7 => unpaired + 1,
        8 | 9 => 10,
        10 | 11 => 11,
        _ => 12,
    }
}

fn parts(name: &str) -> (char, bool, bool) {
    let wide = name.contains('w');
    let quarter = !name.ends_with('2');
    (
        name.trim_start_matches('3').chars().next().unwrap_or(' '),
        wide,
        quarter,
    )
}

/// Inner centers, LR: the 5x5x5 turns with two layers made three.
pub(super) fn inner_moves(name: &str) -> bool {
    !name.contains('w') || name.starts_with('3')
}

/// Inner centers, UD: the same without quarter turns of U D F B wide moves.
pub(super) fn inner_ud_moves(name: &str) -> bool {
    let (face, wide, quarter) = parts(name);
    inner_moves(name) && !(wide && quarter && matches!(face, 'U' | 'F' | 'B' | 'D'))
}

/// L/R oblique pairing: no three-layer quarter turns about U D F B.
pub(super) fn lr_pair_moves(name: &str) -> bool {
    let (face, _, quarter) = parts(name);
    !(name.starts_with('3') && quarter && matches!(face, 'U' | 'D' | 'F' | 'B'))
}

/// U/D oblique pairing.
pub(super) fn ud_pair_moves(name: &str) -> bool {
    let (face, wide, quarter) = parts(name);
    if name.starts_with('3') && quarter {
        return false;
    }
    if wide && quarter && matches!(face, 'U' | 'D' | 'F' | 'B') {
        return false;
    }
    wide || !matches!(face, 'L' | 'R')
}

/// Step 40: L R B turns and half turns of the wide moves.
pub(super) fn step40_moves(name: &str) -> bool {
    let (face, wide, quarter) = parts(name);
    let plain_blocked = !wide && matches!(face, 'U' | 'D' | 'F');
    !(plain_blocked || wide && quarter)
}

/// Step 50: U B turns, L2 R2 and half turns of the wide moves but U D.
pub(super) fn step50_moves(name: &str) -> bool {
    let (face, wide, quarter) = parts(name);
    if wide {
        return !quarter && !matches!(face, 'U' | 'D');
    }
    match face {
        'U' | 'B' => true,
        'L' | 'R' => !quarter,
        _ => false,
    }
}

/// Step 60: F B turns, U2 D2 L2 R2 and half turns of the wide moves but F B.
pub(super) fn step60_moves(name: &str) -> bool {
    let (face, wide, quarter) = parts(name);
    if wide {
        return !quarter && !matches!(face, 'F' | 'B');
    }
    match face {
        'F' | 'B' => true,
        _ => !quarter,
    }
}
