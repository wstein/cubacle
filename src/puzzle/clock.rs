//! The clock the solvers read for their time budgets.
//!
//! On every target but the browser's `wasm32-unknown-unknown`, [`Instant`] is
//! `std::time::Instant`. That target has no clock in `std`: `Instant::now`
//! panics there. On it, [`Instant`] reads the host import
//! `cubacle_host_now` (module `env`, milliseconds as an `f64`, for example
//! `performance.now()`), and the arithmetic below is the same as the standard
//! type's for the calls the solvers make.

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(crate) use std::time::Instant;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) use host::Instant;

/// An instant on a clock the host reads, as a duration since the host's origin.
#[cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]
mod host {
    use std::ops::{Add, AddAssign, Sub};
    use std::time::Duration;

    #[link(wasm_import_module = "env")]
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    extern "C" {
        fn cubacle_host_now() -> f64;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(Duration);

    impl Instant {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        pub fn now() -> Instant {
            // SAFETY: the host import takes no arguments and returns a number.
            Instant::at_millis(unsafe { cubacle_host_now() })
        }

        /// The instant `millis` milliseconds after the host's origin; a negative
        /// or non-finite reading counts as the origin.
        pub fn at_millis(millis: f64) -> Instant {
            if millis.is_finite() && millis > 0.0 {
                Instant(Duration::from_secs_f64(millis / 1000.0))
            } else {
                Instant(Duration::ZERO)
            }
        }

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        pub fn elapsed(&self) -> Duration {
            Instant::now().duration_since(*self)
        }

        pub fn duration_since(&self, earlier: Instant) -> Duration {
            self.0.saturating_sub(earlier.0)
        }

        pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
            self.0.checked_add(duration).map(Instant)
        }
    }

    impl Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, duration: Duration) -> Instant {
            Instant(self.0 + duration)
        }
    }

    impl AddAssign<Duration> for Instant {
        fn add_assign(&mut self, duration: Duration) {
            self.0 += duration;
        }
    }

    impl Sub<Instant> for Instant {
        type Output = Duration;
        fn sub(self, earlier: Instant) -> Duration {
            self.duration_since(earlier)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Instant;
        use std::time::Duration;

        #[test]
        fn arithmetic_matches_the_standard_instant() {
            let start = Instant::at_millis(1500.0);
            let later = start + Duration::from_millis(250);
            assert_eq!(later.duration_since(start), Duration::from_millis(250));
            assert_eq!(later - start, Duration::from_millis(250));
            assert_eq!(start.duration_since(later), Duration::ZERO);
            assert!(later > start);
            assert_eq!(
                start.checked_add(Duration::from_secs(1)),
                Some(Instant::at_millis(2500.0))
            );
            let mut moved = start;
            moved += Duration::from_millis(500);
            assert_eq!(moved, Instant::at_millis(2000.0));
        }

        #[test]
        fn a_bad_reading_is_the_origin() {
            assert_eq!(Instant::at_millis(-1.0), Instant::at_millis(0.0));
            assert_eq!(Instant::at_millis(f64::NAN), Instant::at_millis(0.0));
            assert_eq!(Instant::at_millis(f64::INFINITY), Instant::at_millis(0.0));
        }
    }
}
