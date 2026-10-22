//! Phase 3: the wings into four slots per layer, the middle edges in place,
//! the right and left centres in their halves.
//!
//! Ported from cs0x7f/cube555 src/Phase3Center.java, src/Phase3Edge.java and
//! src/Phase3Search.java, MIT, Copyright (c) 2023 Chen Shuang; this file is
//! AGPL-3.0-or-later, see NOTICE.md.
//!
//! The 12 wings of one layer among 24 slots give 2,704,156 states, 86,048
//! classes under 32 symmetries (`class << 5 | symmetry`); the 12 middle
//! edges give 2048 flip states; the 8 right and left centres of each kind
//! give 35 states, 1225 together.

use super::build::Build;
use super::cubie::{move_edges, Cubie};
use super::cubie_sym::Syms;
use super::mv::{self, *};
use super::spaces::Pair;
use crate::reduction::bfs::Space;
use crate::reduction::comb::{get_comb, get_scomb, set_comb, swap2, swap4};
use crate::reduction::ida::Stage;
use crate::reduction::nibbles::Prun;

pub(super) const MOVES: [u8; 24] = [
    U1, U2, U3, R1, R2, R3, F1, F2, F3, D1, D2, D3, L1, L2, L3, B1, B2, B3, UW2, RW2, FW2, DW2,
    LW2, BW2,
];
const COUNT: usize = 24;
const WINGS: usize = 2_704_156;
const CLASSES: usize = 86_048;
const MEDGES: usize = 2048;
const CENTRES: usize = 1225;

const SOLVED_X: [usize; 6] = [0, 9, 14, 23, 27, 28];
const SOLVED_T: [usize; 18] = [
    0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 18, 23, 25, 27, 28, 30, 32, 34,
];

/// The eight right and left centres of each kind.
struct Centre {
    t: [i32; 8],
    x: [i32; 8],
}

impl Centre {
    fn set(&mut self, idx: usize) {
        set_comb(&mut self.x, (idx / 35) as i32, 4, 8);
        set_comb(&mut self.t, (idx % 35) as i32, 4, 8);
    }

    fn get(&self) -> usize {
        get_scomb(&self.x) as usize * 35 + get_scomb(&self.t) as usize
    }

    fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let pow = i32::from(code % 3);
        let (t, x) = (&mut self.t, &mut self.x);
        match code {
            UW2 => {
                swap2(t, 3, 7);
                swap2(x, 3, 7);
                swap2(x, 0, 4);
            }
            RW2 | R1 | R2 | R3 => {
                swap4(t, [0, 1, 2, 3], pow, false);
                swap4(x, [0, 1, 2, 3], pow, false);
            }
            FW2 => {
                swap2(t, 2, 4);
                swap2(x, 2, 4);
                swap2(x, 3, 5);
            }
            DW2 => {
                swap2(t, 1, 5);
                swap2(x, 1, 5);
                swap2(x, 2, 6);
            }
            LW2 | L1 | L2 | L3 => {
                swap4(t, [4, 5, 6, 7], pow, false);
                swap4(x, [4, 5, 6, 7], pow, false);
            }
            BW2 => {
                swap2(t, 0, 6);
                swap2(x, 0, 6);
                swap2(x, 1, 7);
            }
            _ => {}
        }
    }
}

/// Middle-edge flips and the wings of one layer (marked `0`, others `-1`).
struct Edge {
    m: [i32; 12],
    w: [i32; 24],
}

impl Edge {
    fn new() -> Edge {
        let mut w = [-1; 24];
        w[..12].fill(0);
        Edge { m: [0; 12], w }
    }

    fn set_medge(&mut self, mut idx: usize) {
        let mut parity = 0;
        for i in 0..11 {
            self.m[i] = (idx & 1) as i32;
            idx >>= 1;
            parity ^= self.m[i];
        }
        self.m[11] = parity;
    }

    fn get_medge(&self) -> usize {
        (0..11).fold(0, |idx, i| idx | (self.m[i] as usize) << i)
    }

    fn set_wedge(&mut self, idx: usize) {
        set_comb(&mut self.w, idx as i32, 12, 24);
    }

