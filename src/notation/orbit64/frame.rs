//! Origin: Rust re-implementation of `Orbit64.Frame` and the frame transforms
//! of `Orbit64.Net` (flix-orbit64, Apache-2.0, Werner Stein).
//!
//! An odd cube's fixed centres do not move relative to each other, so a token
//! records the whole-cube pose as one of 24 frames. Rank `r` is `x` turns,
//! then `z` turns, then `y` turns: 0..16 are `(r / 4, 0, r % 4)`, 16..20 add
//! one `z` turn and 20..24 three.

/// The number of whole-cube frames.
pub(super) const FRAME_COUNT: usize = 24;

type Sticker = (i32, i32, i32, i32, i32, i32);

fn turns(rank: usize) -> (usize, usize, usize) {
    let x = if rank < 16 { rank / 4 } else { 0 };
    let z = match rank {
        16..=19 => 1,
        20.. => 3,
        _ => 0,
    };
    (x, z, rank % 4)
}

/// A facelet as its cubie-grid position followed by its outward normal.
fn sticker_of(n: i32, index: usize) -> Sticker {
    let last = n - 1;
    let size = (n * n) as usize;
    let (face, row, col) = (
        index / size,
        (index / n as usize % n as usize) as i32,
        (index % n as usize) as i32,
    );
    match face {
        0 => (col, last, row, 0, 1, 0),
        1 => (last, last - row, last - col, 1, 0, 0),
        2 => (col, last - row, last, 0, 0, 1),
        3 => (col, 0, last - row, 0, -1, 0),
        4 => (0, last - row, col, -1, 0, 0),
        _ => (last - col, last - row, 0, 0, 0, -1),
    }
}

fn facelet_of(n: i32, (x, y, z, nx, ny, nz): Sticker) -> usize {
    let last = n - 1;
    let at = |face: i32, row: i32, col: i32| (face * n * n + row * n + col) as usize;
    if ny == 1 {
        at(0, z, x)
    } else if ny == -1 {
        at(3, last - z, x)
    } else if nz == 1 {
        at(2, last - y, x)
    } else if nz == -1 {
        at(5, last - y, last - x)
    } else if nx == 1 {
        at(1, last - y, last - z)
    } else {
        debug_assert_eq!(nx, -1);
        at(4, last - y, z)
    }
}

fn turn_x(n: i32, (x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (x, z, n - 1 - y, nx, nz, -ny)
}

fn turn_y(n: i32, (x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (n - 1 - z, y, x, -nz, ny, nx)
}

fn turn_z(n: i32, (x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (y, n - 1 - x, z, ny, -nx, nz)
}

fn repeat(
    n: i32,
    turn: fn(i32, Sticker) -> Sticker,
    mut sticker: Sticker,
    times: usize,
) -> Sticker {
    for _ in 0..times {
        sticker = turn(n, sticker);
    }
    sticker
}

/// The facelets of a cube in canonical pose, seen in `frame`.
pub(super) fn apply(n: usize, stickers: &[usize], frame: usize) -> Vec<usize> {
    let (x, z, y) = turns(frame);
    let n = n as i32;
    (0..stickers.len())
        .map(|index| {
            let mut sticker = sticker_of(n, index);
            sticker = repeat(n, turn_y, sticker, (4 - y) % 4);
            sticker = repeat(n, turn_z, sticker, (4 - z) % 4);
            sticker = repeat(n, turn_x, sticker, (4 - x) % 4);
            stickers[facelet_of(n, sticker)]
        })
        .collect()
}

/// Undoes [`apply`]: the canonical-pose facelets of a cube seen in `frame`.
pub(super) fn unapply(n: usize, stickers: &[usize], frame: usize) -> Vec<usize> {
    let (x, z, y) = turns(frame);
    let n = n as i32;
    (0..stickers.len())
        .map(|index| {
            let mut sticker = sticker_of(n, index);
            sticker = repeat(n, turn_x, sticker, x);
            sticker = repeat(n, turn_z, sticker, z);
            sticker = repeat(n, turn_y, sticker, y);
            stickers[facelet_of(n, sticker)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unapply_inverts_apply_for_every_frame_and_odd_size() {
        for n in [3usize, 5, 7] {
            let numbered: Vec<usize> = (0..6 * n * n).collect();
            for frame in 0..FRAME_COUNT {
                let seen = apply(n, &numbered, frame);
                assert_eq!(unapply(n, &seen, frame), numbered, "{n} frame {frame}");
            }
        }
    }

    #[test]
    fn the_24_frames_place_the_centres_differently() {
        let n = 3;
        let solved: Vec<usize> = (0..54).map(|i| i / 9).collect();
        let mut seen = std::collections::HashSet::new();
        for frame in 0..FRAME_COUNT {
            let centres: Vec<usize> = (0..6)
                .map(|f| apply(n, &solved, frame)[f * 9 + 4])
                .collect();
            assert!(seen.insert(centres), "frame {frame} repeats a pose");
        }
    }
}
