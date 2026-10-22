//! Building blocks of the reduction engines (5x5x5 now, 4x4x4 later).
//!
//! Crate-private on purpose: engines use these pieces, applications do not.
//! The pieces have no cube in them. `comb` is the combinatorics of piece
//! coordinates, `nibbles` the 4-bit pruning table, `bfs` its breadth-first
//! builder (`hashed` for spaces too large to enumerate), `cache` its on-disk form, `par` the thread helpers, `improve` the schedule of `--improve` and `ida` the
//! generic iterative-deepening search over a stage of a reduction.
//!
//! Dependency rule: `puzzle <- notation <- reduction <- engines`.

pub(crate) mod bfs;
pub(crate) mod cache;
pub(crate) mod comb;
pub(crate) mod hashed;
pub(crate) mod ida;
pub(crate) mod improve;
pub(crate) mod nibbles;
pub(crate) mod par;