    fn get_wedge(&self) -> usize {
        get_comb(&self.w, 12, 24) as usize
    }

    fn do_move(&mut self, m: usize) {
        move_edges(&mut self.m, &mut self.w, MOVES[m]);
    }

    fn complement_wings(&mut self) {
        for w in &mut self.w {
            *w = -1 - *w;
        }
    }

    fn do_conj(&mut self, conj: u8) {
        let (m, w) = (&mut self.m, &mut self.w);
        match conj {
            0 => {
                swap4(w, [1, 14, 3, 12], 0, false);
                swap4(w, [5, 22, 6, 23], 0, false);
                swap4(w, [17, 10, 18, 11], 0, false);
                swap4(w, [0, 15, 2, 13], 2, false);
                swap4(w, [4, 20, 7, 21], 2, false);
                swap4(w, [16, 8, 19, 9], 2, false);
                swap4(m, [5, 10, 6, 11], 0, true);
                swap4(m, [4, 8, 7, 9], 2, true);
                swap4(m, [0, 1, 2, 3], 0, true);
                for i in 0..12 {
                    m[i] ^= 1;
                    w[i] = -1 - w[i];
                    w[i + 12] = -1 - w[i + 12];
                }
            }
            1 => {
                for q in [
                    [9, 22, 11, 20],
                    [0, 4, 1, 5],
                    [12, 16, 13, 17],
                    [8, 23, 10, 21],
                    [2, 7, 3, 6],
                    [14, 19, 15, 18],
                ] {
                    swap4(w, q, 1, false);
                }
                swap4(m, [0, 4, 1, 5], 1, false);
                swap4(m, [2, 7, 3, 6], 1, false);
                swap4(m, [8, 9, 10, 11], 1, true);
            }
            2 => {
                for (a, b) in [
                    (0, 12),
                    (1, 13),
                    (2, 14),
                    (3, 15),
                    (4, 17),
                    (5, 16),
                    (6, 19),
                    (7, 18),
                    (8, 23),
                    (9, 22),
                    (10, 21),
                    (11, 20),
                ] {
                    swap2(w, a, b);
                }
                for (a, b) in [(4, 5), (6, 7), (8, 11), (9, 10)] {
                    swap2(m, a, b);
                }
                self.complement_wings();
            }
            _ => {
                self.complement_wings();
                for e in &mut self.m {
                    *e ^= 1;
                }
            }
        }
    }

    /// Steps through the 32 symmetries in order.
    fn next_sym(&mut self, sym: usize) {
        self.do_conj(0);
        if sym & 3 == 3 {
            self.do_conj(1);
        }
        if sym & 7 == 7 {
            self.do_conj(2);
        }
        if sym & 0xf == 0xf {
            self.do_conj(3);
        }
    }
}

/// Space of the wing classes by middle-edge flips.
struct WingEdge<'a> {
    w_move: &'a [u32],
    m_move: &'a [u32],
    m_conj: &'a [u32],
    self_sym: &'a [u32],
}

impl Space for WingEdge<'_> {
    fn size(&self) -> usize {
        CLASSES * MEDGES
    }

    fn solved(&self) -> Vec<usize> {
        vec![0]
    }

    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        let (class, raw) = (state / MEDGES, state % MEDGES);
        for m in 0..COUNT {
            let val = self.w_move[class * COUNT + m] as usize;
            let flip = if val & 0x10 == 0 { 0 } else { 0x7ff };
            let raw = (self.m_move[raw * COUNT + m] as usize) ^ flip;
            let raw = self.m_conj[raw * 32 + (val & 0xf)] as usize;
            f((val >> 5) * MEDGES + raw, 0);
        }
    }

    fn images<F: FnMut(usize)>(&self, target: usize, _tag: u32, f: &mut F) {
        let (class, raw) = (target / MEDGES, target % MEDGES);
        let mut state = self.self_sym[class];
        let mut j = 1;
        loop {
            state >>= 1;
            if state == 0 {
                return;
            }
            if state & 1 == 1 {
                f(class * MEDGES + self.m_conj[raw * 32 + j] as usize);
            }
            j += 1;
        }
    }
}

