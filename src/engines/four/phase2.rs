//! Phase 2: from each candidate of phase 1, the centres sorted far enough and the wings split so that
//! the last phase can finish, until 100 results are found.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Search.java (`doSearch`'s second
//! loop, `search2`, `init3`), MIT, Copyright (c) 2023 Chen Shuang; this file
//! is AGPL-3.0-or-later, see NOTICE.md.
//!
//! For rising total length `length12`, every candidate whose bound allows it
//! is searched for solutions of exactly `length12 - length1` moves (at most
//! `max_length2`, 9 to start with). The reference stops at the 100th result;
//! here the candidates of one length are searched on several threads and
//! their results joined in candidate order, so the first 100 are the same.

use super::candidates::Cand;
use super::center3::State as Centres3;
use super::control::{Control, Stop, TICK};
use super::cube::{move_centres, move_wings, wings_pairable, CornerMoves, Cube};
use super::edge3::Edge;
use super::moves::{MOVE2STD, SET2};
use super::tables::Tables;
use crate::reduction::par;

/// Results the second phase collects.
pub(super) const PHASE2_SOLUTIONS: usize = 100;
/// The longest total length tried before the allowed phase 2 length grows.
const MAX_TOTAL: i32 = 100;
/// The phase 2 length is raised at most up to this.
const MAX_LENGTH2_LIMIT: i32 = 20;

struct Search<'a> {
    t: &'a Tables,
    ctl: &'a Control,
    cancelled: &'a dyn Fn() -> bool,
    cube: &'a Cube,
    cand: &'a Cand,
    length2: usize,
    move2: [usize; 32],
    out: Vec<Cand>,
    limit: usize,
    /// The first move to search (the others are left to other tasks), or
    /// `ALL` to search them all.
    only: usize,
    nodes: u32,
    stop: Option<Stop>,
    e12: Edge,
}

/// Search every first move.
const ALL: usize = usize::MAX;
/// First moves of phase 2 (the search tries the moves 0 to 22).
const FIRST_MOVES: usize = 23;

/// The results of searching candidate `cand` (whose state is `cube`) for
/// solutions of exactly `length2` moves that begin with move `only` (any
/// move for `ALL`), at most `limit` of them.
#[allow(clippy::too_many_arguments)] // one task: the state, the lengths and the clock
pub(super) fn one(
    t: &Tables,
    ctl: &Control,
    cancelled: &dyn Fn() -> bool,
    cand: &Cand,
    cube: &Cube,
    length2: usize,
    limit: usize,
    only: usize,
) -> Result<Vec<Cand>, Stop> {
    if ctl.check() {
        return Err(Stop::Timeout);
    }
    let mut s = Search {
        t,
        ctl,
        cancelled,
        cube,
        cand,
        length2,
        move2: [0; 32],
        out: Vec::new(),
        limit,
        only,
        nodes: 0,
        stop: None,
        e12: Edge::new(),
    };
    let c2 = super::center2::State::of(&cube.ct, cube.wing_parity());
    let Some((ct, rl)) = c2.numbers() else {
        return Err(Stop::Internal(
            "the centres after the first phase do not fit the second",
        ));
    };
    s.search2(ct, rl, length2 as i32, 28, 0);
    match s.stop {
        Some(stop) => Err(stop),
        None => Ok(s.out),
    }
}

