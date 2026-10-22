//! Phase 4: pair the high and low wings, finish the up and down centres.
//!
//! Ported from cs0x7f/cube555 src/Phase4Search.java, MIT, Copyright (c) 2023
//! Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! Coordinates: the middle, low and high wing of four slots (70 x 1680 x
//! 1680), the up/down centres (4900) and the right/left centres (216 of the
//! 2450 reachable). The middle and low wings together have 29,616 classes
//! under 4 symmetries (`class << 2 | symmetry`).

use super::build::Build;
use super::cubie::Cubie;
use super::mv;
use super::p4_model::{Centre, Edge, MOVES};
use super::spaces::{ClassTable, Pair};
use crate::reduction::bfs::Space;
use crate::reduction::ida::Stage;
use crate::reduction::nibbles::Prun;

const COUNT: usize = 16;
const M: usize = 70;
const HL: usize = 1680;
const ML: usize = M * HL;
const CLASSES: usize = 29_616;
const UD: usize = 4900;
const RL: usize = 216;
const UD_SOLVED: [usize; 6] = [0, 1895, 1967, 2905, 2977, 4876];

/// Middle/low wing classes by the raw high wings, which move with them.
struct ClassBy<'a> {
    ml_move: &'a [u32],
    ml_self: &'a [u32],
    h_move: &'a [u32],
    h_conj: &'a [u32],
    /// Middle edge of each class-and-symmetry state.
    ml_rep_m: &'a [u32],
}

impl Space for ClassBy<'_> {
    fn size(&self) -> usize {
        CLASSES * HL
    }

    fn solved(&self) -> Vec<usize> {
        vec![0]
    }

    fn moves<F: FnMut(usize, u32)>(&self, state: usize, f: &mut F) {
        let (class, raw) = (state / HL, state % HL);
        let m0 = self.ml_rep_m[class << 2] as usize;
        for m in 0..COUNT {
            let new = self.ml_move[class * COUNT + m] as usize;
            let m1 = self.ml_rep_m[new] as usize;
            let moved = self.h_move[(m0 * HL + raw) * COUNT + m] as usize;
            let moved = self.h_conj[(m1 * HL + moved) * 4 + (new & 3)] as usize;
            f((new >> 2) * HL + moved, m1 as u32);
        }
    }

    fn images<F: FnMut(usize)>(&self, target: usize, tag: u32, f: &mut F) {
        let (class, raw) = (target / HL, target % HL);
        let mut state = self.ml_self[class];
        let mut j = 1;
        loop {
            state >>= 1;
            if state == 0 {
                return;
            }
            if state & 1 == 1 {
                f(class * HL + self.h_conj[(tag as usize * HL + raw) * 4 + j] as usize);
            }
            j += 1;
        }
    }
}

pub(super) struct P4 {
    rl_to_half: Vec<i32>,
    ud_move: Vec<u32>,
    ud_conj: Vec<u32>,
    rl_move: Vec<u32>,
    rl_conj: Vec<u32>,
    m_move: Vec<u32>,
    h_move: Vec<u32>,
    l_move: Vec<u32>,
    h_conj: Vec<u32>,
    ml_mirror: Vec<u32>,
    ml_raw2sym: Vec<u32>,
    prun_edge: Prun,
    prun_center: Prun,
    prun_ud: Prun,
    prun_rl: Prun,
    skips: Vec<u64>,
    next_axis: u64,
}

struct Centres {
    rl_to_half: Vec<i32>,
    ud_move: Vec<u32>,
    ud_conj: Vec<u32>,
    rl_move: Vec<u32>,
    rl_conj: Vec<u32>,
}