pub(super) struct P3 {
    center_move: Vec<u32>,
    m_move: Vec<u32>,
    m_conj: Vec<u32>,
    w_move: Vec<u32>,
    w_raw2sym: Vec<u32>,
    sym_move: Vec<u8>,
    prun_cm: Prun,
    prun_wm: Prun,
    skips: Vec<u64>,
    next_axis: u64,
}

impl P3 {
    pub(super) fn build(syms: &Syms, b: &Build) -> P3 {
        // Wing classes.
        let mut e = Edge::new();
        let mut raw2sym = vec![0u32; WINGS];
        let mut rep = Vec::with_capacity(CLASSES);
        let mut self_sym = Vec::with_capacity(CLASSES);
        for i in 0..WINGS {
            if raw2sym[i] != 0 {
                continue;
            }
            let class = rep.len();
            self_sym.push(0u32);
            e.set_wedge(i);
            for sym in 0..32 {
                let idx = e.get_wedge();
                raw2sym[idx] = (class << 5 | sym) as u32;
                if idx == i {
                    self_sym[class] |= 1 << sym;
                }
                e.next_sym(sym);
            }
            rep.push(i as u32);
        }
        assert_eq!(rep.len(), CLASSES, "class count");
        let mut w_move = Vec::with_capacity(CLASSES * COUNT);
        for &r in &rep {
            for m in 0..COUNT {
                e.set_wedge(r as usize);
                e.do_move(m);
                w_move.push(raw2sym[e.get_wedge()]);
            }
        }
        // Middle edges.
        let mut m_move = vec![0u32; MEDGES * COUNT];
        let mut m_conj = vec![0u32; MEDGES * 32];
        for i in 0..MEDGES {
            for m in 0..COUNT {
                e.set_medge(i);
                e.do_move(m);
                m_move[i * COUNT + m] = e.get_medge() as u32;
            }
            e.set_medge(i);
            for sym in 0..32 {
                let slot = usize::from(syms.mult_inv[0][sym & 0xf]) | sym & 0x10;
                m_conj[i * 32 + slot] = e.get_medge() as u32;
                e.next_sym(sym);
            }
        }
        // Centres.
        let mut c = Centre {
            t: [0; 8],
            x: [0; 8],
        };
        let mut center_move = vec![0u32; CENTRES * COUNT];
        for i in 0..CENTRES {
            for m in 0..COUNT {
                c.set(i);
                c.do_move(m);
                center_move[i * COUNT + m] = c.get() as u32;
            }
        }
        let solved_c: Vec<usize> = SOLVED_X
            .iter()
            .flat_map(|x| SOLVED_T.iter().map(move |t| x * 35 + t))
            .collect();
        let cm = Pair {
            a_move: &center_move,
            b_move: &m_move,
            moves: COUNT,
            n1: CENTRES,
            n2: MEDGES,
            a_solved: &solved_c,
            b_solved: &[0, 2047],
        };
        let prun_cm = b.dense("p3_center_medge", &cm);
        let wm = WingEdge {
            w_move: &w_move,
            m_move: &m_move,
            m_conj: &m_conj,
            self_sym: &self_sym,
        };
        let prun_wm = b.dense("p3_wing_medge_sym", &wm);
        P3 {
            center_move,
            m_move,
            m_conj,
            w_move,
            w_raw2sym: raw2sym,
            sym_move: syms.sym_moves(&MOVES, 16),
            prun_cm: Prun::new(prun_cm, MEDGES, usize::MAX),
            prun_wm: Prun::new(prun_wm, MEDGES, usize::MAX),
            skips: mv::skip_masks(&MOVES),
            next_axis: mv::next_axis(&MOVES),
        }
    }

    pub(super) fn bytes(&self) -> usize {
        self.prun_cm.nibbles().bytes() + self.prun_wm.nibbles().bytes()
    }

