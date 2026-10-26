//! Phase 1: the two opposite colors of both center kinds onto two faces.
//!
//! Ported from cs0x7f/cube555 src/Phase1Center.java and src/Phase1Search.java,
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md.
//!
//! The x-centers and the t-centers each give C(24,8) = 735,471 states, taken
//! by 16 symmetries to 46,935 (t) and 46,371 (x) classes. A state is
//! `class << 4 | symmetry`. Three pruning tables: t classes, x classes, and
//! x classes by raw t state, a hash of 2^26 entries of depth up to 7.

use super::build::Build;
use super::cubie::move_centers;
use super::cubie_sym::Syms;
use super::mv;
use crate::reduction::bfs::Space;
use crate::reduction::comb::{get_comb, set_comb, swap2, swap4};
use crate::reduction::hashed::Hashed;
use crate::reduction::ida::Stage;
use crate::reduction::nibbles::Prun;

pub(super) const MOVES: [u8; 36] = {
    let mut m = [0u8; 36];
    let mut i = 0;
    while i < 36 {
        m[i] = i as u8;
        i += 1;
    }
    m
};
const RAW: usize = 735_471;
const T_CLASSES: usize = 46_935;
const X_CLASSES: usize = 46_371;
const HASH_BITS: u32 = 26;
const HASH_DEPTH: u32 = 7;

/// Centers of one phase-1 state, as slot arrays (`0` marked, `-1` not).
struct Center {
    t: [i32; 24],
    x: [i32; 24],
}

impl Center {
    fn new() -> Center {
        let mut c = Center {
            t: [0; 24],
            x: [0; 24],
        };
        c.set_t(0);
        c.set_x(0);
        c
    }

    fn set_t(&mut self, idx: usize) {
        set_comb(&mut self.t, (RAW - 1 - idx) as i32, 8, 24);
    }
    fn get_t(&self) -> usize {
        RAW - 1 - get_comb(&self.t, 8, 24) as usize
    }
    fn set_x(&mut self, idx: usize) {
        set_comb(&mut self.x, (RAW - 1 - idx) as i32, 8, 24);
    }
    fn get_x(&self) -> usize {
        RAW - 1 - get_comb(&self.x, 8, 24) as usize
    }

    fn do_conj(&mut self, conj: u8) {
        match conj {
            0 => {
                move_centers(&mut self.x, &mut self.t, mv::RW1);
                move_centers(&mut self.x, &mut self.t, mv::LW3);
                swap4(&mut self.t, [0, 14, 4, 8], 0, false);
                swap4(&mut self.t, [2, 12, 6, 10], 0, false);
            }
            1 => {
                move_centers(&mut self.x, &mut self.t, mv::UW2);
                move_centers(&mut self.x, &mut self.t, mv::DW2);
                swap4(&mut self.t, [9, 21, 13, 17], 1, false);
                swap4(&mut self.t, [11, 23, 15, 19], 1, false);
            }
            _ => {
                for (a, b) in [
                    (1, 3),
                    (5, 7),
                    (9, 11),
                    (13, 15),
                    (16, 20),
                    (17, 23),
                    (18, 22),
                    (19, 21),
                ] {
                    swap2(&mut self.t, a, b);
                }
                for (a, b) in [
                    (0, 1),
                    (2, 3),
                    (4, 5),
                    (6, 7),
                    (8, 9),
                    (10, 11),
                    (12, 13),
                    (14, 15),
                    (16, 21),
                    (17, 20),
                    (18, 23),
                    (19, 22),
                ] {
                    swap2(&mut self.x, a, b);
                }
            }
        }
    }

    /// The conjugations that step through the 16 symmetries in order.
    fn next_sym(&mut self, sym: usize) {
        self.do_conj(0);
        if sym & 3 == 3 {
            self.do_conj(1);
        }
        if sym & 7 == 7 {
            self.do_conj(2);
        }
    }
}

/// Symmetry classes of one center kind.
struct Classes {
    raw2sym: Vec<u32>,
    sym2raw: Vec<u32>,
    self_sym: Vec<u32>,
}

fn classes(
    expected: usize,
    set: impl Fn(&mut Center, usize),
    get: impl Fn(&Center) -> usize,
) -> Classes {
    let mut c = Center::new();
    let mut raw2sym = vec![0u32; RAW];
    let mut sym2raw = vec![0u32; expected * 16];
    let mut self_sym = Vec::with_capacity(expected);
    for i in 0..RAW {
        // As in the original, 0 means "not seen"; only raw state 0 maps to 0.
        if raw2sym[i] != 0 {
            continue;
        }
        let class = self_sym.len();
        self_sym.push(0u32);
        set(&mut c, i);
        for sym in 0..16 {
            let idx = get(&c);
            raw2sym[idx] = (class << 4 | sym) as u32;
            sym2raw[class << 4 | sym] = idx as u32;
            if idx == i {
                self_sym[class] |= 1 << sym;
            }
            c.next_sym(sym);
        }
    }
    assert_eq!(self_sym.len(), expected, "class count");
    Classes {
        raw2sym,
        sym2raw,
        self_sym,
    }
}

