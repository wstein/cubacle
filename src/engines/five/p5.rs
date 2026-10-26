//! Phase 5: complete the centers and the pairing, leaving a 3x3x3.
//!
//! Ported from cs0x7f/cube555 src/Phase5Search.java, MIT, Copyright (c) 2023
//! Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! Coordinates: the low wings (a permutation of eight, 40,320, in 5,288
//! classes under 8 symmetries), the centers (36 x 70 x 70) and the high wings
//! seen in a mirror, so that one table serves both.

use super::build::Build;
use super::cubie::Cubie;
use super::cubie_sym::Syms;
use super::mv;
use super::p5_model::{Center, Edge, MOVES};
use super::spaces::{ClassTable, Plain};
use crate::reduction::ida::Stage;
use crate::reduction::nibbles::Prun;

const COUNT: usize = 14;
const PERMS: usize = 40_320;
const CLASSES: usize = 5_288;
const UD: usize = 4900;
const CENTERS: usize = 70 * 70 * 36;

pub(super) struct P5 {
    l_move: Vec<u32>,
    l_raw2sym: Vec<u32>,
    l_mirror: Vec<u32>,
    center_move: Vec<u32>,
    ud_mirror: Vec<u32>,
    ud_conj: Vec<u32>,
    sym_move: Vec<u8>,
    prun_center: Prun,
    prun_edge: Prun,
    skips: Vec<u64>,
    next_axis: u64,
}

impl P5 {
    pub(super) fn build(syms: &Syms, b: &Build) -> P5 {
        // Low wings.
        let mut e = Edge::new();
        let mut l_mirror = vec![0u32; PERMS];
        for (i, slot) in l_mirror.iter_mut().enumerate() {
            e.set_l(i);
            e.do_conj(2);
            *slot = e.get_h() as u32;
        }
        let mut raw2sym = vec![0u32; PERMS];
        let mut rep = Vec::with_capacity(CLASSES);
        let mut self_sym = Vec::with_capacity(CLASSES);
        for i in 0..PERMS {
            if raw2sym[i] != 0 {
                continue;
            }
            let class = rep.len();
            self_sym.push(0u32);
            e.set_l(i);
            for sym in 0..8 {
                let idx = e.get_l();
                raw2sym[idx] = (class << 3 | sym) as u32;
                if idx == i {
                    self_sym[class] |= 1 << sym;
                }
                e.do_conj(0);
                if sym & 3 == 3 {
                    e.do_conj(1);
                }
            }
            rep.push(i);
        }
        assert_eq!(rep.len(), CLASSES, "class count");
        let mut class_move = Vec::with_capacity(CLASSES * COUNT);
        for &r in &rep {
            for m in 0..COUNT {
                e.set_l(r);
                e.do_move(m);
                class_move.push(raw2sym[e.get_l()]);
            }
        }
        let mut l_move = vec![0u32; PERMS * COUNT];
        for i in 0..PERMS {
            for m in 0..COUNT {
                e.set_l(i);
                e.do_move(m);
                l_move[i * COUNT + m] = e.get_l() as u32;
            }
        }
        // Centers.
        let mut c = Center::new();
        let mut t_move = vec![0usize; 70 * COUNT];
        let mut x_move = vec![0usize; 70 * COUNT];
        let mut t_conj = vec![0usize; 70 * 8];
        let mut x_conj = vec![0usize; 70 * 8];
        let mut rflb_move = vec![0usize; 36 * COUNT];
        for i in 0..70 {
            for m in 0..COUNT {
                c.set_t(i);
                c.set_x(i);
                c.do_move(m);
                t_move[i * COUNT + m] = c.get_t();
                x_move[i * COUNT + m] = c.get_x();
            }
            c.set_t(i);
            c.set_x(i);
            for sym in 0..8 {
                let slot = usize::from(syms.mult_inv[0][sym]);
                t_conj[i * 8 + slot] = c.get_t();
                x_conj[i * 8 + slot] = c.get_x();
                c.do_conj(0);
                if sym & 3 == 3 {
                    c.do_conj(1);
                }
            }
        }
        for i in 0..36 {
            for m in 0..COUNT {
                c.set_rflb(i);
                c.do_move(m);
                rflb_move[i * COUNT + m] = c.get_rflb();
            }
        }
        let mut center_move = vec![0u32; CENTERS * COUNT];
        for i in 0..CENTERS {
            let (t, x, r) = (i % 70, i / 70 % 70, i / 70 / 70);
            for m in 0..COUNT {
                center_move[i * COUNT + m] =
                    ((rflb_move[r * COUNT + m] * 70 + x_move[x * COUNT + m]) * 70
                        + t_move[t * COUNT + m]) as u32;
            }
        }
        let mut ud_mirror = vec![0u32; UD];
        let mut ud_conj = vec![0u32; UD * 8];
        for i in 0..UD {
            let (t, x) = (i % 70, i / 70 % 70);
            c.set_t(t);
            c.set_x(x);
            c.do_conj(2);
            ud_mirror[i] = (c.get_x() * 70 + c.get_t()) as u32;
            for s in 0..8 {
                ud_conj[i * 8 + s] = (x_conj[x * 8 + s] * 70 + t_conj[t * 8 + s]) as u32;
            }
        }
        let ud_move: Vec<u32> = (0..UD * COUNT)
            .map(|k| center_move[k] % UD as u32)
            .collect();
        let prun_center = b.dense(
            "p5_center",
            &Plain {
                table: &center_move,
                moves: COUNT,
                solved: &[0],
            },
        );
        let edge_space = ClassTable {
            class_move: &class_move,
            self_sym: &self_sym,
            moves: COUNT,
            bits: 3,
            raw_count: UD,
            raw_move: &ud_move,
            raw_conj: &ud_conj,
            solved: vec![0],
        };
        let prun_edge = b.dense("p5_ledge_sym_center", &edge_space);
        P5 {
            l_move,
            l_raw2sym: raw2sym,
            l_mirror,
            center_move,
            ud_mirror,
            ud_conj,
            sym_move: syms.sym_moves(&MOVES, 16),
            prun_center: Prun::new(prun_center, 1, usize::MAX),
            prun_edge: Prun::new(prun_edge, UD, usize::MAX),
            skips: mv::skip_masks(&MOVES),
            next_axis: mv::next_axis(&MOVES),
        }
    }

