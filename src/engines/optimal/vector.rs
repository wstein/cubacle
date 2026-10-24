//! The move step, `Cube` after move `m`: the one place a vector version may
//! replace the portable loop. A vector version keeps this signature: a
//! `Cube`, the tables and a move number in, a `Cube` out.

use super::model::{Cube, MoveTables};

/// The cube after move `m`.
#[inline]
pub(crate) fn turn(cube: &Cube, tables: &MoveTables, m: usize) -> Cube {
    cube.turned_portable(tables, m)
}
