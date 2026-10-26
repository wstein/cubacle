//! Phase 3, wings: the 24 wings of a 4x4x4 whose pairs are formed, seen as 12
//! pairs in 12 places, and the way they move and turn with the symmetries.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Edge3.java, MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! A state is an even permutation of 12 pairs (239,500,800 states, number
//! `0..12!/2`). The first four places (11,880 values) are reduced by the 8
//! symmetries that keep phase 3 closed to 1,538 classes; with the other six
//! places (20,160 values) a state is `class * 20160 + rest`.

use super::cube::Cube;

pub(super) const N_SYM: usize = 1538;
pub(super) const N_RAW: usize = 20160;
pub(super) const N_PRUN: usize = N_SYM * N_RAW;

/// `12! / 2` and its divisors, the radices of the numbers of even permutations.
const FACT_X: [i32; 13] = [
    1, 1, 1, 3, 12, 60, 360, 2520, 20160, 181440, 1814400, 19958400, 239500800,
];
/// The inverses of the 8 symmetries, as numbered by their generators.
const SYM_INV: [usize; 8] = [0, 1, 6, 3, 4, 5, 2, 7];
const ALL: u64 = 0xba98_7654_3210;
const ONES: u64 = 0x1111_1111_1110;

/// A placement of the 12 wing pairs, with the symmetries' relabelling kept
/// lazily in `edgeo`.
#[derive(Clone, Copy)]
pub(super) struct Edge {
    pub edge: [i32; 12],
    edgeo: [i32; 12],
    std: bool,
}

fn circle(arr: &mut [i32; 12], a: usize, b: usize, c: usize, d: usize) {
    let t = arr[d];
    arr[d] = arr[c];
    arr[c] = arr[b];
    arr[b] = arr[a];
    arr[a] = t;
}

fn swap4(arr: &mut [i32; 12], a: usize, b: usize, c: usize, d: usize) {
    arr.swap(a, c);
    arr.swap(b, d);
}

impl Edge {
    pub(super) fn new() -> Edge {
        Edge {
            edge: [0; 12],
            edgeo: [0; 12],
            std: true,
        }
    }

    /// The placement with number `idx`.
    pub(super) fn set(&mut self, mut idx: i32) {
        let mut val = ALL;
        let mut parity = 0;
        for i in 0..11 {
            let p = FACT_X[11 - i];
            let v = idx / p;
            idx %= p;
            parity ^= v;
            let shift = (v << 2) as u32;
            self.edge[i] = ((val >> shift) & 0xf) as i32;
            let m = (1u64 << shift) - 1;
            val = (val & m) + ((val >> 4) & !m);
        }
        if parity & 1 == 0 {
            self.edge[11] = val as i32;
        } else {
            self.edge[11] = self.edge[10];
            self.edge[10] = val as i32;
        }
        for i in 0..12 {
            self.edgeo[i] = i as i32;
        }
        self.std = true;
    }

    /// The wings of `cube` (after phase 2) as a placement; returns the parity
    /// the pairing leaves.
    pub(super) fn set_from(&mut self, cube: &Cube) -> u8 {
        const FULL_EDGE_MAP: [usize; 12] = [0, 2, 4, 6, 1, 3, 7, 5, 8, 9, 10, 11];
        let mut temp = [0i32; 12];
        for i in 0..12 {
            temp[i] = i as i32;
            self.edge[i] = i32::from(cube.ep[FULL_EDGE_MAP[i] + 12] % 12);
        }
        let mut parity = 1u8;
        for i in 0..12 {
            while self.edge[i] != i as i32 {
                let t = self.edge[i] as usize;
                self.edge[i] = self.edge[t];
                self.edge[t] = t as i32;
                temp.swap(i, t);
                parity ^= 1;
            }
        }
        for i in 0..12 {
            self.edge[i] = temp[usize::from(cube.ep[FULL_EDGE_MAP[i]] % 12)];
        }
        for i in 0..12 {
            self.edgeo[i] = i as i32;
        }
        self.std = true;
        parity
    }

    fn standardize(&mut self) {
        let mut temp = [0i32; 12];
        for i in 0..12 {
            temp[self.edgeo[i] as usize] = i as i32;
        }
        for i in 0..12 {
            self.edge[i] = temp[self.edge[i] as usize];
            self.edgeo[i] = i as i32;
        }
        self.std = true;
    }

    /// The number of the first `end` places (4 or 10).
    pub(super) fn get(&mut self, end: usize) -> i32 {
        if !self.std {
            self.standardize();
        }
        let mut idx = 0;
        let mut val = ALL;
        for i in 0..end {
            let v = (self.edge[i] << 2) as u32;
            idx *= 12 - i as i32;
            idx += ((val >> v) & 0xf) as i32;
            val = val.wrapping_sub(ONES << v);
        }
        idx
    }

