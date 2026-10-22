//! Thread helpers: the configured thread count and a parallel for.

use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The most threads `CUBACLE_THREADS` can ask for.
const MAX_THREADS: usize = 256;

/// The number of worker threads: `CUBACLE_THREADS` if it is a positive
/// number (at most 256), else the available parallelism.
pub(crate) fn threads() -> usize {
    from_setting(std::env::var("CUBACLE_THREADS").ok().as_deref())
}

fn from_setting(value: Option<&str>) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count > 0)
        .map(|count| count.min(MAX_THREADS))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn the_thread_setting_is_capped_and_checked() {
        assert_eq!(from_setting(Some("3")), 3);
        assert_eq!(from_setting(Some("1000000")), MAX_THREADS);
        let default = from_setting(None);
        assert!(default >= 1);
        assert_eq!(from_setting(Some("0")), default);
        assert_eq!(from_setting(Some("many")), default);
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
