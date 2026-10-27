//! Thread helpers: the configured thread count and a parallel for.

use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub(crate) use crate::puzzle::threads;

/// Runs `f(worker)` for `worker` in `0..threads`, worker 0 on the calling
/// thread, and returns when all are done. A panic in a worker is re-raised.
pub(crate) fn run(threads: usize, f: impl Fn(usize) + Sync) {
    if threads <= 1 {
        f(0);
        return;
    }
    std::thread::scope(|scope| {
        for worker in 1..threads {
            let f = &f;
            scope.spawn(move || f(worker));
        }
        f(0);
    });
}

/// Splits `0..items` into chunks of `chunk` and hands them out dynamically
/// to `threads` workers, each call getting one range.
pub(crate) fn for_chunks(
    threads: usize,
    items: usize,
    chunk: usize,
    f: impl Fn(Range<usize>) + Sync,
) {
    let next = AtomicUsize::new(0);
    run(threads, |_| loop {
        let start = next.fetch_add(chunk, Ordering::Relaxed);
        if start >= items {
            return;
        }
        f(start..(start + chunk).min(items));
    });
}

/// Runs tasks `0..tasks` on `threads` workers, handing them out in order, and
/// returns the first `need` results of the tasks taken in task order, as a
/// serial run would: the results of task 0, then those of task 1, and so on.
///
/// `work(task, cancelled)` returns the results of one task. `cancelled()`
/// turns true once the tasks before this one have found `need` results in
/// total, so a task that is still running may stop and give any answer: it
/// is not used. The result does not depend on the number of threads.
pub(crate) fn first_results<T: Send>(
    threads: usize,
    tasks: usize,
    need: usize,
    work: impl Fn(usize, &dyn Fn() -> bool) -> Vec<T> + Sync,
) -> Vec<T> {
    struct Slots<T> {
        done: Vec<Option<Vec<T>>>,
        /// Tasks before this one are all done.
        prefix: usize,
        count: usize,
    }
    let next = AtomicUsize::new(0);
    let cutoff = AtomicUsize::new(usize::MAX);
    let slots = Mutex::new(Slots {
        done: (0..tasks).map(|_| None).collect(),
        prefix: 0,
        count: 0,
    });
    run(threads.min(tasks.max(1)).max(1), |_| loop {
        let task = next.fetch_add(1, Ordering::Relaxed);
        if task >= tasks || task >= cutoff.load(Ordering::Relaxed) {
            return;
        }
        let cancelled = || task >= cutoff.load(Ordering::Relaxed);
        let results = work(task, &cancelled);
        let mut slots = slots.lock().expect("no panic while holding the lock");
        slots.done[task] = Some(results);
        while slots.prefix < tasks {
            let at = slots.prefix;
            let Some(done) = &slots.done[at] else { break };
            let len = done.len();
            slots.count += len;
            slots.prefix += 1;
            if slots.count >= need {
                cutoff.fetch_min(slots.prefix, Ordering::Relaxed);
                break;
            }
        }
    });
    let slots = slots.into_inner().expect("no panic while holding the lock");
    let mut out = Vec::new();
    for done in slots.done {
        let Some(done) = done else { break };
        out.extend(done);
        if out.len() >= need {
            break;
        }
    }
    out.truncate(need);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn the_first_results_are_those_of_a_serial_run() {
        // Task k gives k % 4 results, each k * 100 + n.
        let run_task = |k: usize, _: &dyn Fn() -> bool| (0..k % 4).map(|n| k * 100 + n).collect();
        let serial: Vec<usize> = (0..40)
            .flat_map(|k| (0..k % 4).map(move |n| k * 100 + n))
            .collect();
        for threads in [1, 2, 7] {
            for need in [1, 5, 30, 1000] {
                let got = first_results(threads, 40, need, run_task);
                assert_eq!(
                    got,
                    serial[..need.min(serial.len())],
                    "{threads} threads, {need}"
                );
            }
        }
    }

    #[test]
    fn tasks_after_the_cut_are_cancelled() {
        let started = AtomicUsize::new(0);
        let got = first_results(1, 100, 1, |k, cancelled| {
            started.fetch_add(1, Ordering::Relaxed);
            assert!(!cancelled());
            vec![k]
        });
        assert_eq!(got, vec![0]);
        assert_eq!(started.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn every_item_is_visited_once() {
        for threads in [1, 3, 8] {
            let sum = AtomicU64::new(0);
            for_chunks(threads, 1000, 7, |range| {
                sum.fetch_add(range.map(|i| i as u64).sum(), Ordering::Relaxed);
            });
            assert_eq!(sum.load(Ordering::Relaxed), 999 * 1000 / 2);
        }
    }
}
