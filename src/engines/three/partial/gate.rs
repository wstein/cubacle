//! The gate of a table build: its own deadline, a progress report and a cancel.
//!
//! The builders call [`Gate::check`] once per chunk of work (some tens of
//! thousands of steps). The clock is `crate::puzzle::Instant`, which is the host
//! import `cubacle_host_now` in the WebAssembly build, and the observer is the
//! host's progress callback, so a build can be stopped without threads.

use crate::puzzle::Instant;

/// Why a build stopped; the number is the progress in thousandths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stop {
    /// The build deadline passed.
    Budget(u16),
    /// The observer asked to stop.
    Cancelled(u16),
}

/// Receives the progress in thousandths of the whole build; `false` cancels.
pub(super) type Observer<'a> = &'a mut dyn FnMut(u16) -> bool;

pub(super) struct Gate<'a> {
    deadline: Option<Instant>,
    observer: Option<Observer<'a>>,
    index: u32,
    count: u32,
    /// The progress last known, in thousandths of the whole build.
    last: Option<u16>,
}

impl<'a> Gate<'a> {
    pub(super) fn new(deadline: Option<Instant>, observer: Option<Observer<'a>>) -> Gate<'a> {
        Gate {
            deadline,
            observer,
            index: 0,
            count: 1,
            last: None,
        }
    }

    /// A gate that never stops anything.
    pub(super) fn none() -> Gate<'static> {
        Gate::new(None, None)
    }

    /// Starts stage `index` of `count` equal stages.
    pub(super) fn stage(&mut self, index: usize, count: usize) {
        self.index = index as u32;
        self.count = (count as u32).max(1);
    }

    /// Reports `local` thousandths of the current stage and says whether to go on.
    pub(super) fn check(&mut self, local: u16) -> Result<(), Stop> {
        let overall = ((self.index * 1000 + u32::from(local.min(1000))) / self.count) as u16;
        let overall = overall.max(self.last.unwrap_or(0));
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            self.last = Some(overall);
            return Err(Stop::Budget(overall));
        }
        if self.last.is_none_or(|last| overall > last) {
            self.last = Some(overall);
            if let Some(observer) = self.observer.as_mut() {
                if !observer(overall) {
                    return Err(Stop::Cancelled(overall));
                }
            }
        }
        Ok(())
    }

    /// Reports the end of the build.
    pub(super) fn finish(&mut self) {
        if self.last != Some(1000) {
            self.last = Some(1000);
            if let Some(observer) = self.observer.as_mut() {
                observer(1000);
            }
        }
    }
}
