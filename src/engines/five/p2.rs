//! Phase 2: the remaining centres of the two colours onto U and D, with the
//! parity of the wings made even.
//!
//! Ported from cs0x7f/cube555 src/Phase2Center.java and src/Phase2Search.java,
//! MIT, Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md.

use super::build::Build;
use super::cubie::Cubie;
use super::mv::{self, *};
use super::spaces::Pair;
use crate::reduction::comb::{get_comb, parity, set_comb, swap2, swap4};
use crate::reduction::ida::Stage;
use crate::reduction::nibbles::Prun;

pub(super) const MOVES: [u8; 22] = [
    U1, U2, U3, F1, F2, F3, D1, D2, D3, B1, B2, B3, UW2, RW1, RW2, RW3, FW2, DW2, LW1, LW2, LW3,
    BW2,
];
const COUNT: usize = 22;
const STATES: usize = 12_870;

/// Which moves change the parity of the wings.
const PARITY_DIFF: [u32; COUNT] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0,
];

/// The centres on the 16 slots of U, D, F and B, as slot arrays.
struct Centre {
    t: [i32; 16],
    x: [i32; 16],
}

impl Centre {
    fn do_move(&mut self, m: usize) {
        let code = MOVES[m];
        let (axis, pow) = (code / 3, i32::from(code % 3));
        let (t, x) = (&mut self.t, &mut self.x);
        match axis {
            6 => {
                swap2(x, 8, 12);
                swap2(x, 9, 13);
                swap2(t, 8, 12);
                swap4(x, [0, 1, 2, 3], pow, false);
                swap4(t, [0, 1, 2, 3], pow, false);
            }
            0 => {
                swap4(x, [0, 1, 2, 3], pow, false);
                swap4(t, [0, 1, 2, 3], pow, false);
            }
            7 => {
                swap4(x, [1, 15, 5, 9], pow, false);
                swap4(x, [2, 12, 6, 10], pow, false);
                swap4(t, [1, 15, 5, 9], pow, false);
            }
            8 => {
                swap2(x, 2, 4);
                swap2(x, 3, 5);
                swap2(t, 2, 4);
                swap4(x, [8, 9, 10, 11], pow, false);
                swap4(t, [8, 9, 10, 11], pow, false);
            }
            2 => {
                swap4(x, [8, 9, 10, 11], pow, false);
                swap4(t, [8, 9, 10, 11], pow, false);
            }
            9 => {
                swap2(x, 10, 14);
                swap2(x, 11, 15);
                swap2(t, 10, 14);
                swap4(x, [4, 5, 6, 7], pow, false);
                swap4(t, [4, 5, 6, 7], pow, false);
            }
            3 => {
                swap4(x, [4, 5, 6, 7], pow, false);
                swap4(t, [4, 5, 6, 7], pow, false);
            }
            10 => {
                swap4(x, [0, 8, 4, 14], pow, false);
                swap4(x, [3, 11, 7, 13], pow, false);
                swap4(t, [3, 11, 7, 13], pow, false);
            }
            11 => {
                swap2(x, 1, 7);
                swap2(x, 0, 6);
                swap2(t, 0, 6);
                swap4(x, [12, 13, 14, 15], pow, false);
                swap4(t, [12, 13, 14, 15], pow, false);
            }
            5 => {
                swap4(x, [12, 13, 14, 15], pow, false);
                swap4(t, [12, 13, 14, 15], pow, false);
            }
            _ => {}
        }
    }
}

pub(super) struct P2 {
    t_move: Vec<u32>,
    x_move: Vec<u32>,
    prun_t: Prun,
    prun_x: Prun,
    skips: Vec<u64>,
    next_axis: u64,
}

