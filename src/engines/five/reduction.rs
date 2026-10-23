//! The five phases in a row, with the candidate lists between them.
//!
//! Ported from cs0x7f/cube555 src/Search.java (`solveReduction`), MIT,
//! Copyright (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see
//! NOTICE.md. Phase 1 starts from the cube seen three ways (turned by the
//! symmetry 16 twice) and keeps 200 solutions; each is a start of phase 2
//! (500 kept), and so on to phase 5, which keeps one.

use std::time::Instant;

use super::cubie::Cubie;
use super::p1::Stage1;
use super::p2::Stage2;
use super::p3::Stage3;
use super::p4::Stage4;
use super::p5::Stage5;
use super::solving::Solving;
use super::tables::Tables;
use crate::reduction::ida::{self, Collector, Stage, Start, TimedOut};
use crate::reduction::par;

const KEEP: [usize; 5] = [200, 500, 500, 500, 1];

/// A reduction: its moves (numbers 0 to 35) and, for each of the five
/// phases, the number of moves made when it ended.
pub(super) struct Reduction {
    pub moves: Vec<u8>,
    pub ends: Vec<usize>,
}

impl Reduction {
    fn of(sc: Solving) -> Reduction {
        Reduction {
            moves: sc.moves,
            ends: sc.ends,
        }
    }
}

/// Why the reduction gave no result.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Stuck {
    TimedOut,
    /// No phase-5 solution within the searched lengths.
    NoWay,
}

/// Turns the moves found from a base cube into the cubes that go on.
struct Step<'a, F: Fn(Solving, &mut Vec<Solving>) + Sync> {
    syms: &'a super::cubie_sym::Syms,
    bases: &'a [Solving],
    finish: F,
}

impl<F: Fn(Solving, &mut Vec<Solving>) + Sync> Collector for Step<'_, F> {
    type Out = Solving;

    fn collect(&self, owner: usize, moves: &[u8], out: &mut Vec<Solving>) {
        let mut sc = self.bases[owner].clone();
        sc.do_moves(self.syms, moves);
        sc.ends.push(sc.moves.len());
        (self.finish)(sc, out);
    }
}

/// One phase: from the candidates `bases`, each giving start nodes through
/// `starts`, to the cubes `finish` makes of the solutions.
// Eight parameters: the stage and the search settings are what the five phases
// differ in, and bundling them in a struct would only move the list elsewhere.
#[allow(clippy::too_many_arguments)]
fn phase<S: Stage>(
    stage: &S,
    syms: &super::cubie_sym::Syms,
    bases: &[Solving],
    starts: impl Fn(&Solving) -> Vec<S::Node> + Sync,
    quota: usize,
    threads: usize,
    deadline: Option<Instant>,
    finish: impl Fn(Solving, &mut Vec<Solving>) + Sync,
) -> Result<Vec<Solving>, Stuck> {
    // The start nodes of different candidates are independent: make them in
    // parallel, then lay them out in candidate order.
    let slots: Vec<std::sync::Mutex<Vec<S::Node>>> = bases
        .iter()
        .map(|_| std::sync::Mutex::new(Vec::new()))
        .collect();
    par::for_chunks(threads, bases.len(), 8, |range| {
        for i in range {
            *slots[i].lock().expect("starts") = starts(&bases[i]);
        }
    });
    let mut all = Vec::new();
    for (owner, slot) in slots.into_iter().enumerate() {
        for node in slot.into_inner().expect("starts") {
            all.push(Start {
                node,
                cost: bases[owner].cost(),
                owner,
            });
        }
    }
    let collector = Step {
        syms,
        bases,
        finish,
    };
    ida::search(stage, &all, &collector, quota, threads, deadline)
        .map_err(|TimedOut| Stuck::TimedOut)
}

/// The moves (numbers 0 to 35) that bring the centres and wings of `cube`
/// into a state where only a 3x3x3 is left.
pub(super) fn reduce(
    tables: &Tables,
    cube: &Cubie,
    threads: usize,
    deadline: Option<Instant>,
) -> Result<Reduction, Stuck> {
    let sols4 = first_four(tables, cube, threads, deadline)?;
    let sols5 = last(tables, &sols4, 1, threads, deadline)?;
    sols5
        .into_iter()
        .next()
        .map(Reduction::of)
        .ok_or(Stuck::NoWay)
}

/// Up to `n` reductions of one cube, the best first: the first `n` results of
/// phase 5 in the serial order, so that the list does not depend on the
/// thread count and starts with the reduction of [`reduce`].
///
/// Phase 5 collects under `early`, a deadline before `deadline`; if only that
/// one runs out, the single reduction of [`reduce`] is found instead.
pub(super) fn reduce_many(
    tables: &Tables,
    cube: &Cubie,
    threads: usize,
    (deadline, early): (Option<Instant>, Option<Instant>),
    n: usize,
) -> Result<Vec<Reduction>, Stuck> {
    let sols4 = first_four(tables, cube, threads, deadline)?;
    let sols5 = match last(tables, &sols4, n, threads, early) {
        Err(Stuck::TimedOut)
            if early != deadline && deadline.is_none_or(|d| Instant::now() < d) =>
        {
            last(tables, &sols4, 1, threads, deadline)?
        }
        other => other?,
    };
    if sols5.is_empty() {
        return Err(Stuck::NoWay);
    }
    Ok(sols5.into_iter().take(n).map(Reduction::of).collect())
}

