//! Phase 3: from a result of phase 2, the last moves that solve the centers and
//! pair the wings, so that a 3x3x3 is left.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Search.java (`doSearch`'s third
//! loop, `search3`), MIT, Copyright (c) 2023 Chen Shuang; this file is
//! AGPL-3.0-or-later, see NOTICE.md.
//!
//! For rising total length, the results of phase 2 (best bound first, the
//! first 100) are tried in order, each for a solution of exactly the moves
//! that remain (at most `max_length3`, 13 to start with); the first to
//! succeed wins. The (length, result) pairs are searched on several threads
//! and the first that succeeds in the serial order is taken.

use super::candidates::Cand;
use super::control::{Control, Stop, TICK};
use super::edge3::{Edge, N_RAW};
use super::moves::{MOVE3STD, SET3};
use super::tables::Tables;
use crate::reduction::par;

/// Results of phase 2 that are tried.
const PHASE3_ATTEMPTS: usize = 100;
const MAX_TOTAL: i32 = 100;
const MAX_LENGTH3_LIMIT: i32 = 20;

struct Search<'a> {
    t: &'a Tables,
    ctl: &'a Control,
    cancelled: &'a dyn Fn() -> bool,
    /// The first move to search, or `ALL`.
    only: usize,
    tempe: [Edge; 24],
    move3: [usize; 24],
    nodes: u32,
    stop: Option<Stop>,
}

/// Search every first move.
const ALL: usize = usize::MAX;
/// First moves of phase 3 (the search tries the moves 0 to 16).
const FIRST_MOVES: usize = 17;

/// The phase 3 moves (as move numbers) that finish `cand` in exactly
/// `remaining` moves and begin with move `only` (any for `ALL`), if there
/// are any.
pub(super) fn one(
    t: &Tables,
    ctl: &Control,
    cancelled: &dyn Fn() -> bool,
    cand: &Cand,
    remaining: i32,
    only: usize,
) -> Result<Option<Vec<usize>>, Stop> {
    if cand.prun3 > remaining {
        return Ok(None);
    }
    if ctl.check() {
        return Err(Stop::Timeout);
    }
    let mut s = Search {
        t,
        ctl,
        cancelled,
        only,
        tempe: [Edge::new(); 24],
        move3: [0; 24],
        nodes: 0,
        stop: None,
    };
    let found = s.search3(cand.edge, cand.ct3 as usize, cand.prun3, remaining, 20, 0);
    match (s.stop, found) {
        (Some(stop), _) => Err(stop),
        (None, true) => Ok(Some(
            s.move3[..remaining as usize]
                .iter()
                .map(|&m| MOVE3STD[m])
                .collect(),
        )),
        (None, false) => Ok(None),
    }
}

/// The result of phase 2 that phase 3 finishes first, with its moves.
pub(super) fn run(
    t: &Tables,
    results: &[Cand],
    threads: usize,
    ctl: &Control,
) -> Result<(usize, Vec<usize>), Stop> {
    let mut found = run_many(t, results, threads, ctl, 1)?;
    Ok(found.swap_remove(0))
}

/// The first `need` successes in the serial order (fewer if the attempts at
/// the first `max_length3` that has any hold fewer), best first: a result of
/// phase 2 with its moves, the same result again when more than one first
/// move or total length succeeds.
pub(super) fn run_many(
    t: &Tables,
    results: &[Cand],
    threads: usize,
    ctl: &Control,
    need: usize,
) -> Result<Vec<(usize, Vec<usize>)>, Stop> {
    let attempts = results.len().min(PHASE3_ATTEMPTS);
    let mut max_length3 = 13;
    while max_length3 <= MAX_LENGTH3_LIMIT {
        // Every (total length, result, first move) triple in the order the
        // serial loop visits them; the first to succeed in this order wins.
        let tasks: Vec<(i32, usize, usize)> = (results[0].value..MAX_TOTAL)
            .flat_map(|length123| {
                (0..attempts)
                    .take_while(move |&i| results[i].value <= length123)
                    .filter(move |&i| {
                        length123 - i32::from(results[i].length1) - i32::from(results[i].length2)
                            <= max_length3
                    })
                    .flat_map(move |i| {
                        let remaining = length123
                            - i32::from(results[i].length1)
                            - i32::from(results[i].length2);
                        if remaining == 0 {
                            vec![(length123, i, ALL)]
                        } else {
                            (0..FIRST_MOVES).map(|m| (length123, i, m)).collect()
                        }
                    })
            })
            .collect();
        let failed = std::sync::Mutex::new(None);
        let got = par::first_results(threads, tasks.len(), need, |k, cancelled| {
            let (length123, i, first) = tasks[k];
            let remaining =
                length123 - i32::from(results[i].length1) - i32::from(results[i].length2);
            match one(t, ctl, cancelled, &results[i], remaining, first) {
                Ok(found) => found.map(|moves| (i, moves)).into_iter().collect(),
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
        if !got.is_empty() {
            return Ok(got);
        }
        if ctl.check() {
            return Err(Stop::Timeout);
        }
        max_length3 += 1;
    }
    Err(Stop::Exhausted("no third phase within the limits"))
}

impl Search<'_> {
    fn search3(
        &mut self,
        edge: i32,
        ct: usize,
        prun: i32,
        maxl: i32,
        lm: usize,
        depth: usize,
    ) -> bool {
        if maxl == 0 {
            return edge == 0 && ct == 0;
        }
        self.nodes += 1;
        if self.nodes & TICK == 0 && ((self.cancelled)() || self.ctl.check()) {
            if self.ctl.expired() {
                self.stop = Some(Stop::Timeout);
            }
            return true;
        }
        let t = self.t;
        self.tempe[depth].set(edge);
        let ep = self.tempe[depth].edge;
        let mut m = 0;
        while m < 17 {
            if SET3.blocked[lm][m] {
                m = SET3.skip[m] + 1;
                continue;
            }
            let ctx = usize::from(t.c3.ctmove[ct * 20 + m]);
            let prun1 = i32::from(t.c3.prun[ctx]);
            if prun1 >= maxl {
                if prun1 > maxl && m < 14 {
                    m = SET3.skip[m];
                }
                m += 1;
                continue;
            }
            let edgex = t.e3.mvrot(&ep, m << 3, 10);
            let sym_cord = t.e3.raw2sym[edgex as usize / N_RAW] as usize;
            let sym = sym_cord & 7;
            let cord2x = t.e3.mvrot(&ep, m << 3 | sym, 10) as usize % N_RAW;
            let prunx = t.wings.relative((sym_cord >> 3) * N_RAW + cord2x, prun);
            if prunx >= maxl {
                if prunx > maxl && m < 14 {
                    m = SET3.skip[m];
                }
                m += 1;
                continue;
            }
            if depth == 0 && self.only != ALL && m != self.only {
                m += 1;
                continue;
            }
            if self.search3(edgex, ctx, prunx, maxl - 1, m, depth + 1) {
                self.move3[depth] = m;
                return true;
            }
            m += 1;
        }
        false
    }
}