fn centres() -> Centres {
    let mut c = Centre::new();
    let mut to_half = vec![-1i32; 2450];
    let mut from_half = [0usize; RL];
    let mut tail = 1;
    to_half[0] = 0;
    while tail < RL {
        for i in 0..to_half.len() {
            if to_half[i] == -1 {
                continue;
            }
            for m in 10..COUNT {
                c.set_rl(i);
                c.do_move(m);
                let idx = c.get_rl();
                if to_half[idx] == -1 {
                    from_half[tail] = idx;
                    to_half[idx] = tail as i32;
                    tail += 1;
                }
            }
        }
    }
    let mut ud_move = vec![0u32; UD * COUNT];
    let mut ud_conj = vec![0u32; UD * 8];
    for i in 0..UD {
        for m in 0..COUNT {
            c.set_ud(i);
            c.do_move(m);
            ud_move[i * COUNT + m] = c.get_ud() as u32;
        }
        c.set_ud(i);
        for sym in 0..8 {
            ud_conj[i * 8 + sym] = c.get_ud() as u32;
            c.do_conj(0);
            if sym & 1 == 1 {
                c.do_conj(1);
            }
            if sym & 3 == 3 {
                c.do_conj(2);
            }
        }
    }
    let mut rl_move = vec![0u32; RL * COUNT];
    let mut rl_conj = vec![0u32; RL * 8];
    for i in 0..RL {
        for m in 0..COUNT {
            c.set_rl(from_half[i]);
            c.do_move(m);
            rl_move[i * COUNT + m] = to_half[c.get_rl()] as u32;
        }
        c.set_rl(from_half[i]);
        for sym in 0..8 {
            rl_conj[i * 8 + sym] = to_half[c.get_rl()] as u32;
            c.do_conj(0);
            if sym & 1 == 1 {
                c.do_conj(1);
            }
            if sym & 3 == 3 {
                c.do_conj(2);
            }
        }
    }
    Centres {
        rl_to_half: to_half,
        ud_move,
        ud_conj,
        rl_move,
        rl_conj,
    }
}

impl P4 {
    pub(super) fn build(b: &Build) -> P4 {
        let ct = centres();
        // Wing move tables, one row per (middle, wing) pair.
        let mut e = Edge::new();
        let mut m_move = vec![0u32; M * COUNT];
        let mut h_move = vec![0u32; ML * COUNT];
        let mut l_move = vec![0u32; ML * COUNT];
        let mut h_conj = vec![0u32; ML * 4];
        for m_edge in 0..M {
            for i in 0..HL {
                let row = m_edge * HL + i;
                for m in 0..COUNT {
                    e.set_m(m_edge);
                    e.set_h(i);
                    e.set_l(i);
                    e.do_move(m);
                    m_move[m_edge * COUNT + m] = e.get_m() as u32;
                    h_move[row * COUNT + m] = e.get_h() as u32;
                    l_move[row * COUNT + m] = e.get_l() as u32;
                }
                e.set_m(m_edge);
                e.set_h(i);
                for sym in 0..4 {
                    h_conj[row * 4 + sym] = e.get_h() as u32;
                    e.do_conj(0);
                    if sym & 1 == 1 {
                        e.do_conj(1);
                    }
                }
            }
        }
        // Middle/low classes.
        let mut ml_mirror = vec![0u32; ML];
        for (i, slot) in ml_mirror.iter_mut().enumerate() {
            e.set_m(i % M);
            e.set_l(i / M);
            e.do_conj(2);
            *slot = (e.get_h() * M + e.get_m()) as u32;
        }
        let mut raw2sym = vec![0u32; ML];
        let mut sym2raw = vec![0u32; CLASSES * 4];
        let mut ml_self = Vec::with_capacity(CLASSES);
        for i in 0..ML {
            if raw2sym[i] != 0 {
                continue;
            }
            let class = ml_self.len();
            ml_self.push(0u32);
            e.set_m(i % M);
            e.set_l(i / M);
            for sym in 0..4 {
                let idx = e.get_l() * M + e.get_m();
                raw2sym[idx] = (class << 2 | sym) as u32;
                sym2raw[class << 2 | sym] = idx as u32;
                if idx == i {
                    ml_self[class] |= 1 << sym;
                }
                e.do_conj(0);
                if sym & 1 == 1 {
                    e.do_conj(1);
                }
            }
        }
        assert_eq!(ml_self.len(), CLASSES, "class count");
        let mut ml_move = Vec::with_capacity(CLASSES * COUNT);
        for class in 0..CLASSES {
            for m in 0..COUNT {
                let rep = sym2raw[class << 2] as usize;
                e.set_m(rep % M);
                e.set_l(rep / M);
                e.do_move(m);
                ml_move.push(raw2sym[e.get_l() * M + e.get_m()]);
            }
        }
        let ml_rep_m: Vec<u32> = sym2raw.iter().map(|r| r % M as u32).collect();

        let rl_solved: Vec<usize> = UD_SOLVED[..3]
            .iter()
            .map(|&u| ct.rl_to_half[u] as usize)
            .collect();
        let centre_pair = Pair {
            a_move: &ct.rl_move,
            b_move: &ct.ud_move,
            moves: COUNT,
            n1: RL,
            n2: UD,
            a_solved: &rl_solved,
            b_solved: &UD_SOLVED,
        };
        let prun_center = b.dense("p4_center", &centre_pair);
        let edge_space = ClassBy {
            ml_move: &ml_move,
            ml_self: &ml_self,
            h_move: &h_move,
            h_conj: &h_conj,
            ml_rep_m: &ml_rep_m,
        };
        let prun_edge = b.dense("p4_edge_sym", &edge_space);
        let ud_space = ClassTable {
            class_move: &ml_move,
            self_sym: &ml_self,
            moves: COUNT,
            bits: 2,
            raw_count: UD,
            raw_move: &ct.ud_move,
            raw_conj: &ct.ud_conj,
            solved: UD_SOLVED.to_vec(),
        };
        let prun_ud = b.dense("p4_mledge_sym_udcenter", &ud_space);
        let rl_space = ClassTable {
            class_move: &ml_move,
            self_sym: &ml_self,
            moves: COUNT,
            bits: 2,
            raw_count: RL,
            raw_move: &ct.rl_move,
            raw_conj: &ct.rl_conj,
            solved: rl_solved.clone(),
        };
        let prun_rl = b.dense("p4_mledge_sym_rlcenter", &rl_space);
        P4 {
            rl_to_half: ct.rl_to_half,
            ud_move: ct.ud_move,
            ud_conj: ct.ud_conj,
            rl_move: ct.rl_move,
            rl_conj: ct.rl_conj,
            m_move,
            h_move,
            l_move,
            h_conj,
            ml_mirror,
            ml_raw2sym: raw2sym,
            prun_edge: Prun::new(prun_edge, HL, usize::MAX),
            prun_center: Prun::new(prun_center, UD, usize::MAX),
            prun_ud: Prun::new(prun_ud, UD, usize::MAX),
            prun_rl: Prun::new(prun_rl, RL, usize::MAX),
            skips: mv::skip_masks(&MOVES),
            next_axis: mv::next_axis(&MOVES),
        }
    }

