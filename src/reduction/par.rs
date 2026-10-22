//! Thread helpers: the configured thread count and a parallel for.

use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// The number of worker threads: `CUBACLE_THREADS` if it is a positive
/// number, else the available parallelism.
pub(crate) fn threads() -> usize {
    std::env::var("CUBACLE_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count > 0)
        .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
        .unwrap_or(1)
}

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

/// Hands the items out dynamically to `threads` workers.
pub(crate) fn for_each<T: Send>(threads: usize, items: Vec<T>, f: impl Fn(T) + Sync) {
    let queue = Mutex::new(items.into_iter());
    run(threads, |_| loop {
        let item = queue.lock().expect("work queue").next();
        match item {
            Some(item) => f(item),
            None => return,
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

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
