//! Iterative-deepening search over one stage of a reduction, in parallel.
//!
//! Ported from cs0x7f/cube555 src/PhaseSearch.java, MIT, Copyright (c) 2023
//! Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! A stage starts from several nodes. For rising total length the search
//! visits the nodes in order of `cost so far + pruning value` (ties by cost,
//! then by position) and enumerates every solution of exactly that length by
//! depth-first search, handing each to a collector, until the collector has
//! enough outputs. This file adds the parallel run: the depth-first searches
//! are cut into tasks in the order the serial search would visit them, the
//! tasks run on any number of threads, and their results are merged in task
//! order, so the outputs are the same for every thread count.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use super::par;

/// The deepest solution a stage may need.
const MAX_DEPTH: usize = 40;
/// Total lengths tried before giving up.
const MAX_LENGTH: i32 = 100;
/// Nodes between two looks at the clock.
const TICK: u32 = 0x3fff;

/// One stage: a node type and the moves between nodes.
pub(crate) trait Stage: Sync {
    type Node: Copy + Send + Sync;

    fn move_count(&self) -> usize;

    /// The code of move `m`, as the caller wants it back in a solution.
    fn move_code(&self, m: usize) -> u8;

    /// Moves not to try after move `last` (`move_count()` stands for none),
    /// as a bit mask over move numbers.
    fn skips(&self) -> &[u64];

    /// Bit pair per move: after a move that did not get closer, skip its
    /// next powers.
    fn next_axis(&self) -> u64;

    /// A node that is a goal is only accepted if fewer than this many moves
    /// remain (so a search does not walk away from a goal and back).
    fn min_back_depth(&self) -> i32;

    /// The pruning value of a node.
    fn prun(&self, node: &Self::Node) -> i32;

    /// The node after move `mv` and its pruning value, or something at least
    /// `maxl` when it cannot be solved in the `maxl` moves left.
    fn step(&self, node: &Self::Node, mv: usize, maxl: i32) -> (Self::Node, i32);
}

/// A start node of a stage. `owner` says which candidate it came from.
pub(crate) struct Start<N> {
    pub node: N,
    pub cost: i32,
    pub owner: usize,
}

/// Turns the moves of a solution into outputs.
pub(crate) trait Collector: Sync {
    type Out: Send;

    /// Adds the outputs of the solution `moves` found from start `owner`.
    /// Pure: the same solution gives the same outputs.
    fn collect(&self, owner: usize, moves: &[u8], out: &mut Vec<Self::Out>);
}

/// The search stopped on the deadline.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TimedOut;

struct Task<N> {
    owner: usize,
    node: N,
    depth: usize,
    maxl: i32,
    last: usize,
    prun: i32,
    prefix: [u8; MAX_DEPTH],
}

impl<N: Copy> Clone for Task<N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: Copy> Copy for Task<N> {}

/// What a task found: one group of outputs per solution.
struct Found<O> {
    groups: Vec<Vec<O>>,
}

struct Control<'a> {
    stop: &'a AtomicBool,
    timed_out: &'a AtomicBool,
    deadline: Option<Instant>,
    ticks: Cell<u32>,
}

impl Control<'_> {
    fn should_stop(&self) -> bool {
        let ticks = self.ticks.get().wrapping_add(1);
        self.ticks.set(ticks);
        if ticks & TICK != 0 {
            return false;
        }
        if self.stop.load(Ordering::Relaxed) {
            return true;
        }
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.timed_out.store(true, Ordering::Relaxed);
            self.stop.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }
}

/// The moves to try from a node, in order, with the skipping of powers.
fn children<S: Stage>(stage: &S, task: &Task<S::Node>, mut each: impl FnMut(usize, S::Node, i32)) {
    let skip = stage.skips()[task.last];
    let next_axis = stage.next_axis();
    let mut mv = 0;
    while mv < stage.move_count() {
        if (skip >> mv) & 1 != 0 {
            mv += 1;
            continue;
        }
        let (child, prun) = stage.step(&task.node, mv, task.maxl);
        if prun >= task.maxl {
            mv += ((next_axis >> mv) & 3 & i64::from(task.maxl - prun) as u64) as usize;
            mv += 1;
            continue;
        }
        each(mv, child, prun);
        mv += 1;
    }
}

