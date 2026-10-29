//! What an exact search leaves behind for a certificate: the bound at the
//! start, the work per completed depth and a digest of every table.

/// The record of one exact search, for a certificate that a second program can
/// check by recomputing the state, the moves and the bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRecord {
    /// The lower bound at the start: the largest entry of the tables there.
    pub start_bound: usize,
    /// For every depth searched out without an answer: the depth and the nodes
    /// of that iteration. A proven answer of `n` moves has the depths
    /// `start_bound..n` here, then its own depth in `found`.
    pub completed: Vec<(usize, u64)>,
    /// The depth of the answer and the nodes of its iteration, when found.
    pub found: Option<(usize, u64)>,
    /// The tables used: a name, the cells, and an FNV-1a 64 digest of the contents.
    pub tables: Vec<TableDigest>,
    /// The number of views of the goal (24 for any orientation, else 1 or more).
    pub views: usize,
}

/// One table of the bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDigest {
    pub name: String,
    pub cells: usize,
    pub fnv1a64: u64,
}

/// FNV-1a over bytes, 64 bits.
pub(super) fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}