    pub(super) fn bytes(&self) -> usize {
        [
            &self.prun_edge,
            &self.prun_center,
            &self.prun_ud,
            &self.prun_rl,
        ]
        .iter()
        .map(|p| p.nibbles().bytes())
        .sum()
    }

    /// One node per way to choose the four wing slots of the layer.
    pub(super) fn starts(&self, cube: &Cubie) -> Vec<Node4> {
        let mut e = Edge::new();
        let mut c = Centre::new();
        for i in 0..8 {
            let slot = 16 + (i & 4) + (i + 1) % 4;
            c.udx[i] = if cube.xcenter[i] == 0 { 0 } else { -1 };
            c.udt[i] = if cube.tcenter[i] == 0 { 0 } else { -1 };
            c.rlx[i] = if cube.xcenter[slot] == 1 { 0 } else { -1 };
            c.rlt[i] = if cube.tcenter[slot] == 1 { 0 } else { -1 };
        }
        let rl = self.rl_to_half[c.get_rl()] as u32;
        let ud = c.get_ud() as u32;
        let mut mask_y = 0u32;
        for i in 0..4 {
            mask_y |= 1 << (cube.wedge[8 + i] % 12);
            mask_y |= 1 << (cube.wedge[8 + i + 12] % 12);
            mask_y |= 1 << (cube.medge[8 + i] >> 1);
        }
        mask_y ^= 0xfff;
        let rank = |mask: u32, piece: i32| {
            if (mask >> piece) & 1 == 0 {
                -1
            } else {
                (mask & ((1 << piece) - 1)).count_ones() as i32
            }
        };
        let mut nodes = Vec::new();
        let mut mask = mask_y;
        while mask != 0 {
            if mask.count_ones() == 4 {
                for i in 0..8 {
                    e.e[0][i] = rank(mask, cube.medge[i] >> 1);
                    e.e[1][i] = rank(mask, cube.wedge[i] % 12);
                    e.e[2][i] = rank(mask, cube.wedge[i + 12] % 12);
                }
                e.standard = false;
                let m = e.get_m() as u32;
                let l = e.get_l() as u32;
                let h = e.get_h() as u32;
                nodes.push(Node4 { rl, ud, m, l, h });
            }
            mask = (mask - 1) & mask_y;
        }
        nodes
    }
}

