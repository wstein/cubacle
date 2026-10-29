use std::mem::size_of;

use sha2::{Digest, Sha256};

use crate::puzzle::frame::{moves, State, Sticker};
use crate::puzzle::MoveSequence;

use super::coordinates::Coordinates;
use super::{BuildLimits, FixedError, FrameGoal};

const UNSEEN: u8 = 255;

/// A shortest HTM word and the complete physical state obtained by replaying it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub key: u32,
    pub distance: u8,
    pub path: MoveSequence,
    pub reached: State,
}

/// A verified-construction distance table; no table is exposed for an unsatisfiable display.
#[derive(Debug)]
pub struct GoalTable {
    goal: FrameGoal,
    distances: Option<Vec<u8>>,
    histogram: Vec<u32>,
    goal_keys: u32,
    sha256: Option<String>,
    build_memory_bytes: usize,
}

impl GoalTable {
    /// Build exact distances by multi-source BFS. The entire u32 queue is reserved up front,
    /// so its worst-case live allocation is counted rather than hidden by Vec growth.
    pub fn build(goal: FrameGoal, limits: BuildLimits) -> Result<GoalTable, FixedError> {
        limits.check_deadline()?;
        let keys = goal.projection().keys() as usize;
        // Coordinate setup temporarily expands one complete sticker array, including artwork.
        let setup_scratch = goal
            .print()
            .printing()
            .iter()
            .filter_map(|sticker| sticker.image.as_ref())
            .map(String::len)
            .sum::<usize>()
            + size_of::<[Sticker; 24]>();
        let build_memory_bytes = keys
            .checked_mul(1 + size_of::<u32>())
            .and_then(|bytes| bytes.checked_add(goal.memory_bytes()))
            .and_then(|bytes| {
                bytes.checked_add(
                    size_of::<Coordinates>()
                        + 8
                        + 255 * size_of::<u32>()
                        + size_of::<Self>()
                        + 64
                        + setup_scratch,
                )
            })
            .ok_or(FixedError::OverLimit)?;
        if build_memory_bytes > limits.max_memory_bytes {
            return Err(FixedError::OverLimit);
        }
        let mut distances = Vec::new();
        distances
            .try_reserve_exact(keys)
            .map_err(|_| FixedError::OverLimit)?;
        distances.resize(keys, UNSEEN);
        let mut queue = Vec::new();
        queue
            .try_reserve_exact(keys)
            .map_err(|_| FixedError::OverLimit)?;
        let coordinates = Coordinates::new(goal.print(), goal.projection());
        for key in 0..keys as u32 {
            if key % 4096 == 0 {
                limits.check_deadline()?;
            }
            if goal.matches_poses(&coordinates.decode(key)) {
                distances[key as usize] = 0;
                queue.push(key);
            }
        }
        limits.check_deadline()?;
        let goal_keys = queue.len() as u32;
        if goal_keys == 0 {
            return Ok(GoalTable {
                goal,
                distances: None,
                histogram: Vec::new(),
                goal_keys: 0,
                sha256: None,
                build_memory_bytes,
            });
        }
        let mut histogram = Vec::with_capacity(255);
        histogram.push(goal_keys);
        let mut head = 0;
        while head < queue.len() {
            limits.check_deadline()?;
            let layer_end = queue.len();
            while head < layer_end {
                if head % 4096 == 0 {
                    limits.check_deadline()?;
                }
                let key = queue[head];
                head += 1;
                let depth = distances[key as usize];
                let next_depth = depth
                    .checked_add(1)
                    .filter(|value| *value < UNSEEN)
                    .ok_or(FixedError::Internal("distance exceeds dense encoding"))?;
                let poses = coordinates.decode(key);
                for mv in 0..18 {
                    let to = coordinates.successor(&poses, mv) as usize;
                    if distances[to] == UNSEEN {
                        distances[to] = next_depth;
                        if histogram.len() <= next_depth as usize {
                            histogram.push(0);
                        }
                        histogram[next_depth as usize] += 1;
                        queue.push(to as u32);
                    }
                }
            }
        }
        if queue.len() != keys {
            return Err(FixedError::Internal("projected graph is disconnected"));
        }
        limits.check_deadline()?;
        let sha256 = Sha256::digest(&distances)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        limits.check_deadline()?;
        Ok(GoalTable {
            goal,
            distances: Some(distances),
            histogram,
            goal_keys,
            sha256: Some(sha256),
            build_memory_bytes,
        })
    }

    pub fn goal(&self) -> &FrameGoal {
        &self.goal
    }
    pub fn goal_keys(&self) -> u32 {
        self.goal_keys
    }
    pub fn histogram(&self) -> &[u32] {
        &self.histogram
    }
    pub fn eccentricity(&self) -> Option<u8> {
        self.distances
            .as_ref()
            .map(|_| (self.histogram.len() - 1) as u8)
    }
    pub fn distances(&self) -> Option<&[u8]> {
        self.distances.as_deref()
    }
    pub fn sha256(&self) -> Option<&str> {
        self.sha256.as_deref()
    }
    /// Conservative peak build bytes, including the entire reserved queue and goal metadata.
    pub fn build_memory_bytes(&self) -> usize {
        self.build_memory_bytes
    }
    pub fn memory_bytes(&self) -> usize {
        self.goal.memory_bytes()
            + size_of::<Self>()
            + self.distances.as_ref().map_or(0, Vec::capacity)
            + self.histogram.capacity() * size_of::<u32>()
            + self.sha256.as_ref().map_or(0, String::capacity)
    }

    pub fn distance_key(&self, key: u32) -> Result<u8, FixedError> {
        self.distances
            .as_ref()
            .ok_or(FixedError::Unsatisfiable)?
            .get(key as usize)
            .copied()
            .ok_or_else(|| FixedError::InvalidState("key outside goal projection".to_owned()))
    }

    pub fn distance(&self, state: &State) -> Result<u8, FixedError> {
        self.goal
            .print()
            .state_from_poses(*state.poses())
            .map_err(|e| FixedError::InvalidState(e.to_string()))?;
        self.distance_key(self.goal.projection().encode(state))
    }

    /// Greedy descent in the canonical 18-move order gives the lexicographically least
    /// shortest word. Every chosen move is applied to all eight bodies before returning.
    pub fn shortest_path(&self, from: &State) -> Result<Transition, FixedError> {
        let distance = self.distance(from)?;
        let key = self.goal.projection().encode(from);
        let mut current = *from;
        let mut path = Vec::with_capacity(distance as usize);
        for remaining in (1..=distance).rev() {
            let mut next = None;
            for mv in moves() {
                let reached = self
                    .goal
                    .print()
                    .apply(&current, mv)
                    .map_err(|e| FixedError::InvalidState(e.to_string()))?;
                if self.distance_key(self.goal.projection().encode(&reached))? == remaining - 1 {
                    next = Some((mv, reached));
                    break;
                }
            }
            let (mv, reached) =
                next.ok_or(FixedError::Internal("table has no descending successor"))?;
            path.push(mv);
            current = reached;
        }
        if !self.goal.matches(&current)? {
            return Err(FixedError::Internal(
                "shortest word does not reach the display",
            ));
        }
        Ok(Transition {
            key,
            distance,
            path: MoveSequence::new(path),
            reached: current,
        })
    }
}