    /// The start nodes for a cube: one per choice of which of the two
    /// layers' wings go to which half, as many as the pruning allows.
    pub(super) fn starts(&self, cube: &Cubie, syms: &Syms) -> Vec<Node3> {
        let mut ct = Centre {
            t: [0; 8],
            x: [0; 8],
        };
        for i in 0..8 {
            let slot = 16 + (i & 4) + (i + 1) % 4;
            ct.x[i] = if cube.xcenter[slot] == 1 { 0 } else { -1 };
            ct.t[i] = if cube.tcenter[slot] == 1 { 0 } else { -1 };
        }
        let center = ct.get() as u32;
        let stage = Stage3 { p: self, syms };
        let mut ed = Edge::new();
        let mut nodes = Vec::new();
        let mut filter = 8;
        while nodes.is_empty() {
            for idx in 0..1024u32 {
                let flip = idx << 1 | (idx.count_ones() & 1);
                let flip = (flip ^ 0xfff) << 12 | flip;
                for i in 0..12 {
                    ed.m[i] = cube.medge[i] & 1;
                    ed.m[i] ^= ((flip >> (cube.medge[i] >> 1)) & 1) as i32;
                }
                for i in 0..24 {
                    ed.w[i] = if (flip >> cube.wedge[i]) & 1 == 0 {
                        0
                    } else {
                        -1
                    };
                }
                let node = Node3 {
                    center,
                    medge: ed.get_medge() as u32,
                    wedge: self.w_raw2sym[ed.get_wedge()],
                };
                if stage.prun(&node) > filter {
                    continue;
                }
                nodes.push(node);
            }
            filter += 1;
        }
        nodes
    }
}

#[derive(Clone, Copy)]
pub(super) struct Node3 {
    pub center: u32,
    pub medge: u32,
    pub wedge: u32,
}

pub(super) struct Stage3<'a> {
    pub p: &'a P3,
    pub syms: &'a Syms,
}

impl Stage for Stage3<'_> {
    type Node = Node3;

    fn move_count(&self) -> usize {
        COUNT
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
        1
    }

    fn prun(&self, n: &Node3) -> i32 {
        let p = self.p;
        let conj = p.m_conj[n.medge as usize * 32 + (n.wedge as usize & 0x1f)] as usize;
        p.prun_cm
            .get2(n.center as usize, n.medge as usize)
            .max(p.prun_wm.get2(n.wedge as usize >> 5, conj))
    }

    fn step(&self, n: &Node3, m: usize, _: i32) -> (Node3, i32) {
        let p = self.p;
        let w = n.wedge as usize;
        let sym = w & 0xf;
        let image = usize::from(p.sym_move[sym * COUNT + m]);
        let moved = (p.w_move[(w >> 5) * COUNT + image] as usize) ^ (w & 0x10);
        let wedge = moved & !0xf | usize::from(self.syms.mult[moved & 0xf][sym]);
        let node = Node3 {
            center: p.center_move[n.center as usize * COUNT + m],
            medge: p.m_move[n.medge as usize * COUNT + m],
            wedge: wedge as u32,
        };
        (node, self.prun(&node))
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_same, java_table};
    use super::*;
    use crate::reduction::cache::Cache;

    fn build(cap: u32) -> (Syms, P3) {
        let syms = Syms::new();
        let p = P3::build(
            &syms,
            &Build {
                threads: 4,
                cache: Cache::new(None, 1),
                cap,
            },
        );
        (syms, p)
    }

    #[test]
    fn a_state_with_everything_in_place_has_a_start_node_at_zero() {
        let (syms, p) = build(2);
        let nodes = p.starts(&Cubie::solved(), &syms);
        let stage = Stage3 { p: &p, syms: &syms };
        assert!(nodes.iter().any(|n| stage.prun(n) == 0));
    }

    #[test]
    #[ignore = "needs CUBE555_TABLES, a directory with the Java solver's tables"]
    fn tables_equal_the_java_tables() {
        let (_, p) = build(15);
        for (name, built) in [
            ("Phase3CenterMEdgePrun.jhdata", &p.prun_cm),
            ("Phase3MWEdgeSymPrun.jhdata", &p.prun_wm),
        ] {
            let java = java_table(name).expect("set CUBE555_TABLES");
            assert_same(name, built.nibbles().words(), &java);
        }
    }
}