#[derive(Clone, Copy)]
pub(super) struct Node4 {
    rl: u32,
    ud: u32,
    m: u32,
    l: u32,
    h: u32,
}

pub(super) struct Stage4<'a>(pub &'a P4);

impl Stage4<'_> {
    /// The pruning values of the edge-related tables, in the order of the
    /// original; calls `f` with each and stops at the first `true`.
    fn bounds(&self, n: &Node4, mut f: impl FnMut(i32) -> bool) -> bool {
        let p = self.0;
        let (m, rl, ud) = (n.m as usize, n.rl as usize, n.ud as usize);
        let ml = p.ml_raw2sym[n.l as usize * M + m] as usize;
        let mh = p.ml_raw2sym[p.ml_mirror[n.h as usize * M + m] as usize] as usize;
        f(p.prun_edge.get2(
            ml >> 2,
            p.h_conj[(m * HL + n.h as usize) * 4 + (ml & 3)] as usize,
        )) || f(p
            .prun_rl
            .get2(ml >> 2, p.rl_conj[rl * 8 + (ml & 3)] as usize))
            || f(p
                .prun_rl
                .get2(mh >> 2, p.rl_conj[rl * 8 + (mh & 3 | 4)] as usize))
            || f(p
                .prun_ud
                .get2(ml >> 2, p.ud_conj[ud * 8 + (ml & 3)] as usize))
            || f(p
                .prun_ud
                .get2(mh >> 2, p.ud_conj[ud * 8 + (mh & 3 | 4)] as usize))
    }
}

impl Stage for Stage4<'_> {
    type Node = Node4;

    fn move_count(&self) -> usize {
        COUNT
    }
    fn move_code(&self, m: usize) -> u8 {
        MOVES[m]
    }
    fn skips(&self) -> &[u64] {
        &self.0.skips
    }
    fn next_axis(&self) -> u64 {
        self.0.next_axis
    }
    fn min_back_depth(&self) -> i32 {
        1
    }

    fn prun(&self, n: &Node4) -> i32 {
        let mut best = self.0.prun_center.get2(n.rl as usize, n.ud as usize);
        self.bounds(n, |v| {
            best = best.max(v);
            false
        });
        best
    }

    /// Returns `maxl` if some table proves the node too far for `maxl` moves,
    /// else `maxl - 1`: the search only needs to know that.
    fn step(&self, n: &Node4, mv: usize, maxl: i32) -> (Node4, i32) {
        let p = self.0;
        let row = n.m as usize * HL;
        let node = Node4 {
            rl: p.rl_move[n.rl as usize * COUNT + mv],
            ud: p.ud_move[n.ud as usize * COUNT + mv],
            m: p.m_move[n.m as usize * COUNT + mv],
            l: p.l_move[(row + n.l as usize) * COUNT + mv],
            h: p.h_move[(row + n.h as usize) * COUNT + mv],
        };
        let far = self.bounds(&node, |v| maxl <= v)
            || maxl <= self.0.prun_center.get2(node.rl as usize, node.ud as usize);
        (node, if far { maxl } else { maxl - 1 })
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_same, java_table};
    use super::*;
    use crate::reduction::cache::Cache;

    fn build(cap: u32) -> P4 {
        P4::build(&Build {
            threads: 4,
            cache: Cache::new(None, 1),
            cap,
        })
    }

    #[test]
    fn centre_numbering_has_216_right_left_states() {
        let p = build(1);
        assert_eq!(p.rl_to_half.iter().filter(|&&h| h >= 0).count(), RL);
    }

    #[test]
    #[ignore = "needs CUBE555_TABLES, a directory with the Java solver's tables"]
    fn tables_equal_the_java_tables() {
        let p = build(15);
        for (name, built) in [
            ("Phase4CenterPrun.jhdata", &p.prun_center),
            ("Phase4EdgeSymPrun.jhdata", &p.prun_edge),
            ("Phase4MLEdgeSymUDCenterPrun.jhdata", &p.prun_ud),
            ("Phase4MLEdgeSymRLCenterPrun.jhdata", &p.prun_rl),
        ] {
            let java = java_table(name).expect("set CUBE555_TABLES");
            assert_same(name, built.nibbles().words(), &java);
        }
    }
}
