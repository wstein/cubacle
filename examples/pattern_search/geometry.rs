//! Sticker geometry for the search: every move is a table `src` with
//! `new[i] = old[src[i]]`, built from the same layout as `Facelets::apply`
//! (the search replays its result with the library before printing it).

use cubacle::puzzle::{Face, LayerMove, Turn};

type Vec3 = [i32; 3];

fn locate(n: usize, index: usize) -> (Vec3, Vec3) {
    let m = n as i32 - 1;
    let (face, row, col) = (index / (n * n), (index % (n * n)) / n, index % n);
    let (a, b) = (2 * col as i32 - m, 2 * row as i32 - m);
    match Face::ALL[face] {
        Face::U => ([a, m, b], [0, 1, 0]),
        Face::R => ([m, -b, -a], [1, 0, 0]),
        Face::F => ([a, -b, m], [0, 0, 1]),
        Face::D => ([a, -m, -b], [0, -1, 0]),
        Face::L => ([-m, -b, a], [-1, 0, 0]),
        Face::B => ([-a, -b, -m], [0, 0, -1]),
    }
}

fn index_of(n: usize, position: Vec3, normal: Vec3) -> usize {
    let m = n as i32 - 1;
    let [x, y, z] = position;
    let (face, a, b) = match normal {
        [0, 1, 0] => (Face::U, x, z),
        [1, 0, 0] => (Face::R, -z, -y),
        [0, 0, 1] => (Face::F, x, -y),
        [0, -1, 0] => (Face::D, x, -z),
        [-1, 0, 0] => (Face::L, z, -y),
        _ => (Face::B, -x, -y),
    };
    face as usize * n * n + ((b + m) / 2) as usize * n + ((a + m) / 2) as usize
}

fn quarter(axis: usize, v: Vec3) -> Vec3 {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// The sticker that lands on each index when the block is turned. The block
/// is layers `from..=to` counted from `face` (1 is the outer layer).
pub fn source_table(n: usize, mv: &LayerMove) -> Vec<usize> {
    let (axis, sign) = match mv.face {
        Face::U => (1, 1),
        Face::D => (1, -1),
        Face::R => (0, 1),
        Face::L => (0, -1),
        Face::F => (2, 1),
        Face::B => (2, -1),
    };
    let quarters = match (mv.turn, sign) {
        (Turn::Half, _) => 2,
        (Turn::Clockwise, 1) | (Turn::CounterClockwise, -1) => 3,
        _ => 1,
    };
    let m = n as i32 - 1;
    let hi = m - 2 * (i32::from(mv.from) - 1);
    let lo = m - 2 * (i32::from(mv.to) - 1);
    let mut src: Vec<usize> = (0..6 * n * n).collect();
    for index in 0..6 * n * n {
        let (position, normal) = locate(n, index);
        let height = sign * position[axis];
        if height > hi || height < lo {
            continue;
        }
        let (mut p, mut q) = (position, normal);
        for _ in 0..quarters {
            p = quarter(axis, p);
            q = quarter(axis, q);
        }
        src[index_of(n, p, q)] = index;
    }
    src
}

pub fn invert(src: &[usize]) -> Vec<usize> {
    let mut inv = vec![0; src.len()];
    for (i, &s) in src.iter().enumerate() {
        inv[s] = i;
    }
    inv
}

/// `first` then `second`.
pub fn compose(first: &[usize], second: &[usize]) -> Vec<usize> {
    second.iter().map(|&s| first[s]).collect()
}

pub fn apply(table: &[usize], state: &[u8], out: &mut [u8]) {
    for (slot, &s) in out.iter_mut().zip(table) {
        *slot = state[s];
    }
}

pub fn hash(state: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for chunk in state.chunks(8) {
        let mut word = 0u64;
        for &b in chunk {
            word = word << 8 | u64::from(b);
        }
        h = (h ^ word).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        h ^= h >> 29;
    }
    h | 1
}

/// The target in every orientation (even cubes, which have no fixed centres),
/// else the target alone.
pub fn seeds(n: usize, target: &[u8]) -> Vec<Vec<u8>> {
    if n % 2 == 1 {
        return vec![target.to_vec()];
    }
    let half = (n / 2) as u8;
    let mut rotations = Vec::new();
    for face in [Face::R, Face::U] {
        let opposite = Face::ALL[(face as usize + 3) % 6];
        let a = source_table(n, &LayerMove::new(face, 1, half, Turn::Clockwise));
        let b = source_table(
            n,
            &LayerMove::new(opposite, 1, half, Turn::CounterClockwise),
        );
        rotations.push(compose(&a, &b));
    }
    let mut out = vec![target.to_vec()];
    let mut seen = std::collections::HashSet::from([target.to_vec()]);
    let mut at = 0;
    while at < out.len() {
        for table in &rotations {
            let mut next = vec![0; target.len()];
            apply(table, &out[at], &mut next);
            if seen.insert(next.clone()) {
                out.push(next);
            }
        }
        at += 1;
    }
    out
}