/// `table[class * 36 + move]`: the state after the move, `class << 4 | sym`.
fn move_table(
    cls: &Classes,
    set: impl Fn(&mut Center, usize),
    get: impl Fn(&Center) -> usize,
) -> Vec<u32> {
    let mut c = Center::new();
    let mut table = Vec::with_capacity(cls.self_sym.len() * 36);
    for class in 0..cls.self_sym.len() {
        for m in 0..36u8 {
            set(&mut c, cls.sym2raw[class << 4] as usize);
            move_centers(&mut c.x, &mut c.t, m);
            table.push(cls.raw2sym[get(&c)]);
        }
    }
    table
}

/// The classes of one center kind, with plain moves, as a search space.
struct Sym<'a> {
    table: &'a [u32],
}

impl Space for Sym<'_> {
    fn size(&self) -> usize {
        self.table.len() / 36
    }
    fn solved(&self) -> Vec<usize> {
        vec![0]
    }
    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        for m in 0..36 {
            f((self.table[state * 36 + m] >> 4) as usize, 0);
        }
    }
}

/// x classes by raw t states, for the hashed table.
struct Joint<'a> {
    p: &'a Moves1,
    syms: &'a Syms,
}

impl Joint<'_> {
    /// The raw t state after move `m` of raw t state `raw`, seen from `conj`.
    fn t_moved(&self, raw: usize, m: usize, conj: usize) -> usize {
        let idx = self.p.t_raw2sym[raw] as usize;
        let sym = idx & 0xf;
        let mut r =
            self.p.t_move[(idx >> 4) * 36 + usize::from(self.p.sym_move[sym * 36 + m])] as usize;
        r = r & !0xf | usize::from(self.syms.mult[r & 0xf][sym]);
        let moved = self.p.t_sym2raw[r] as usize;
        self.t_conj(moved, conj)
    }

    fn t_conj(&self, raw: usize, conj: usize) -> usize {
        let idx = self.p.t_raw2sym[raw] as usize;
        let idx = idx & !0xf | usize::from(self.syms.mult_inv[idx & 0xf][conj]);
        self.p.t_sym2raw[idx] as usize
    }
}

impl Hashed for Joint<'_> {
    fn solved(&self) -> Vec<u64> {
        vec![0]
    }

    fn neighbors<F: FnMut(u64)>(&self, key: u64, f: &mut F) {
        let class = key as usize / RAW;
        let raw = key as usize % RAW;
        for m in 0..36 {
            let moved = self.p.x_move[class * 36 + m] as usize;
            let new_class = moved >> 4;
            let new_raw = self.t_moved(raw, m, moved & 0xf);
            f((new_class * RAW + new_raw) as u64);
            let mut state = self.p.x_self_sym[new_class];
            let mut j = 1;
            loop {
                state >>= 1;
                if state == 0 {
                    break;
                }
                if state & 1 == 1 {
                    f((new_class * RAW + self.t_conj(new_raw, j)) as u64);
                }
                j += 1;
            }
        }
    }
}

/// The move tables of the phase.
struct Moves1 {
    t_move: Vec<u32>,
    t_raw2sym: Vec<u32>,
    t_sym2raw: Vec<u32>,
    x_move: Vec<u32>,
    x_raw2sym: Vec<u32>,
    x_self_sym: Vec<u32>,
    sym_move: Vec<u8>,
}

pub(super) struct P1 {
    m: Moves1,
    prun_t: Prun,
    prun_x: Prun,
    prun_c: Prun,
    skips: Vec<u64>,
    next_axis: u64,
}

fn set_t(c: &mut Center, i: usize) {
    c.set_t(i);
}
fn get_t(c: &Center) -> usize {
    c.get_t()
}
fn set_x(c: &mut Center, i: usize) {
    c.set_x(i);
}
fn get_x(c: &Center) -> usize {
    c.get_x()
}

impl P1 {
    pub(super) fn build(syms: &Syms, b: &Build) -> P1 {
        let t = classes(T_CLASSES, set_t, get_t);
        let x = classes(X_CLASSES, set_x, get_x);
        let t_move = move_table(&t, set_t, get_t);
        let x_move = move_table(&x, set_x, get_x);
        let m = Moves1 {
            t_move,
            t_raw2sym: t.raw2sym,
            t_sym2raw: t.sym2raw,
            x_move,
            x_raw2sym: x.raw2sym,
            x_self_sym: x.self_sym,
            sym_move: syms.sym_moves(&MOVES, 16),
        };
        let prun_t = b.dense("p1_t_sym", &Sym { table: &m.t_move });
        let prun_x = b.dense("p1_x_sym", &Sym { table: &m.x_move });
        let joint = Joint { p: &m, syms };
        let prun_c = b.hashed("p1_center_sym", &joint, HASH_DEPTH, HASH_BITS);
        P1 {
            m,
            prun_t: Prun::new(prun_t, 1, usize::MAX),
            prun_x: Prun::new(prun_x, 1, usize::MAX),
            prun_c: Prun::new(prun_c, RAW, (1 << HASH_BITS) - 1),
            skips: mv::skip_masks(&MOVES),
            next_axis: mv::next_axis(&MOVES),
        }
    }