    /// Makes phase-3 move `i` (0 to 19).
    pub(super) fn apply(&mut self, i: usize) {
        self.std = false;
        let (e, o) = (&mut self.edge, &mut self.edgeo);
        match i {
            0 => {
                circle(e, 0, 4, 1, 5);
                circle(o, 0, 4, 1, 5);
            }
            1 => {
                swap4(e, 0, 4, 1, 5);
                swap4(o, 0, 4, 1, 5);
            }
            2 => {
                circle(e, 0, 5, 1, 4);
                circle(o, 0, 5, 1, 4);
            }
            3 => {
                swap4(e, 5, 10, 6, 11);
                swap4(o, 5, 10, 6, 11);
            }
            4 => {
                circle(e, 0, 11, 3, 8);
                circle(o, 0, 11, 3, 8);
            }
            5 => {
                swap4(e, 0, 11, 3, 8);
                swap4(o, 0, 11, 3, 8);
            }
            6 => {
                circle(e, 0, 8, 3, 11);
                circle(o, 0, 8, 3, 11);
            }
            7 => {
                circle(e, 2, 7, 3, 6);
                circle(o, 2, 7, 3, 6);
            }
            8 => {
                swap4(e, 2, 7, 3, 6);
                swap4(o, 2, 7, 3, 6);
            }
            9 => {
                circle(e, 2, 6, 3, 7);
                circle(o, 2, 6, 3, 7);
            }
            10 => {
                swap4(e, 4, 8, 7, 9);
                swap4(o, 4, 8, 7, 9);
            }
            11 => {
                circle(e, 1, 9, 2, 10);
                circle(o, 1, 9, 2, 10);
            }
            12 => {
                swap4(e, 1, 9, 2, 10);
                swap4(o, 1, 9, 2, 10);
            }
            13 => {
                circle(e, 1, 10, 2, 9);
                circle(o, 1, 10, 2, 9);
            }
            14 => {
                swap4(e, 0, 4, 1, 5);
                swap4(o, 0, 4, 1, 5);
                e.swap(9, 11);
                o.swap(8, 10);
            }
            15 => {
                swap4(e, 5, 10, 6, 11);
                swap4(o, 5, 10, 6, 11);
                e.swap(1, 3);
                o.swap(0, 2);
            }
            16 => {
                swap4(e, 0, 11, 3, 8);
                swap4(o, 0, 11, 3, 8);
                e.swap(5, 7);
                o.swap(4, 6);
            }
            17 => {
                swap4(e, 2, 7, 3, 6);
                swap4(o, 2, 7, 3, 6);
                e.swap(8, 10);
                o.swap(9, 11);
            }
            18 => {
                swap4(e, 4, 8, 7, 9);
                swap4(o, 4, 8, 7, 9);
                e.swap(0, 2);
                o.swap(1, 3);
            }
            _ => {
                swap4(e, 1, 9, 2, 10);
                swap4(o, 1, 9, 2, 10);
                e.swap(4, 6);
                o.swap(5, 7);
            }
        }
    }

    fn swapx(&mut self, x: usize, y: usize) {
        std::mem::swap(&mut self.edge[x], &mut self.edgeo[y]);
    }

    fn circlex(&mut self, a: usize, b: usize, c: usize, d: usize) {
        let t = self.edgeo[d];
        self.edgeo[d] = self.edge[c];
        self.edge[c] = self.edgeo[b];
        self.edgeo[b] = self.edge[a];
        self.edge[a] = t;
    }

    fn rot(&mut self, r: usize) {
        self.std = false;
        match r {
            0 => {
                self.apply(14);
                self.apply(17);
            }
            1 => {
                self.circlex(11, 5, 10, 6);
                self.circlex(5, 10, 6, 11);
                self.circlex(1, 2, 3, 0);
                self.circlex(4, 9, 7, 8);
                self.circlex(8, 4, 9, 7);
                self.circlex(0, 1, 2, 3);
            }
            _ => {
                for (x, y) in [
                    (4, 5),
                    (5, 4),
                    (11, 8),
                    (8, 11),
                    (7, 6),
                    (6, 7),
                    (9, 10),
                    (10, 9),
                    (1, 1),
                    (0, 0),
                    (3, 3),
                    (2, 2),
                ] {
                    self.swapx(x, y);
                }
            }
        }
        // `apply` inside `rot(0)` clears the flag too; make it explicit.
        self.std = false;
    }

    /// Applies symmetry `r` (0 to 7).
    pub(super) fn rotate(&mut self, mut r: usize) {
        while r >= 2 {
            r -= 2;
            self.rot(1);
            self.rot(2);
        }
        if r != 0 {
            self.rot(0);
        }
    }
}

/// What a move leads to in the reduced space.
#[derive(Clone, Copy)]
pub(super) struct Moved {
    pub state: usize,
    pub class: usize,
    pub sym: usize,
}

/// The tables that relate placements, moves and symmetries.
pub(super) struct Edge3 {
    pub sym2raw: Vec<i32>,
    /// Bit `j` of `symstate[class]`: symmetry `j` fixes the class.
    pub symstate: Vec<u16>,
    pub raw2sym: Vec<i32>,
    /// `mvrot[m << 3 | r]`: the placement after move `m` then symmetry `r`
    /// (relabelled), and the relabelling.
    mvrot: Vec<[i32; 12]>,
    mvroto: Vec<[i32; 12]>,
}

