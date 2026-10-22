//! Phase 1: the first search, 10,000 solutions of the first centre phase
//! kept as the 500 best.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Search.java (`doSearch`'s first
//! loop, `search1`, `init2`), MIT, Copyright (c) 2023 Chen Shuang; this file
//! is AGPL-3.0-or-later, see NOTICE.md.
//!
//! The search goes through the three kinds of centres (R/L first, then U/D,
//! then F/B), for lengths from the least pruning value up, and hands every
//! solution to `init2` until 10,000 have been seen. `init2` turns a solution
//! into a candidate and keeps the 500 with the smallest bound.

use super::candidates::{Cand, Heap};
use super::center1::Flags;
use super::center2::State;
use super::control::{Control, Stop, TICK};
use super::cube::{flips_wing_parity, move_centres, Cube};
use super::moves::{BX3, DX3, FX1, UX1};
use super::tables::Tables;

/// Solutions of the first phase looked at.
const PHASE1_SOLUTIONS: usize = 10_000;
/// Candidates kept for the second phase.
pub(super) const PHASE2_ATTEMPTS: usize = 500;
/// The longest first phase tried (the reference has no limit but the stack).
const MAX_LENGTH1: usize = 20;

struct Search<'a> {
    t: &'a Tables,
    ctl: &'a Control,
    base_ct: [u8; 24],
    base_parity: u8,
    move1: [usize; MAX_LENGTH1 + 2],
    length1: usize,
    heap: Heap,
    count: usize,
    nodes: u32,
    stop: Option<Stop>,
}

/// The candidates of phase 1 for `cube`, best first (ties in the order of the
/// reference).
pub(super) fn run(t: &Tables, cube: &Cube, ctl: &Control) -> Result<Vec<Cand>, Stop> {
    let mut s = Search {
        t,
        ctl,
        base_ct: cube.ct,
        base_parity: cube.wing_parity(),
        move1: [0; MAX_LENGTH1 + 2],
        length1: 0,
        heap: Heap::with_capacity(PHASE2_ATTEMPTS),
        count: 0,
        nodes: 0,
        stop: None,
    };
    let mut starts = [(0usize, 0usize, 0usize); 3];
    for (urf, start) in starts.iter_mut().enumerate() {
        let flags = Flags::of_centres(&cube.ct, urf as u8);
        let Some(cs) = flags.class_sym(&t.c1.sym2raw) else {
            return Err(Stop::Internal("a centre state has no symmetry class"));
        };
        *start = (
            (cs >> 6) as usize,
            (cs & 63) as usize,
            usize::from(t.c1.csprun[(cs >> 6) as usize]),
        );
    }
    let [ud, fb, rl] = starts;
    let least = ud.2.min(fb.2).min(rl.2);
    for length1 in least..MAX_LENGTH1 {
        s.length1 = length1;
        let done = (rl.2 <= length1 && s.search1(rl.0, rl.1, length1 as i32, -1, 0))
            || (ud.2 <= length1 && s.search1(ud.0, ud.1, length1 as i32, -1, 0))
            || (fb.2 <= length1 && s.search1(fb.0, fb.1, length1 as i32, -1, 0));
        if let Some(stop) = s.stop {
            return Err(stop);
        }
        if done {
            return Ok(s.heap.into_sorted());
        }
        if ctl.check() {
            return Err(Stop::Timeout);
        }
    }
    if s.count > 0 {
        return Ok(s.heap.into_sorted());
    }
    Err(Stop::Exhausted("no first phase within 20 moves"))
}

impl Search<'_> {
    fn search1(&mut self, ct: usize, sym: usize, maxl: i32, lm: i32, depth: usize) -> bool {
        if ct == 0 && maxl < 5 {
            return maxl == 0 && self.init2(sym);
        }
        self.nodes += 1;
        if self.nodes & TICK == 0 && self.ctl.check() {
            self.stop = Some(Stop::Timeout);
            return true;
        }
        let t = self.t;
        for axis in (0..27).step_by(3) {
            let a = axis as i32;
            if a == lm || a == lm - 9 || a == lm - 18 {
                continue;
            }
            for power in 0..3 {
                let m = axis + power;
                let ctx = t.c1.ctsmv[ct * 36 + usize::from(t.syms.mv[sym][m])];
                let prun = i32::from(t.c1.csprun[(ctx >> 6) as usize]);
                if prun >= maxl {
                    if prun > maxl {
                        break;
                    }
                    continue;
                }
                let symx = t.syms.mult(sym, (ctx & 0x3f) as usize);
                self.move1[depth] = m;
                if self.search1((ctx >> 6) as usize, symx, maxl - 1, a, depth + 1) {
                    return true;
                }
            }
        }
        false
    }

    /// A solution of the first phase: make a candidate of it.
    fn init2(&mut self, sym: usize) -> bool {
        let mut cand = Cand::empty();
        let mut ct = self.base_ct;
        let mut parity = self.base_parity;
        let mut turn = |cand: &mut Cand, m: usize| {
            cand.push(m);
            move_centres(&mut ct, m);
            parity ^= u8::from(flips_wing_parity(m));
        };
        for i in 0..self.length1 {
            turn(&mut cand, self.move1[i]);
        }
        let (add1, model_sym) = match self.t.syms.finish[sym] {
            0 => {
                turn(&mut cand, FX1);
                turn(&mut cand, BX3);
                (true, 19)
            }
            12869 => {
                turn(&mut cand, UX1);
                turn(&mut cand, DX3);
                (true, 34)
            }
            735470 => (false, 0),
            _ => {
                self.stop = Some(Stop::Internal("a first phase ended in no known position"));
                return true;
            }
        };
        let Some((s2ct, s2rl)) = State::of(&ct, parity).numbers() else {
            self.stop = Some(Stop::Internal(
                "the centres after the first phase do not fit the second",
            ));
            return true;
        };
        let ctp = i32::from(self.t.c2.ctprun[s2ct * 70 + s2rl]);
        cand.value = ctp + self.length1 as i32;
        cand.length1 = self.length1 as u8;
        cand.add1 = add1;
        cand.sym = model_sym;
        self.count += 1;

        // A queue of 500 that keeps the smaller values: when it is full, the
        // front (the greatest value) goes, or the new one does not get in.
        if self.heap.len() < PHASE2_ATTEMPTS {
            self.heap.add(cand);
        } else {
            let mut top = self.heap.poll().expect("a full queue");
            if top.value > cand.value {
                top = cand;
            }
            self.heap.add(top);
        }
        self.count == PHASE1_SOLUTIONS
    }
}