/// The first 100 results over all candidates, as the reference finds them.
pub(super) fn run(
    t: &Tables,
    cands: &[Cand],
    cubes: &[Cube],
    threads: usize,
    ctl: &Control,
) -> Result<Vec<Cand>, Stop> {
    let mut results: Vec<Cand> = Vec::with_capacity(PHASE2_SOLUTIONS);
    let mut max_length2 = 9;
    while max_length2 <= MAX_LENGTH2_LIMIT {
        for length12 in cands[0].value..MAX_TOTAL {
            // Each candidate is split by its first move, in the order the
            // serial search takes them, so the joined results are the serial
            // ones. A candidate with nothing left to move is one task.
            let tasks: Vec<(usize, usize)> = (0..cands.len())
                .take_while(|&i| cands[i].value <= length12)
                .filter(|&i| length12 - i32::from(cands[i].length1) <= max_length2)
                .flat_map(|i| {
                    if length12 == i32::from(cands[i].length1) {
                        vec![(i, ALL)]
                    } else {
                        (0..FIRST_MOVES).map(|m| (i, m)).collect()
                    }
                })
                .collect();
            if tasks.is_empty() {
                continue;
            }
            let need = PHASE2_SOLUTIONS - results.len();
            let failed = std::sync::Mutex::new(None);
            let got = par::first_results(threads, tasks.len(), need, |k, cancelled| {
                let (i, first) = tasks[k];
                let length2 = (length12 - i32::from(cands[i].length1)) as usize;
                match one(
                    t, ctl, cancelled, &cands[i], &cubes[i], length2, need, first,
                ) {
                    Ok(found) => found,
                    Err(stop) => {
                        *failed.lock().expect("no panic while holding the lock") = Some(stop);
                        Vec::new()
                    }
                }
            });
            if let Some(stop) = failed
                .into_inner()
                .expect("no panic while holding the lock")
            {
                return Err(stop);
            }
            if ctl.check() {
                return Err(Stop::Timeout);
            }
            results.extend(got);
            if results.len() == PHASE2_SOLUTIONS {
                return Ok(results);
            }
        }
        max_length2 += 1;
    }
    Err(Stop::Exhausted("no second phase within the limits"))
}

impl Search<'_> {
    fn search2(&mut self, ct: usize, rl: usize, maxl: i32, lm: usize, depth: usize) -> bool {
        if ct == 0 && self.t.c2.ctprun[rl] == 0 && maxl == 0 {
            return self.init3(depth);
        }
        self.nodes += 1;
        if self.nodes & TICK == 0 && ((self.cancelled)() || self.ctl.check()) {
            if self.ctl.expired() {
                self.stop = Some(Stop::Timeout);
            }
            return true;
        }
        let c2 = &self.t.c2;
        let mut m = 0;
        while m < 23 {
            if SET2.blocked[lm][m] {
                m = SET2.skip[m] + 1;
                continue;
            }
            let ctx = usize::from(c2.ctmv[ct * 28 + m]);
            let rlx = usize::from(c2.rlmv[rl * 28 + m]);
            let prun = i32::from(c2.ctprun[ctx * 70 + rlx]);
            if prun >= maxl {
                if prun > maxl {
                    m = SET2.skip[m];
                }
                m += 1;
                continue;
            }
            if depth == 0 && self.only != ALL && m != self.only {
                m += 1;
                continue;
            }
            self.move2[depth] = MOVE2STD[m];
            if self.search2(ctx, rlx, maxl - 1, m, depth + 1) {
                return true;
            }
            m += 1;
        }
        false
    }

    /// A solution of the second phase: keep it if the wings can be paired.
    fn init3(&mut self, length2: usize) -> bool {
        debug_assert_eq!(length2, self.length2);
        let mut ep = self.cube.ep;
        for &m in &self.move2[..length2] {
            move_wings(&mut ep, m);
        }
        if !wings_pairable(&ep) {
            return false;
        }
        let cm: &CornerMoves = &self.t.corner_moves;
        let mut c2 = *self.cube;
        c2.ep = ep;
        for &m in &self.move2[..length2] {
            move_centres(&mut c2.ct, m);
            cm.apply(&mut c2.corners, m);
        }
        let eparity = self.e12.set_from(&c2);
        let Some(ct) = Centres3::of(&c2.ct, eparity ^ c2.corners.parity()).number() else {
            self.stop = Some(Stop::Internal(
                "the centres after the second phase do not fit the third",
            ));
            return true;
        };
        let edge = self.e12.get(10);
        let prun = self
            .t
            .wings
            .exact(&self.t.e3, self.t.e3.reduced(&mut self.e12));

        let mut next = *self.cand;
        for &m in &self.move2[..length2] {
            if !next.push(m) {
                self.stop = Some(Stop::Exhausted("a partial solution is too long"));
                return true;
            }
        }
        next.length2 = length2 as u8;
        next.ct3 = ct as u32;
        next.edge = edge;
        next.prun3 = prun;
        next.value =
            i32::from(next.length1) + length2 as i32 + prun.max(i32::from(self.t.c3.prun[ct]));
        self.out.push(next);
        self.out.len() == self.limit
    }
}