/// Depth-first search of one task; true when it must stop.
fn dfs<S: Stage>(
    stage: &S,
    task: &mut Task<S::Node>,
    ctl: &Control,
    sink: &mut impl FnMut(usize, &[u8]) -> bool,
) -> bool {
    if ctl.should_stop() {
        return true;
    }
    if task.prun == 0 && task.maxl < stage.min_back_depth() {
        return task.maxl == 0 && sink(task.owner, &task.prefix[..task.depth]);
    }
    if task.depth >= MAX_DEPTH {
        // The prefix is full; no stage searches this deep.
        return false;
    }
    let mut stop = false;
    let here = *task;
    children(stage, &here, |mv, child, prun| {
        if stop {
            return;
        }
        let mut next = here;
        next.node = child;
        next.depth += 1;
        next.maxl -= 1;
        next.last = mv;
        next.prun = prun;
        next.prefix[here.depth] = stage.move_code(mv);
        stop = dfs(stage, &mut next, ctl, sink);
    });
    stop
}

/// Cuts tasks one level deeper, keeping the order of the serial search.
fn split<S: Stage>(stage: &S, tasks: &[Task<S::Node>]) -> Vec<Task<S::Node>> {
    let mut out = Vec::with_capacity(tasks.len() * 8);
    for task in tasks {
        if task.depth + 1 >= MAX_DEPTH
            || task.maxl == 0
            || (task.prun == 0 && task.maxl < stage.min_back_depth())
        {
            out.push(*task);
            continue;
        }
        children(stage, task, |mv, child, prun| {
            let mut next = *task;
            next.node = child;
            next.depth += 1;
            next.maxl -= 1;
            next.last = mv;
            next.prun = prun;
            next.prefix[task.depth] = stage.move_code(mv);
            out.push(next);
        });
    }
    out
}

struct Merge<O> {
    slots: Vec<Option<Found<O>>>,
    next: usize,
    outs: Vec<O>,
    done: bool,
}

