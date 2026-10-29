use super::{invalid, FrameError, Print, State};

/// The canonical `key.v1` projection on an ascending set of physical body ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    bodies: Vec<u8>,
    encoded: usize,
}

impl Projection {
    pub fn new(mut bodies: Vec<u8>) -> Result<Projection, FrameError> {
        bodies.sort_unstable();
        if bodies.iter().any(|b| *b >= 8) || bodies.windows(2).any(|w| w[0] == w[1]) {
            return Err(invalid("tracked bodies must be distinct ids in 0..7"));
        }
        Ok(Projection {
            encoded: bodies.len().min(7),
            bodies,
        })
    }

    pub fn bodies(&self) -> &[u8] {
        &self.bodies
    }

    pub fn keys(&self) -> u32 {
        (0..self.encoded).fold(1, |n, j| n * (8 - j) as u32 * 3)
    }

    pub fn encode(&self, state: &State) -> u32 {
        let poses: Vec<_> = self
            .bodies
            .iter()
            .map(|b| state.poses[*b as usize])
            .collect();
        self.rank(&poses[..self.encoded])
    }

    /// Encodes tracked poses in ascending [`Projection::bodies`] order, without
    /// inventing placements for untracked bodies.
    pub fn encode_poses(&self, print: &Print, poses: &[u8]) -> Result<u32, FrameError> {
        if poses.len() != self.bodies.len() {
            return Err(invalid("one pose is required per tracked body"));
        }
        let mut partial = [None; 8];
        for (&body, &pose) in self.bodies.iter().zip(poses) {
            partial[body as usize] = Some(pose);
        }
        print.complete(partial)?;
        Ok(self.rank(&poses[..self.encoded]))
    }

    /// Decodes in tracked-body order; at k=8 the last pose follows from reachability.
    pub fn decode(&self, print: &Print, key: u32) -> Result<Vec<u8>, FrameError> {
        if key >= self.keys() {
            return Err(invalid("key outside projection domain"));
        }
        let twists = 3u32.pow(self.encoded as u32);
        let mut position_part = key / twists;
        let mut twist_part = key % twists;
        let mut digits = vec![0; self.encoded];
        let mut poses = vec![0; self.encoded];
        for j in (0..self.encoded).rev() {
            digits[j] = position_part % (8 - j) as u32;
            position_part /= (8 - j) as u32;
            poses[j] = (twist_part % 3) as u8;
            twist_part /= 3;
        }
        let mut available: Vec<_> = (0u8..8).collect();
        for j in 0..self.encoded {
            poses[j] += 3 * available.remove(digits[j] as usize);
        }
        if self.bodies.len() == 8 {
            let mut partial = [None; 8];
            for (body, pose) in poses.iter().enumerate() {
                partial[body] = Some(*pose);
            }
            poses = print.complete(partial)?.poses.to_vec();
        }
        Ok(poses)
    }

    fn rank(&self, poses: &[u8]) -> u32 {
        let mut used = [false; 8];
        let mut positions = 0;
        let mut twists = 0;
        for (j, &pose) in poses.iter().enumerate() {
            let p = pose as usize / 3;
            let digit = used[..p].iter().filter(|u| !**u).count() as u32;
            positions = positions * (8 - j) as u32 + digit;
            twists = twists * 3 + u32::from(pose % 3);
            used[p] = true;
        }
        positions * 3u32.pow(self.encoded as u32) + twists
    }
}
