use super::geometry::position;
use super::{invalid, move_index, FrameError, Move, Print, State, Sticker};
use crate::puzzle::MoveSequence;

impl Print {
    /// Checks occupancy and the complete corner-twist reachability invariant.
    pub fn state_from_poses(&self, poses: [u8; 8]) -> Result<State, FrameError> {
        let mut occupied = [false; 8];
        let mut twist = 0;
        for (body, pose) in poses.into_iter().enumerate() {
            if pose >= 24 || occupied[pose as usize / 3] {
                return Err(invalid(
                    "poses must place eight bodies at distinct positions",
                ));
            }
            occupied[pose as usize / 3] = true;
            twist += self.orientation(body, pose);
        }
        if twist % 3 != 0 {
            return Err(invalid("unreachable state: corner twist sum is not zero"));
        }
        Ok(State { poses })
    }

    /// Restores a state only when every sticker follows the same proper body rotation.
    pub fn state_from_stickers(&self, mut stickers: Vec<Sticker>) -> Result<State, FrameError> {
        if stickers.len() != 24 {
            return Err(invalid("a state needs exactly 24 sticker frames"));
        }
        stickers.sort_by_key(|s| s.id);
        for (expected, actual) in self.stickers.iter().zip(&stickers) {
            if actual.id != expected.id
                || actual.cubie != expected.cubie
                || actual.image != expected.image
            {
                return Err(invalid(
                    "state must preserve all sticker identities and body bindings",
                ));
            }
        }
        let mut poses = [0; 8];
        for (body, pose) in poses.iter_mut().enumerate() {
            *pose = (0..24)
                .find(|candidate| {
                    let r = self.rotations[body][*candidate as usize];
                    self.stickers.iter().zip(&stickers).all(|(a, b)| {
                        a.cubie as usize != body
                            || (r.apply(a.p) == b.p && r.apply(a.n) == b.n && r.apply(a.up) == b.up)
                    })
                })
                .ok_or_else(|| invalid("stickers do not form a proper rigid body rotation"))?;
        }
        self.state_from_poses(poses)
    }

    /// Expands a complete state to immutable sticker identities and current artwork frames.
    pub fn stickers(&self, state: &State) -> Result<[Sticker; 24], FrameError> {
        self.state_from_poses(state.poses)?;
        Ok(std::array::from_fn(|id| {
            let mut s = self.stickers[id].clone();
            let r = self.rotations[s.cubie as usize][state.poses[s.cubie as usize] as usize];
            s.p = r.apply(s.p);
            s.n = r.apply(s.n);
            s.up = r.apply(s.up);
            s
        }))
    }

    pub fn apply(&self, state: &State, mv: Move) -> Result<State, FrameError> {
        self.apply_moves(state, &MoveSequence::new(vec![mv]))
    }

    pub fn apply_moves(&self, state: &State, moves: &MoveSequence) -> Result<State, FrameError> {
        self.state_from_poses(state.poses)?;
        let mut poses = state.poses;
        for mv in moves.moves() {
            let m = move_index(*mv)?;
            for (body, pose) in poses.iter_mut().enumerate() {
                *pose = self.move_table[body][*pose as usize][m];
            }
        }
        Ok(State { poses })
    }

    /// One-body action, usable by projected state graphs without a hidden-state choice.
    pub fn moved_pose(&self, body: u8, pose: u8, mv: Move) -> Result<u8, FrameError> {
        if body >= 8 || pose >= 24 {
            return Err(invalid("body or pose outside the fixed-frame domain"));
        }
        Ok(self.move_table[body as usize][pose as usize][move_index(mv)?])
    }

    /// The lexicographically lowest full key completing these assigned body poses.
    pub fn complete(&self, partial: [Option<u8>; 8]) -> Result<State, FrameError> {
        let mut occupied = [false; 8];
        for pose in partial.into_iter().flatten() {
            if pose >= 24 || occupied[pose as usize / 3] {
                return Err(invalid("partial poses overlap or lie outside 0..23"));
            }
            occupied[pose as usize / 3] = true;
        }
        let Some(last) = partial.iter().rposition(Option::is_none) else {
            return self.state_from_poses(partial.map(Option::unwrap));
        };
        let mut poses = [0; 8];
        for (body, pose) in poses.iter_mut().enumerate() {
            *pose = if let Some(given) = partial[body] {
                given
            } else {
                let free = occupied.iter().position(|used| !used).unwrap();
                occupied[free] = true;
                3 * free as u8
            };
        }
        for digit in 0..3 {
            poses[last] = poses[last] / 3 * 3 + digit;
            if let Ok(state) = self.state_from_poses(poses) {
                return Ok(state);
            }
        }
        Err(invalid("partial state has no reachable completion"))
    }

    fn orientation(&self, body: usize, pose: u8) -> u8 {
        let p = position(pose / 3);
        let r = self.rotations[body][pose as usize];
        let normal = r.apply([0, self.positions[body][1], 0]);
        let [x, y, z] = [[p[0], 0, 0], [0, p[1], 0], [0, 0, p[2]]];
        let cyclic = if p[0] * p[1] * p[2] == 1 {
            [y, z, x]
        } else {
            [y, x, z]
        };
        cyclic.iter().position(|n| *n == normal).unwrap() as u8
    }
}
