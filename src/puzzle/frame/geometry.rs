//! Integer geometry for proper rigid corner rotations.

pub(super) type Vector = [i8; 3];

pub(super) fn axis(v: Vector) -> Option<usize> {
    let mut nonzero = None;
    for (i, value) in v.into_iter().enumerate() {
        if value != 0 {
            if !matches!(value, -1 | 1) || nonzero.is_some() {
                return None;
            }
            nonzero = Some(i);
        }
    }
    nonzero
}

pub(super) fn position_rank(p: Vector) -> Option<u8> {
    p.into_iter()
        .all(|v| matches!(v, -1 | 1))
        .then(|| ((p[0] == 1) as u8) * 4 + ((p[1] == 1) as u8) * 2 + (p[2] == 1) as u8)
}

pub(super) fn position(rank: u8) -> Vector {
    std::array::from_fn(|i| if rank & (1 << (2 - i)) == 0 { -1 } else { 1 })
}

pub(super) fn dot(a: Vector, b: Vector) -> i8 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

pub(super) fn quarter(v: Vector, axis: usize, sign: i8) -> Vector {
    let [x, y, z] = v;
    match axis {
        0 => [x, -sign * z, sign * y],
        1 => [sign * z, y, -sign * x],
        _ => [-sign * y, sign * x, z],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Rotation(pub(super) [[i8; 3]; 3]);

impl Rotation {
    pub(super) fn apply(self, v: Vector) -> Vector {
        self.0.map(|row| dot(row, v))
    }

    pub(super) fn quarter(self, axis: usize, sign: i8) -> Rotation {
        let columns: [Vector; 3] =
            std::array::from_fn(|j| quarter(std::array::from_fn(|i| self.0[i][j]), axis, sign));
        Rotation(std::array::from_fn(|i| {
            std::array::from_fn(|j| columns[j][i])
        }))
    }

    pub(super) fn twist_digit(self) -> u8 {
        self.0[0].iter().position(|v| *v != 0).unwrap() as u8
    }

    pub(super) fn all() -> Vec<Rotation> {
        let mut rotations = Vec::with_capacity(24);
        for a in 0..3 {
            for b in 0..3 {
                if a == b {
                    continue;
                }
                let c = 3 - a - b;
                for sa in [-1, 1] {
                    for sb in [-1, 1] {
                        let mut x = [0; 3];
                        let mut y = [0; 3];
                        x[a] = sa;
                        y[b] = sb;
                        let mut z = [0; 3];
                        z[c] = if (a + 1) % 3 == b { sa * sb } else { -sa * sb };
                        rotations.push(Rotation(std::array::from_fn(|i| [x[i], y[i], z[i]])));
                    }
                }
            }
        }
        rotations
    }
}