/// Runs the tasks and merges their outputs in task order until `quota`
/// outputs exist. Returns the outputs (possibly a few over the quota, as the
/// serial search is: a solution adds all its outputs before the count is
/// checked).
fn run_tasks<S: Stage, C: Collector>(
    stage: &S,
    tasks: &[Task<S::Node>],
    collector: &C,
    quota: usize,
    threads: usize,
    ctl_flags: (&AtomicBool, &AtomicBool, Option<Instant>),
) -> Option<Vec<C::Out>> {
    let (stop, timed_out, deadline) = ctl_flags;
    let merge = Mutex::new(Merge {
        slots: (0..tasks.len()).map(|_| None).collect(),
        next: 0,
        outs: Vec::new(),
        done: false,
    });
    let claim = AtomicUsize::new(0);
    par::run(threads.min(tasks.len()).max(1), |_| {
        let ctl = Control {
            stop,
            timed_out,
            deadline,
            ticks: Cell::new(0),
        };
        loop {
            let index = claim.fetch_add(1, Ordering::Relaxed);
            if index >= tasks.len() || stop.load(Ordering::Relaxed) {
                return;
            }
            let mut task = tasks[index];
            let mut found = Found { groups: Vec::new() };
            let mut count = 0;
            let mut sink = |owner: usize, moves: &[u8]| {
                let mut group = Vec::new();
                collector.collect(owner, moves, &mut group);
                count += group.len();
                found.groups.push(group);
                count >= quota
            };
            dfs(stage, &mut task, &ctl, &mut sink);
            if timed_out.load(Ordering::Relaxed) {
                return;
            }
            let mut guard = merge.lock().expect("merge lock");
            let state = &mut *guard;
            state.slots[index] = Some(found);
            while !state.done && state.next < state.slots.len() {
                let Some(found) = state.slots[state.next].take() else {
                    break;
                };
                state.next += 1;
                for group in found.groups {
                    state.outs.extend(group);
                    if state.outs.len() >= quota {
                        state.done = true;
                        stop.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }
    });
    if timed_out.load(Ordering::Relaxed) {
        return None;
    }
    Some(merge.into_inner().expect("merge lock").outs)
}

/// Runs a stage: the outputs of its first solutions, in the order the serial
/// search finds them, until there are at least `quota` of them (or the
/// lengths are used up, with fewer).
pub(crate) fn search<S: Stage, C: Collector>(
    stage: &S,
    starts: &[Start<S::Node>],
    collector: &C,
    quota: usize,
    threads: usize,
    deadline: Option<Instant>,
) -> Result<Vec<C::Out>, TimedOut> {
    struct Entry<N> {
        start: usize,
        prun: i32,
        est: i32,
        node: N,
    }
    let mut entries: Vec<Entry<S::Node>> = starts
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let prun = stage.prun(&s.node);
            Entry {
                start: i,
                prun,
                est: s.cost + prun,
                node: s.node,
            }
        })
        .collect();
    entries.sort_by_key(|e| (e.est, starts[e.start].cost));
    let stop = AtomicBool::new(false);
    let timed_out = AtomicBool::new(false);
    let none = stage.move_count();
    let mut all = Vec::new();
    for length in 0..MAX_LENGTH {
        let mut tasks: Vec<Task<S::Node>> = entries
            .iter()
            .take_while(|e| e.est <= length)
            .map(|e| Task {
                owner: starts[e.start].owner,
                node: e.node,
                depth: 0,
                maxl: length - starts[e.start].cost,
                last: none,
                prun: e.prun,
                prefix: [0; MAX_DEPTH],
            })
            .collect();
        if tasks.is_empty() {
            continue;
        }
        if threads > 1 {
            for _ in 0..3 {
                if tasks.len() >= threads * 16 {
                    break;
                }
                tasks = split(stage, &tasks);
            }
        }
        stop.store(false, Ordering::Relaxed);
        let need = quota - all.len();
        let flags = (&stop, &timed_out, deadline);
        let outs = run_tasks(stage, &tasks, collector, need, threads, flags).ok_or(TimedOut)?;
        let enough = outs.len() >= need;
        all.extend(outs);
        if enough {
            break;
        }
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A walk on the integers with steps of one; the goal is 0.
    struct Walk;

    impl Stage for Walk {
        type Node = i32;
        fn move_count(&self) -> usize {
            2
        }
        fn move_code(&self, m: usize) -> u8 {
            m as u8
        }
        fn skips(&self) -> &[u64] {
            &[0, 0, 0]
        }
        fn next_axis(&self) -> u64 {
            0
        }
        fn min_back_depth(&self) -> i32 {
            1
        }
        fn prun(&self, node: &i32) -> i32 {
            node.abs()
        }
        fn step(&self, node: &i32, mv: usize, _: i32) -> (i32, i32) {
            let next = if mv == 0 { node + 1 } else { node - 1 };
            (next, next.abs())
        }
    }

    struct Record;

    impl Collector for Record {
        type Out = (usize, Vec<u8>);
        fn collect(&self, owner: usize, moves: &[u8], out: &mut Vec<Self::Out>) {
            out.push((owner, moves.to_vec()));
        }
    }

    fn starts() -> Vec<Start<i32>> {
        [(3, 0), (1, 2), (-2, 1), (4, 0), (0, 5)]
            .into_iter()
            .enumerate()
            .map(|(owner, (node, cost))| Start { node, cost, owner })
            .collect()
    }

    #[test]
    fn solutions_arrive_in_serial_order_for_any_thread_count() {
        let serial = search(&Walk, &starts(), &Record, 40, 1, None).unwrap();
        assert!(serial.len() >= 40);
        for (owner, moves) in &serial {
            let start = &starts()[*owner];
            let end = moves
                .iter()
                .fold(start.node, |p, &m| if m == 0 { p + 1 } else { p - 1 });
            assert_eq!(end, 0);
        }
        // shorter total lengths come first
        let totals: Vec<usize> = serial
            .iter()
            .map(|(o, m)| starts()[*o].cost as usize + m.len())
            .collect();
        assert!(totals.windows(2).all(|w| w[0] <= w[1]));
        for threads in [2, 3, 8] {
            let parallel = search(&Walk, &starts(), &Record, 40, threads, None).unwrap();
            assert_eq!(parallel, serial, "{threads} threads");
        }
    }

    #[test]
    fn a_passed_deadline_stops_the_search() {
        let past = Instant::now() - std::time::Duration::from_secs(1);
        let result = search(&Walk, &starts(), &Record, usize::MAX / 2, 2, Some(past));
        assert_eq!(result.err(), Some(TimedOut));
    }
}