    /// The start node for a cube: the two colors `1` and `4` (R and L) of
    /// each center kind are the ones to bring onto two faces.
    pub(super) fn start(&self, cube: &super::cubie::Cubie) -> Node1 {
        let mut c = Center::new();
        for i in 0..24 {
            c.x[i] = if cube.xcenter[i] == 1 || cube.xcenter[i] == 4 {
                0
            } else {
                -1
            };
            c.t[i] = if cube.tcenter[i] == 1 || cube.tcenter[i] == 4 {
                0
            } else {
                -1
            };
        }
        Node1 {
            t: self.m.t_raw2sym[c.get_t()],
            x: self.m.x_raw2sym[c.get_x()],
        }
    }

    pub(super) fn bytes(&self) -> usize {
        self.prun_t.nibbles().bytes()
            + self.prun_x.nibbles().bytes()
            + self.prun_c.nibbles().bytes()
    }
}

/// A search node: the t and x state, each `class << 4 | symmetry`.
#[derive(Clone, Copy)]
pub(super) struct Node1 {
    t: u32,
    x: u32,
}

pub(super) struct Stage1<'a> {
    pub p: &'a P1,
    pub syms: &'a Syms,
}

impl Stage for Stage1<'_> {
    type Node = Node1;

    fn move_count(&self) -> usize {
        36
    }
    fn move_code(&self, m: usize) -> u8 {
        MOVES[m]
    }
    fn skips(&self) -> &[u64] {
        &self.p.skips
    }
    fn next_axis(&self) -> u64 {
        self.p.next_axis
    }
    fn min_back_depth(&self) -> i32 {
        5
    }

    fn prun(&self, n: &Node1) -> i32 {
        let (t, x) = (n.t as usize, n.x as usize);
        let p = self.p;
        let joint = t & !0xf | usize::from(self.syms.mult_inv[t & 0xf][x & 0xf]);
        p.prun_t
            .get1(t >> 4)
            .max(p.prun_x.get1(x >> 4))
            .max(p.prun_c.get2(x >> 4, p.m.t_sym2raw[joint] as usize))
    }

    fn step(&self, n: &Node1, mv: usize, _: i32) -> (Node1, i32) {
        let m = &self.p.m;
        let advance = |table: &[u32], state: u32| {
            let sym = state as usize & 0xf;
            let next =
                table[(state as usize >> 4) * 36 + usize::from(m.sym_move[sym * 36 + mv])] as usize;
            (next & !0xf | usize::from(self.syms.mult[next & 0xf][sym])) as u32
        };
        let node = Node1 {
            t: advance(&m.t_move, n.t),
            x: advance(&m.x_move, n.x),
        };
        (node, self.prun(&node))
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_same, java_table};
    use super::*;
    use crate::reduction::cache::Cache;

    fn build(cap: u32) -> (Syms, P1) {
        let syms = Syms::new();
        let b = Build {
            threads: 4,
            cache: Cache::new(None, 1),
            cap,
        };
        let p = P1::build(&syms, &b);
        (syms, p)
    }

    #[test]
    fn classes_are_as_many_as_the_original_counts() {
        let (_, p) = build(2);
        assert_eq!(p.m.t_move.len(), T_CLASSES * 36);
        assert_eq!(p.m.x_move.len(), X_CLASSES * 36);
    }

    #[test]
    fn the_start_node_of_a_solved_cube_is_a_goal() {
        let (syms, p) = build(3);
        let node = p.start(&super::super::cubie::Cubie::solved());
        // colors 1 and 4 sit on R and L: the goal of the phase
        assert_eq!(Stage1 { p: &p, syms: &syms }.prun(&node), 0);
    }

    #[test]
    #[ignore = "needs CUBE555_TABLES, a directory with the Java solver's tables"]
    fn tables_equal_the_java_tables() {
        let (_, p) = build(15);
        for (name, built) in [
            ("Phase1TCenterSymPrun.jhdata", &p.prun_t),
            ("Phase1XCenterSymPrun.jhdata", &p.prun_x),
            ("Phase1CenterSymPrun.jpdata", &p.prun_c),
        ] {
            let Some(java) = java_table(name) else {
                panic!("set CUBE555_TABLES");
            };
            assert_same(name, built.nibbles().words(), &java);
        }
    }
}
