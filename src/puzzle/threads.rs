//! The number of worker threads the parallel code uses.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_thread_setting_is_capped_and_checked() {
        assert_eq!(from_setting(Some("3")), 3);
        assert_eq!(from_setting(Some("1000000")), MAX_THREADS);
        let default = from_setting(None);
        assert!(default >= 1);
        assert_eq!(from_setting(Some("0")), default);
        assert_eq!(from_setting(Some("many")), default);
    }
}
