//! Origin: Rust re-implementation of the geometry of `Orbit64.V2.Facelets`
//! (flix-orbit64, Apache-2.0, Werner Stein), following `FORMAT.md`.
//!
//! A whole-cube rotation is applied to a facelet string through the position
//! and outward normal of every facelet, not through V1's `x`, `z`, `y`
//! composition of `frame.rs`: frame `r` is the rotation that moves the
//! physical face `FRAME_FACES[r][i]` into slot `i` of `U R F D L B`. The two
//! agree; a test checks it.

/// A position (on the doubled grid) or a direction.
type Vec3 = [i32; 3];

/// A rotation as the images of the unit vectors `x`, `y` and `z`.
pub(super) type Turn = [Vec3; 3];

/// The number of right-handed whole-cube frames.
pub(super) const FRAME_COUNT: usize = 24;

/// The physical faces (`URFDLB` = `0..6`) in the slots `U R F D L B` of each
/// frame: `Orbit64.Frame.faces`, the wire order of V1.
#[rustfmt::skip]
pub(super) const FRAME_FACES: [[usize; 6]; FRAME_COUNT] = [
    [0, 1, 2, 3, 4, 5], [0, 5, 1, 3, 2, 4], [0, 4, 5, 3, 1, 2], [0, 2, 4, 3, 5, 1],
    [2, 1, 3, 5, 4, 0], [2, 0, 1, 5, 3, 4], [2, 4, 0, 5, 1, 3], [2, 3, 4, 5, 0, 1],
    [3, 1, 5, 0, 4, 2], [3, 2, 1, 0, 5, 4], [3, 4, 2, 0, 1, 5], [3, 5, 4, 0, 2, 1],
    [5, 1, 0, 2, 4, 3], [5, 3, 1, 2, 0, 4], [5, 4, 3, 2, 1, 0], [5, 0, 4, 2, 3, 1],
    [4, 0, 2, 1, 3, 5], [4, 5, 0, 1, 2, 3], [4, 3, 5, 1, 0, 2], [4, 2, 3, 1, 5, 0],
    [1, 3, 2, 4, 0, 5], [1, 5, 3, 4, 2, 0], [1, 0, 5, 4, 3, 2], [1, 2, 0, 4, 5, 3],
];

/// The outward normal of each face, `URFDLB`.
const NORMALS: [Vec3; 6] = [
    [0, 1, 0],
    [1, 0, 0],
    [0, 0, 1],
    [0, -1, 0],
    [-1, 0, 0],
    [0, 0, -1],
];

/// Position (doubled grid, `-m..=m` with `m = n - 1`) and outward normal of a
/// facelet, in the layout `face * n^2 + row * n + col`.
fn geometry(n: i32, index: usize) -> (Vec3, Vec3) {
    let m = n - 1;
    let size = (n * n) as usize;
    let face = index / size;
    let a = 2 * (index % n as usize) as i32 - m;
    let b = 2 * (index / n as usize % n as usize) as i32 - m;
    let position = match face {
        0 => [a, m, b],
        1 => [m, -b, -a],
        2 => [a, -b, m],
        3 => [a, -m, -b],
        4 => [-m, -b, a],
        _ => [-a, -b, -m],
    };
    (position, NORMALS[face])
}

/// The inverse of [`geometry`].
fn facelet_at(n: i32, [x, y, z]: Vec3, normal: Vec3) -> usize {
    let m = n - 1;
    let face = NORMALS
        .iter()
        .position(|&candidate| candidate == normal)
        .expect("a facelet normal is one of the six face normals");
    let (a, b) = match face {
        0 => (x, z),
        1 => (-z, -y),
        2 => (x, -y),
        3 => (x, -z),
        4 => (z, -y),
        _ => (-x, -y),
    };
    face * (n * n) as usize + ((b + m) / 2 * n + (a + m) / 2) as usize
}

fn dot(a: Vec3, b: Vec3) -> i32 {
    a.iter().zip(b).map(|(p, q)| p * q).sum()
}