    pub(super) fn bytes(&self) -> usize {
        self.prun_center.nibbles().bytes() + self.prun_edge.nibbles().bytes()
    }

    pub(super) fn start(&self, cube: &Cubie) -> Node5 {
        let mut e = Edge::new();
        let mut c = Center::new();
        let mut mask = 0u32;
        for i in 0..8 {
            mask |= 1 << (cube.medge[i] >> 1);
        }
        let rank = |piece: i32| (mask & ((1 << piece) - 1)).count_ones() as i32;
        for i in 0..8 {
            e.e[0][i] = rank(cube.medge[i] >> 1);
            e.e[1][i] = rank(cube.wedge[i] % 12);
            e.e[2][i] = rank(cube.wedge[i + 12] % 12);
            c.x[i] = if cube.xcenter[i] == 0 { 0 } else { -1 };
            c.t[i] = if cube.tcenter[i] == 0 { 0 } else { -1 };
            let side = cube.tcenter[9 + i * 2];
            c.rflb[i] = if side == 1 || side == 2 { 0 } else { -1 };
        }
        e.standard = false;
        Node5 {
            l: e.get_l() as u32,
            h: self.l_mirror[e.get_h()],
            center: ((c.get_rflb() * 70 + c.get_x()) * 70 + c.get_t()) as u32,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Node5 {
    l: u32,
    /// The high wings in the mirror, a permutation like the low wings.
    h: u32,
    center: u32,
}

pub(super) struct Stage5<'a>(pub &'a P5);

impl Stage for Stage5<'_> {
    type Node = Node5;

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

    fn prun(&self, n: &Node5) -> i32 {
        let p = self.0;
        let (ls, hs) = (
            p.l_raw2sym[n.l as usize] as usize,
            p.l_raw2sym[n.h as usize] as usize,
        );
        let ud = n.center as usize % UD;
        p.prun_center.get1(n.center as usize).max(
            p.prun_edge
                .get2(ls >> 3, p.ud_conj[ud * 8 + (ls & 7)] as usize)
                .max(p.prun_edge.get2(
                    hs >> 3,
                    p.ud_conj[p.ud_mirror[ud] as usize * 8 + (hs & 7)] as usize,
                )),
        )
    }

    fn step(&self, n: &Node5, m: usize, _: i32) -> (Node5, i32) {
        let p = self.0;
        let mirrored = usize::from(p.sym_move[8 * COUNT + m]);
        let node = Node5 {
            l: p.l_move[n.l as usize * COUNT + m],
            h: p.l_move[n.h as usize * COUNT + mirrored],
            center: p.center_move[n.center as usize * COUNT + m],
        };
        (node, self.prun(&node))
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_same, java_table};
    use super::*;
    use crate::reduction::cache::Cache;

    fn build(cap: u32) -> P5 {
        P5::build(
            &Syms::new(),
            &Build {
                threads: 4,
                cache: Cache::new(None, 1),
                cap,
            },
        )
    }

    #[test]
    fn a_solved_cube_is_a_goal() {
        let p = build(3);
        assert_eq!(Stage5(&p).prun(&p.start(&Cubie::solved())), 0);
    }

    #[test]
    #[ignore = "needs CUBE555_TABLES, a directory with the Java solver's tables"]
    fn tables_equal_the_java_tables() {
        let p = build(15);
        for (name, built) in [
            ("Phase5CenterPrun.jhdata", &p.prun_center),
            ("Phase5LEdgeSymCenterPrun.jhdata", &p.prun_edge),
        ] {
            let java = java_table(name).expect("set CUBE555_TABLES");
            assert_same(name, built.nibbles().words(), &java);
        }
    }
}
