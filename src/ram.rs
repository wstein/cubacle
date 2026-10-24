//! The memory of this machine, read at run time.
//!
//! Table sizes depend on what the host really has, never on the platform.
//! The reading is done by the `sysinfo` crate (macOS, Linux and Windows) with
//! only its `system` feature; nothing is run and no file is parsed here.

use sysinfo::System;

/// Memory in bytes, as the operating system reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ram {
    pub total: u64,
    /// What can be used now without swapping, where the system says so.
    pub available: Option<u64>,
}

/// The memory of this machine, or `None` when the system gives no figure.
pub(crate) fn detect() -> Option<Ram> {
    let mut system = System::new();
    system.refresh_memory();
    let total = system.total_memory();
    let available = system.available_memory();
    (total > 0).then_some(Ram {
        total,
        available: (available > 0).then_some(available),
    })
}

/// Megabytes (10^6 bytes) the tables may use by default: half of the available
/// memory (of the total when the system does not say), never above 16000.
pub(crate) fn default_budget_mb(ram: &Ram) -> usize {
    let basis = ram.available.unwrap_or(ram.total).min(ram.total);
    ((basis / 2 / 1_000_000) as usize).min(16_000)
}