/// The rotation of a frame: it moves physical face `FRAME_FACES[frame][i]`
/// onto slot `i`.
fn turn_of(frame: usize) -> Turn {
    let faces = &FRAME_FACES[frame];
    let slot_of = |face: usize| {
        NORMALS[faces
            .iter()
            .position(|&f| f == face)
            .expect("a frame lists all six faces")]
    };
    [slot_of(1), slot_of(0), slot_of(2)]
}

fn forward([ex, ey, ez]: Turn, [x, y, z]: Vec3) -> Vec3 {
    std::array::from_fn(|i| x * ex[i] + y * ey[i] + z * ez[i])
}

/// The inverse of a rotation is its transpose.
fn backward([ex, ey, ez]: Turn, v: Vec3) -> Vec3 {
    [dot(v, ex), dot(v, ey), dot(v, ez)]
}

/// The stickers after the cube is rotated by `frame`: each sticker moves
/// with its color.
pub(super) fn rotate(n: usize, frame: usize, stickers: &[usize]) -> Vec<usize> {
    let (n, turn) = (n as i32, turn_of(frame));
    (0..stickers.len())
        .map(|i| {
            let (position, normal) = geometry(n, i);
            stickers[facelet_at(n, backward(turn, position), backward(turn, normal))]
        })
        .collect()
}

/// The stickers of the cube rotated back: the inverse of [`rotate`].
pub(super) fn unrotate(n: usize, frame: usize, stickers: &[usize]) -> Vec<usize> {
    let (n, turn) = (n as i32, turn_of(frame));
    (0..stickers.len())
        .map(|i| {
            let (position, normal) = geometry(n, i);
            stickers[facelet_at(n, forward(turn, position), forward(turn, normal))]
        })
        .collect()
}

// ------------------------------------------------- fixed center rotations

fn cross([ax, ay, az]: Vec3, [bx, by, bz]: Vec3) -> Vec3 {
    [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx]
}

/// The reference direction of a face: toward row 0 of the face, so toward `B`
/// on `U` and toward `F` on `D`.
fn reference(face: usize) -> Vec3 {
    match face {
        0 => [0, 0, -1],
        3 => [0, 0, 1],
        _ => [0, 1, 0],
    }
}

/// A direction on a face turned `k` clockwise quarter turns, seen from outside.
fn spin(normal: Vec3, direction: Vec3, k: usize) -> Vec3 {
    (0..k).fold(direction, |current, _| cross(current, normal))
}

/// The clockwise quarter turns from the reference direction of `face` to
/// `direction`, which lies in the plane of the face.
fn quarter_turns(face: usize, direction: Vec3) -> usize {
    (0..4)
        .find(|&k| spin(NORMALS[face], reference(face), k) == direction)
        .expect("a direction in the plane of a face is a quarter turn from its reference")
}

fn face_of_normal(normal: Vec3) -> usize {
    NORMALS
        .iter()
        .position(|&candidate| candidate == normal)
        .expect("a turned face normal is one of the six face normals")
}

/// `permuted` is `before` with its facelets moved by `movement`; the fixed
/// centers also carry their rotation `value mod 4` to the face they land on,
/// measured there against that face's reference direction.
fn turn_fixed(
    n: usize,
    movement: impl Fn(Vec3) -> Vec3,
    before: &[usize],
    mut permuted: Vec<usize>,
) -> Vec<usize> {
    let fixed = |face: usize| face * n * n + (n * n - 1) / 2;
    for face in 0..6 {
        let value = before[fixed(face)];
        let own = spin(NORMALS[face], reference(face), value % 4);
        let landed = face_of_normal(movement(NORMALS[face]));
        permuted[fixed(landed)] = value / 4 * 4 + quarter_turns(landed, movement(own));
    }
    permuted
}

