//! The move step, `Cube` after move `m`, with a vector version.
//!
//! On aarch64 NEON is part of the baseline, so the step is chosen at compile
//! time: the byte-table lookup `vqtbl` moves all 12 edges (or 8 corners) in
//! one instruction. Every other target runs the portable loop in `model`.
//! The one `unsafe` block of the engine is here; a test compares the two on
//! random cubes. An x86 version (AVX2 `vpshufb`, chosen at run time) would
//! replace `turn` and keep this signature: a `Cube`, the tables, a move
//! number in, a `Cube` out.

use super::model::{Cube, MoveTables};

/// The cube after move `m`.
#[inline]
pub(crate) fn turn(cube: &Cube, tables: &MoveTables, m: usize) -> Cube {
    #[cfg(target_arch = "aarch64")]
    {
        neon::turn(cube, tables, m)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        cube.turned_portable(tables, m)
    }
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use std::arch::aarch64::{
        uint8x16x2_t, vaddq_u8, vandq_u8, vcltq_u8, vdupq_n_u8, veorq_u8, vld1q_u8, vqtbl1q_u8,
        vqtbl2q_u8, vst1q_u8, vsubq_u8,
    };

    use crate::engines::optimal::model::{Cube, MoveTables};

    /// 255 in the lanes of pieces, 0 in the lanes past them.
    const EDGE_LANES: [u8; 16] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
    ];
    const CORNER_LANES: [u8; 16] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    /// The twist (value modulo 3) of a corner byte.
    const TWIST: [u8; 32] = [
        0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    #[inline]
    pub(super) fn turn(cube: &Cube, tables: &MoveTables, m: usize) -> Cube {
        let mut out = *cube;
        // SAFETY: NEON is part of the aarch64 baseline. Every load and store
        // is of 16 bytes (or two adjacent blocks of 16) from or into arrays
        // of that size or larger: `[u8; 16]` fields and `[u8; 32]` tables.
        unsafe {
            let edge_table = &tables.edges[m];
            let table = uint8x16x2_t(
                vld1q_u8(edge_table.as_ptr()),
                vld1q_u8(edge_table.as_ptr().add(16)),
            );
            let edges = vqtbl2q_u8(table, vld1q_u8(cube.edges.as_ptr()));
            vst1q_u8(
                out.edges.as_mut_ptr(),
                vandq_u8(edges, vld1q_u8(EDGE_LANES.as_ptr())),
            );

            let corner_table = &tables.corners[m];
            let table = uint8x16x2_t(
                vld1q_u8(corner_table.as_ptr()),
                vld1q_u8(corner_table.as_ptr().add(16)),
            );
            let corners = vqtbl2q_u8(table, vld1q_u8(cube.corners.as_ptr()));
            vst1q_u8(
                out.corners.as_mut_ptr(),
                vandq_u8(corners, vld1q_u8(CORNER_LANES.as_ptr())),
            );

            // The inverse cube: places take their piece from another place.
            let moved = vqtbl1q_u8(
                vld1q_u8(cube.inv_edges.as_ptr()),
                vld1q_u8(tables.from_edges[m].as_ptr()),
            );
            let flipped = veorq_u8(moved, vld1q_u8(tables.flip_edges[m].as_ptr()));
            vst1q_u8(out.inv_edges.as_mut_ptr(), flipped);

            let moved = vqtbl1q_u8(
                vld1q_u8(cube.inv_corners.as_ptr()),
                vld1q_u8(tables.from_corners[m].as_ptr()),
            );
            let turn = vld1q_u8(tables.turn_corners[m].as_ptr());
            let twist_table =
                uint8x16x2_t(vld1q_u8(TWIST.as_ptr()), vld1q_u8(TWIST.as_ptr().add(16)));
            let twist = vqtbl2q_u8(twist_table, moved);
            let wrap = vandq_u8(vcltq_u8(twist, turn), vdupq_n_u8(3));
            let untwisted = vaddq_u8(vsubq_u8(moved, turn), wrap);
            vst1q_u8(out.inv_corners.as_mut_ptr(), untwisted);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use super::*;
    use crate::engines::optimal::model::{tables, MOVES};

    #[test]
    fn the_vector_step_equals_the_portable_one() {
        let tables = tables();
        let mut rng = rand::rng();
        for _ in 0..300 {
            let mut cube = Cube::SOLVED;
            for _ in 0..40 {
                let m = rng.random_range(0..MOVES);
                let plain = cube.turned_portable(tables, m);
                assert_eq!(turn(&cube, tables, m), plain, "move {m}");
                cube = plain;
            }
        }
    }
}
