/// The reason a solver invocation returned its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveOutcome {
    /// A solution was found without reaching the hard deadline.
    Solved,
    /// The hard deadline was reached after a valid solution had been found.
    DeadlineWithSolution,
    /// The hard deadline was reached before any valid solution was found.
    DeadlineNoSolution,
    /// The input was invalid or no solution exists within the requested move limit.
    Unsolvable,
}

/// The result and search statistics for one solver invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveStats {
    /// The solution move sequence, or an `Error <code>` result.
    pub solution: String,
    /// Number of phase-two search probes performed while solving.
    pub probes: u64,
    /// Number of phase-one search nodes expanded while solving.
    pub phase1_nodes: u64,
    /// Number of phase-one children rejected by the slice-twist pruning estimate.
    pub phase1_rejected_slice_twist: u64,
    /// Number of phase-one children rejected by the slice-flip pruning estimate after passing slice-twist.
    pub phase1_rejected_slice_flip: u64,
    /// Number of phase-one axis branches stopped by a slice-twist estimate above the remaining depth.
    pub phase1_slice_twist_breaks: u64,
    /// Number of phase-one children skipped when slice-twist equals the remaining depth.
    pub phase1_slice_twist_equals: u64,
    /// Number of phase-one axis branches stopped by a slice-flip estimate above the remaining depth.
    pub phase1_slice_flip_breaks: u64,
    /// Number of phase-one children skipped when slice-flip equals the remaining depth.
    pub phase1_slice_flip_equals: u64,
    /// Number of phase-one children that passed pruning and recursed.
    pub phase1_children_recurred: u64,
    /// Whether the hard deadline expired before any solution was found.
    ///
    /// This remains `false` when a solution is returned at the deadline; inspect
    /// [`SolveStats::outcome`] to distinguish that case from an ordinary solve.
    pub timed_out: bool,
    /// The reason the solver returned.
    pub outcome: SolveOutcome,
}