impl Edge3 {
    #[allow(clippy::needless_range_loop)] // index loops as in the reference
    pub(super) fn new() -> Edge3 {
        let mut mvrot = vec![[0i32; 12]; 20 * 8];
        let mut mvroto = vec![[0i32; 12]; 20 * 8];
        let mut e = Edge::new();
        for m in 0..20 {
            for r in 0..8 {
                e.set(0);
                e.apply(m);
                e.rotate(r);
                mvrot[m << 3 | r] = e.edge;
                // `standardize` leaves the inverse of `edgeo` in a temporary:
                // that is what the lookups need.
                let mut temp = [0i32; 12];
                for i in 0..12 {
                    temp[e.edgeo[i] as usize] = i as i32;
                }
                mvroto[m << 3 | r] = temp;
            }
        }

        let mut sym2raw = vec![0i32; N_SYM];
        let mut symstate = vec![0u16; N_SYM];
        let mut raw2sym = vec![0i32; 11880];
        let mut seen = vec![false; 11880];
        let mut count = 0;
        for i in 0..11880 {
            if seen[i] {
                continue;
            }
            e.set(i as i32 * FACT_X[8]);
            for j in 0..8 {
                let idx = e.get(4) as usize;
                if idx == i {
                    symstate[count] |= 1 << j;
                }
                seen[idx] = true;
                raw2sym[idx] = (count << 3 | SYM_INV[j]) as i32;
                e.rot(0);
                if j % 2 == 1 {
                    e.rot(1);
                    e.rot(2);
                }
            }
            sym2raw[count] = i as i32;
            count += 1;
        }
        assert_eq!(count, N_SYM);
        Edge3 {
            sym2raw,
            symstate,
            raw2sym,
            mvrot,
            mvroto,
        }
    }

    /// The number of the first `end` places of the placement `ep` after the
    /// move-and-symmetry `mr_idx` (`move << 3 | symmetry`).
    #[inline(always)]
    pub(super) fn mvrot(&self, ep: &[i32; 12], mr_idx: usize, end: usize) -> i32 {
        let movo = &self.mvroto[mr_idx];
        let mov = &self.mvrot[mr_idx];
        let mut idx = 0;
        let mut val = ALL;
        for i in 0..end {
            let v = (movo[ep[mov[i] as usize] as usize] << 2) as u32;
            idx *= 12 - i as i32;
            idx += ((val >> v) & 0xf) as i32;
            val = val.wrapping_sub(ONES << v);
        }
        idx
    }

    /// Move `m` made on the placement `ep`: the reduced state it leads to,
    /// the symmetry that brought it into its class and the class.
    #[inline(always)]
    pub(super) fn after_move(&self, ep: &[i32; 12], m: usize) -> Moved {
        let cord1x = self.mvrot(ep, m << 3, 4);
        let sym_cord = self.raw2sym[cord1x as usize] as usize;
        let sym = sym_cord & 7;
        let class = sym_cord >> 3;
        let cord2x = self.mvrot(ep, m << 3 | sym, 10) as usize % N_RAW;
        Moved {
            state: class * N_RAW + cord2x,
            class,
            sym,
        }
    }

    /// The reduced state of `e` itself (the reference's `getsym`); rotates `e`.
    pub(super) fn reduced(&self, e: &mut Edge) -> usize {
        let cord1x = e.get(4);
        let sym_cord = self.raw2sym[cord1x as usize] as usize;
        e.rotate(sym_cord & 7);
        let cord2x = e.get(10) as usize % N_RAW;
        (sym_cord >> 3) * N_RAW + cord2x
    }
}

/// The moves of phase 3 that the search and the pruning table use.
pub(super) const SEARCH_MOVES: usize = 17;

#[cfg(test)]
mod tests {
    use super::super::testing::fnv;
    use super::*;

    #[test]
    fn numbers_round_trip() {
        let mut e = Edge::new();
        for idx in (0..FACT_X[12]).step_by(99_991) {
            e.set(idx);
            let mut parity = 0;
            for i in 0..12 {
                for j in i + 1..12 {
                    parity ^= i32::from(e.edge[i] > e.edge[j]);
                }
            }
            assert_eq!(parity, 0, "idx {idx} is an even permutation");
        }
    }

    #[test]
    fn tables_match_the_reference_checksums() {
        let t = Edge3::new();
        assert_eq!(
            fnv(t.sym2raw.iter().map(|&v| i64::from(v))),
            5736095049665229879
        );
        assert_eq!(
            fnv(t.raw2sym.iter().map(|&v| i64::from(v))),
            -7901824425262926915
        );
        assert_eq!(
            fnv(t.symstate.iter().map(|&v| i64::from(v))),
            -8731329763001593011
        );
        assert_eq!(
            fnv(t.mvrot.iter().flatten().map(|&v| i64::from(v))),
            -1783210213242930909
        );
        assert_eq!(
            fnv(t.mvroto.iter().flatten().map(|&v| i64::from(v))),
            584667883775575843
        );
    }
}