fn last(
    tables: &Tables,
    sols4: &[Solving],
    quota: usize,
    threads: usize,
    deadline: Option<Instant>,
) -> Result<Vec<Solving>, Stuck> {
    phase(
        &Stage5(&tables.p5),
        &tables.syms,
        sols4,
        |sc| vec![tables.p5.start(&sc.cube)],
        quota,
        threads,
        deadline,
        |sc, out| out.push(sc),
    )
}

/// Phases 1 to 4.
fn first_four(
    tables: &Tables,
    cube: &Cubie,
    threads: usize,
    deadline: Option<Instant>,
) -> Result<Vec<Solving>, Stuck> {
    let syms = &tables.syms;
    // Phase 1: the cube as it is and turned twice.
    let mut sc = Solving::new(cube.clone());
    let mut first = Vec::with_capacity(3);
    for _ in 0..3 {
        first.push(sc.clone());
        sc.do_conj(syms, 16);
    }
    let sols1 = phase(
        &Stage1 {
            p: &tables.p1,
            syms,
        },
        syms,
        &first,
        |sc| vec![tables.p1.start(&sc.cube)],
        KEEP[0],
        threads,
        deadline,
        |sc, out| out.push(sc),
    )?;
    // Phase 2: every solution, and its two turned copies.
    let sols2 = phase(
        &Stage2(&tables.p2),
        syms,
        &sols1,
        |sc| vec![tables.p2.start(&sc.cube)],
        KEEP[1],
        threads,
        deadline,
        |mut sc, out| {
            for _ in 0..3 {
                out.push(sc.clone());
                sc.do_conj(syms, 16);
            }
        },
    )?;
    // Phase 3: kept if the wings of a layer fit in eight edge slots.
    let sols3 = phase(
        &Stage3 {
            p: &tables.p3,
            syms,
        },
        syms,
        &sols2,
        |sc| tables.p3.starts(&sc.cube, syms),
        KEEP[2],
        threads,
        deadline,
        |mut sc, out| {
            let (mut y, mut z) = (0u32, 0u32);
            for i in 0..4 {
                let c = &sc.cube;
                y |= 1 << (c.wedge[8 + i] % 12);
                y |= 1 << (c.wedge[8 + i + 12] % 12);
                y |= 1 << (c.medge[8 + i] >> 1);
                z |= 1 << (c.wedge[4 + i] % 12);
                z |= 1 << (c.wedge[4 + i + 12] % 12);
                z |= 1 << (c.medge[4 + i] >> 1);
            }
            let (y, z) = (y.count_ones() <= 8, z.count_ones() <= 8);
            if z {
                sc.do_conj(syms, 1);
            }
            // As in the original, when both hold the first entry is the turned cube too.
            for _ in 0..usize::from(y) + usize::from(z) {
                out.push(sc.clone());
            }
        },
    )?;
    let sols4 = phase(
        &Stage4(&tables.p4),
        syms,
        &sols3,
        |sc| tables.p4.starts(&sc.cube),
        KEEP[3],
        threads,
        deadline,
        |mut sc, out| {
            sc.do_conj(syms, 1);
            out.push(sc);
        },
    )?;
    Ok(sols4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::five::tables::Config;

    const TRANSFER: &str = include_str!("../../../tests/fixtures/five_transfer.txt");

    /// `U`, `r2`, `F'` as move numbers.
    fn code(token: &str) -> u8 {
        let mut chars = token.chars();
        let letter = chars.next().expect("a move");
        let face = "URFDLB".find(letter.to_ascii_uppercase()).expect("a face") as u8;
        let turn = match chars.next() {
            None => 0,
            Some('2') => 1,
            Some('\'') => 2,
            Some(other) => panic!("turn {other}"),
        };
        face * 3 + turn + if letter.is_ascii_lowercase() { 18 } else { 0 }
    }

    fn cube_of(facelets: &str) -> Cubie {
        let mut colours = [0u8; 150];
        for (c, f) in colours.iter_mut().zip(facelets.chars()) {
            *c = "URFDLB".find(f).expect("a colour") as u8;
        }
        Cubie::from_colours(&colours).expect("a valid cube")
    }

    /// The Rust port finds the very reduction the Java solver found.
    /// Needs the full tables (a few seconds to build, 237 MB).
    #[test]
    #[ignore = "builds the full tables"]
    fn java_transfer_for_any_thread_count() {
        let config = |threads| Config {
            threads,
            cache_dir: None,
            cap: 15,
        };
        let tables = Tables::build(&config(4));
        for (n, line) in TRANSFER.lines().filter(|l| !l.starts_with('#')).enumerate() {
            let (facelets, moves) = line.split_once('\t').expect("facelets and moves");
            let expected: Vec<u8> = moves.split_whitespace().map(code).collect();
            let cube = cube_of(facelets);
            for threads in [4, 1] {
                if threads == 1 && n >= 6 {
                    continue;
                }
                let got = reduce(&tables, &cube, threads, None)
                    .expect("a reduction")
                    .moves;
                assert_eq!(got, expected, "cube {n} with {threads} threads");
            }
        }
    }
}