/// The picture after the cube is rotated by `frame`: stickers move to their
/// facelets and the six fixed centers, whose value is `4 p + r`, keep their
/// piece `p` and have `r` measured again at the facelet they land on. An odd
/// cube only.
pub(super) fn rotate_picture(n: usize, frame: usize, symbols: &[usize]) -> Vec<usize> {
    let turn = turn_of(frame);
    turn_fixed(n, |v| forward(turn, v), symbols, rotate(n, frame, symbols))
}

/// The picture of the cube rotated back: the inverse of [`rotate_picture`].
pub(super) fn unrotate_picture(n: usize, frame: usize, symbols: &[usize]) -> Vec<usize> {
    let turn = turn_of(frame);
    turn_fixed(
        n,
        |v| backward(turn, v),
        symbols,
        unrotate(n, frame, symbols),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::orbit64::frame;

    #[test]
    fn facelet_at_inverts_geometry() {
        for n in 2..=15 {
            for index in 0..6 * n * n {
                let (position, normal) = geometry(n as i32, index);
                assert_eq!(facelet_at(n as i32, position, normal), index, "{n}");
            }
        }
    }

    #[test]
    fn unrotate_inverts_rotate_for_every_frame() {
        for n in [2usize, 3, 4, 7, 8] {
            let numbered: Vec<usize> = (0..6 * n * n).collect();
            for r in 0..FRAME_COUNT {
                assert_eq!(
                    unrotate(n, r, &rotate(n, r, &numbered)),
                    numbered,
                    "{n} {r}"
                );
            }
        }
    }

    #[test]
    fn unrotate_picture_inverts_rotate_picture_for_every_frame_and_rotation() {
        for n in [3usize, 5, 9] {
            for shift in 0..4 {
                let mut symbols: Vec<usize> = (0..6 * n * n).map(|i| i % 24).collect();
                for face in 0..6 {
                    symbols[face * n * n + (n * n - 1) / 2] = 4 * face + (face + shift) % 4;
                }
                for r in 0..FRAME_COUNT {
                    let turned = rotate_picture(n, r, &symbols);
                    assert_eq!(unrotate_picture(n, r, &turned), symbols, "{n} {shift} {r}");
                    // The stickers move as in `rotate`; only the fixed centers differ.
                    let plain = rotate(n, r, &symbols);
                    let differs: Vec<usize> = (0..symbols.len())
                        .filter(|&i| turned[i] != plain[i])
                        .collect();
                    assert!(differs.iter().all(|i| i % (n * n) == (n * n - 1) / 2));
                    assert!(differs.iter().all(|&i| turned[i] / 4 == plain[i] / 4));
                }
            }
        }
    }

    #[test]
    fn the_frame_faces_are_the_v1_frames() {
        // V1 applies x, z, y turns; the geometry here must give the same pictures.
        for n in [3usize, 5, 7] {
            let solved: Vec<usize> = (0..6 * n * n).map(|i| i / (n * n)).collect();
            let numbered: Vec<usize> = (0..6 * n * n).collect();
            for (r, faces) in FRAME_FACES.iter().enumerate() {
                let v1 = frame::apply(n, &numbered, r);
                assert_eq!(rotate(n, r, &numbered), v1, "{n} frame {r}");
                let centers: Vec<usize> = (0..6)
                    .map(|slot| rotate(n, r, &solved)[slot * n * n + (n * n - 1) / 2])
                    .collect();
                assert_eq!(centers, faces[..], "{n} frame {r}");
            }
        }
    }

    #[test]
    fn the_24_frames_are_distinct_and_right_handed() {
        let det = |r: usize| {
            let [x, y, z] = turn_of(r);
            let cross = [
                y[1] * z[2] - y[2] * z[1],
                y[2] * z[0] - y[0] * z[2],
                y[0] * z[1] - y[1] * z[0],
            ];
            dot(x, cross)
        };
        for (r, faces) in FRAME_FACES.iter().enumerate() {
            assert_eq!(det(r), 1, "frame {r}");
            for earlier in &FRAME_FACES[..r] {
                assert_ne!(faces, earlier);
            }
        }
    }
}
