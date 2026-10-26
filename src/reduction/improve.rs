//! What `--improve` does with a time budget on the reduction engines: how
//! many reductions to finish, and how hard to search each 3x3x3 tail.
//!
//! New code.
//!
//! The engine keeps the first N results of its last reduction phase, gives
//! each its own 3x3x3 tail with a high probe minimum, and answers with the
//! shortest total (the earliest on a tie). N and the probe minimum are
//! functions of the budget alone, so the work is counted, not timed. Only
//! the clock can cut it short: a run that finishes every tail inside the
//! budget gives the same answer on every machine and thread count.

use std::time::Duration;

use crate::puzzle::SolveOptions;

/// The budget `--improve` uses when none is given.
pub(crate) const DEFAULT_BUDGET: Duration = Duration::from_millis(1000);

/// Milliseconds of budget per extra reduction kept.
const MS_PER_CANDIDATE: u64 = 30;
/// The most reductions kept.
const MAX_CANDIDATES: usize = 48;
/// Probes of every tail at the start of the scale, and the budget in
/// milliseconds that adds one more.
const BASE_PROBES: u64 = 500;
const MS_PER_PROBE: u64 = 2;
/// Share of the budget the collection of reductions may take.
const COLLECT_SHARE: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Plan {
    /// Reductions to finish.
    pub candidates: usize,
    /// The probe minimum of each 3x3x3 tail.
    pub probes: u64,
    /// How long the collection of the extra reductions may take after the
    /// start; past it the engine goes on with the first one alone.
    pub collect: Duration,
}

pub(crate) fn plan(budget: Duration) -> Plan {
    let ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX);
    Plan {
        candidates: usize::try_from(ms / MS_PER_CANDIDATE)
            .unwrap_or(usize::MAX)
            .clamp(1, MAX_CANDIDATES),
        probes: BASE_PROBES + ms / MS_PER_PROBE,
        collect: budget * (COLLECT_SHARE - 1) / COLLECT_SHARE,
    }
}

/// The plan of a reduction-only solve (a stage of a bigger cube) with a probe
/// count: `--probes N` keeps N reductions and answers with the shortest. It
/// counts work, not time, so the answer is the same on every machine. Without
/// a probe count, or for a full solve, there is no plan.
pub(crate) fn counted(options: &SolveOptions) -> Option<Plan> {
    let probes = options.min_probes.filter(|_| options.reduce_only)?;
    Some(Plan {
        candidates: usize::try_from(probes)
            .unwrap_or(usize::MAX)
            .clamp(1, MAX_CANDIDATES),
        probes: BASE_PROBES,
        collect: Duration::from_secs(365 * 24 * 3600),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plan_grows_with_the_budget() {
        let small = plan(Duration::from_millis(100));
        let large = plan(Duration::from_millis(3000));
        assert_eq!(small.candidates, 3);
        assert!(large.candidates > small.candidates);
        assert!(large.probes > small.probes);
        assert_eq!(plan(Duration::ZERO).candidates, 1);
    }

    #[test]
    fn a_counted_plan_needs_a_reduction_only_solve_with_probes() {
        let probes = |n| SolveOptions::default().with_min_probes(n);
        assert_eq!(counted(&probes(5)), None);
        assert_eq!(
            counted(&probes(5).with_reduce_only(true))
                .unwrap()
                .candidates,
            5
        );
        assert_eq!(
            counted(&SolveOptions::default().with_reduce_only(true)),
            None
        );
        let many = counted(&probes(9999).with_reduce_only(true)).unwrap();
        assert_eq!(many.candidates, MAX_CANDIDATES);
    }
}