impl P2 {
    pub(super) fn build(b: &Build) -> P2 {
        let mut c = Centre {
            t: [0; 16],
            x: [0; 16],
        };
        let mut t_move = vec![0u32; STATES * COUNT];
        let mut x_move = vec![0u32; STATES * COUNT];
        for i in 0..STATES {
            for m in 0..COUNT {
                set_comb(&mut c.t, i as i32, 8, 16);
                set_comb(&mut c.x, i as i32, 8, 16);
                c.do_move(m);
                t_move[i * COUNT + m] = get_comb(&c.t, 8, 16) as u32;
                x_move[i * COUNT + m] = get_comb(&c.x, 8, 16) as u32;
            }
        }
        let mut parity_move = vec![0u32; 2 * COUNT];
        for m in 0..COUNT {
            parity_move[m] = PARITY_DIFF[m];
            parity_move[COUNT + m] = 1 ^ PARITY_DIFF[m];
        }
        let pair = |name: &str, moves: &[u32]| {
            let space = Pair {
                a_move: moves,
                b_move: &parity_move,
                moves: COUNT,
                n1: STATES,
                n2: 2,
                a_solved: &[0],
                b_solved: &[0],
            };
            Prun::new(b.dense(name, &space), 2, usize::MAX)
        };
        P2 {
            prun_t: pair("p2_t", &t_move),
            prun_x: pair("p2_x", &x_move),
            t_move,
            x_move,
            skips: mv::skip_masks(&MOVES),
            next_axis: mv::next_axis(&MOVES),
        }
    }

    pub(super) fn start(&self, cube: &Cubie) -> Node2 {
        let mut c = Centre {
            t: [0; 16],
            x: [0; 16],
        };
        for i in 0..16 {
            c.x[i] = if matches!(cube.xcenter[i], 0 | 3) {
                0
            } else {
                -1
            };
            c.t[i] = if matches!(cube.tcenter[i], 0 | 3) {
                0
            } else {
                -1
            };
        }
        Node2 {
            t: get_comb(&c.t, 8, 16) as u32,
            x: get_comb(&c.x, 8, 16) as u32,
            parity: parity(&cube.wedge) as u32,
        }
    }

    pub(super) fn bytes(&self) -> usize {
        self.prun_t.nibbles().bytes() + self.prun_x.nibbles().bytes()
    }
}

#[derive(Clone, Copy)]
pub(super) struct Node2 {
    t: u32,
    x: u32,
    parity: u32,
}

pub(super) struct Stage2<'a>(pub &'a P2);

impl Stage for Stage2<'_> {
    type Node = Node2;

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
        5
    }

    fn prun(&self, n: &Node2) -> i32 {
        let p = self.0;
        p.prun_t
            .get2(n.t as usize, n.parity as usize)
            .max(p.prun_x.get2(n.x as usize, n.parity as usize))
    }

    fn step(&self, n: &Node2, m: usize, _: i32) -> (Node2, i32) {
        let p = self.0;
        let node = Node2 {
            t: p.t_move[n.t as usize * COUNT + m],
            x: p.x_move[n.x as usize * COUNT + m],
            parity: n.parity ^ PARITY_DIFF[m],
        };
        (node, self.prun(&node))
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_same, java_table};
    use super::*;
    use crate::reduction::cache::Cache;

    fn build(cap: u32) -> P2 {
        P2::build(&Build {
            threads: 4,
            cache: Cache::new(None, 1),
            cap,
        })
    }

    #[test]
    fn a_solved_cube_is_a_goal() {
        let p = build(15);
        let node = p.start(&Cubie::solved());
        assert_eq!(Stage2(&p).prun(&node), 0);
    }

    #[test]
    fn the_wing_parity_of_a_turn_is_known() {
        // A quarter turn of an inner slice swaps wings in an odd way; the table of
        // parity changes agrees with the wings of the cube model.
        for (m, &code) in MOVES.iter().enumerate() {
            let mut cube = Cubie::solved();
            cube.do_move(code);
            assert_eq!(parity(&cube.wedge) as u32, PARITY_DIFF[m], "move {code}");
        }
    }

    #[test]
    #[ignore = "needs CUBE555_TABLES, a directory with the Java solver's tables"]
    fn tables_equal_the_java_tables() {
        let p = build(15);
        for (name, built) in [
            ("Phase2TCenterPrun.jhdata", &p.prun_t),
            ("Phase2XCenterPrun.jhdata", &p.prun_x),
        ] {
            let java = java_table(name).expect("set CUBE555_TABLES");
            assert_same(name, built.nibbles().words(), &java);
        }
    }
}
