//! Allocation-free dense coordinate actions for exact projected graphs.

use crate::puzzle::frame::{moves, Print, Projection};

pub(super) struct Coordinates {
    bodies: Vec<u8>,
    encoded: usize,
    twists: u32,
    turns: [[[u8; 18]; 24]; 8],
    orientation: [[u8; 24]; 8],
    last_pose: [[u8; 3]; 8],
}

impl Coordinates {
    pub(super) fn new(print: &Print, projection: &Projection) -> Self {
        let mut orientation = [[0; 24]; 8];
        for (body, values) in orientation.iter_mut().enumerate() {
            let id = print
                .printing()
                .iter()
                .find(|sticker| sticker.cubie as usize == body && sticker.n[1] != 0)
                .unwrap()
                .id as usize;
            for (pose, value) in values.iter_mut().enumerate() {
                let mut partial = [None; 8];
                partial[body] = Some(pose as u8);
                let state = print
                    .complete(partial)
                    .expect("one body pose always extends");
                let current = print.stickers(&state).unwrap();
                let [x, y, z] = current[id].p;
                let cyclic = if x * y * z == 1 {
                    [[0, y, 0], [0, 0, z], [x, 0, 0]]
                } else {
                    [[0, y, 0], [x, 0, 0], [0, 0, z]]
                };
                *value = cyclic
                    .iter()
                    .position(|normal| *normal == current[id].n)
                    .unwrap() as u8;
            }
        }
        let mut last_pose = [[0; 3]; 8];
        // Eight-body projections always name all eight ids, so the omitted body is 7.
        for pose in 0..24 {
            last_pose[pose / 3][orientation[7][pose] as usize] = pose as u8;
        }
        let actions = moves();
        Self {
            bodies: projection.bodies().to_vec(),
            encoded: projection.bodies().len().min(7),
            twists: 3u32.pow(projection.bodies().len().min(7) as u32),
            turns: std::array::from_fn(|body| {
                std::array::from_fn(|pose| {
                    std::array::from_fn(|mv| {
                        print
                            .moved_pose(body as u8, pose as u8, actions[mv])
                            .unwrap()
                    })
                })
            }),
            orientation,
            last_pose,
        }
    }

    pub(super) fn decode(&self, key: u32) -> [u8; 8] {
        let mut positions = key / self.twists;
        let mut twists = key % self.twists;
        let mut digits = [0u8; 7];
        let mut out = [0u8; 8];
        for j in (0..self.encoded).rev() {
            digits[j] = (positions % (8 - j) as u32) as u8;
            positions /= (8 - j) as u32;
            out[self.bodies[j] as usize] = (twists % 3) as u8;
            twists /= 3;
        }
        let mut used = 0u8;
        for (j, digit) in digits.iter().enumerate().take(self.encoded) {
            let position = (0..8)
                .filter(|position| used & (1 << position) == 0)
                .nth(*digit as usize)
                .unwrap();
            used |= 1 << position;
            out[self.bodies[j] as usize] += 3 * position;
        }
        if self.bodies.len() == 8 {
            let sum: u8 = (0..7)
                .map(|body| self.orientation[body][out[body] as usize])
                .sum();
            out[7] =
                self.last_pose[(!used).trailing_zeros() as usize][((3 - sum % 3) % 3) as usize];
        }
        out
    }

    pub(super) fn encode(&self, poses: &[u8; 8]) -> u32 {
        let mut used = 0u8;
        let mut positions = 0u32;
        let mut twists = 0u32;
        for (j, body) in self.bodies.iter().take(self.encoded).enumerate() {
            let pose = poses[*body as usize];
            let position = pose / 3;
            let digit = (((1u16 << position) - 1) as u8 & !used).count_ones();
            positions = positions * (8 - j) as u32 + digit;
            twists = twists * 3 + u32::from(pose % 3);
            used |= 1 << position;
        }
        positions * self.twists + twists
    }

    pub(super) fn successor(&self, poses: &[u8; 8], mv: usize) -> u32 {
        let mut moved = [0; 8];
        for body in &self.bodies {
            moved[*body as usize] = self.turns[*body as usize][poses[*body as usize] as usize][mv];
        }
        self.encode(&moved)
    }
}
