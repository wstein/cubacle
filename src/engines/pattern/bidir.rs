//! Bidirectional search over moves with small integer costs.
//!
//! The forward side starts at the solved cube, the backward side at the target
//! (all its orientations on even cubes) and applies inverse moves. Each side
//! settles its states cost level by cost level. A meeting is recorded when a
//! new state is already known to the other side; the search stops when no
//! unsettled path can beat the best meeting, so the result is a cheapest
//! sequence of the given moves.

use std::collections::hash_map::Entry as Slot;
use std::time::{Duration, Instant};

use super::geometry::{apply, hash, KeyMap};

/// One step of the search: a state permutation and what it costs.
pub struct Step {
    pub forward: Vec<usize>,
    pub backward: Vec<usize>,
    pub cost: usize,
}

pub struct Limits {
    pub max_cost: usize,
    pub time: Duration,
    pub max_states: usize,
}

struct Entry {
    cost: usize,
    parent: u64,
    step: u32,
}

const ROOT: u32 = u32::MAX;

struct Side {
    known: KeyMap<Entry>,
    /// States waiting at each cost, stored back to back.
    queue: Vec<Vec<u8>>,
    level: usize,
}

impl Side {
    fn new(roots: &[Vec<u8>]) -> Side {
        let mut side = Side {
            known: KeyMap::default(),
            queue: vec![Vec::new()],
            level: 0,
        };
        for root in roots {
            side.known.insert(
                hash(root),
                Entry {
                    cost: 0,
                    parent: 0,
                    step: ROOT,
                },
            );
            side.queue[0].extend_from_slice(root);
        }
        side
    }

    fn pending(&self) -> usize {
        self.queue.get(self.level).map_or(0, Vec::len)
    }
}

pub enum Outcome {
    /// The steps to apply in order, with their total cost.
    Found(Vec<usize>, usize),
    /// No sequence within the limits; the text says which limit.
    NotFound(String),
}

fn chain(known: &KeyMap<Entry>, mut at: u64) -> Vec<usize> {
    let mut steps = Vec::new();
    loop {
        let entry = &known[&at];
        if entry.step == ROOT {
            return steps;
        }
        steps.push(entry.step as usize);
        at = entry.parent;
    }
}

pub fn search(
    steps: &[Step],
    roots_fwd: Vec<Vec<u8>>,
    roots_bwd: Vec<Vec<u8>>,
    limits: &Limits,
) -> Outcome {
    let len = roots_fwd[0].len();
    let mut sides = [Side::new(&roots_fwd), Side::new(&roots_bwd)];
    let mut best: Option<(usize, u64)> = sides[1]
        .known
        .keys()
        .find(|h| sides[0].known.contains_key(h))
        .map(|h| (0, *h));
    let start = Instant::now();
    let mut buffer = vec![0u8; len];
    loop {
        if let Some((cost, _)) = best {
            if sides[0].level + sides[1].level >= cost {
                break;
            }
        }
        if sides[0].level + sides[1].level > limits.max_cost {
            return Outcome::NotFound(format!("none within cost {}", limits.max_cost));
        }
        let total = sides[0].known.len() + sides[1].known.len();
        if start.elapsed() > limits.time || total > limits.max_states {
            return Outcome::NotFound(format!(
                "budget exhausted at levels {}+{}",
                sides[0].level, sides[1].level
            ));
        }
        let live: Vec<usize> = (0..2)
            .filter(|&i| sides[i].level < sides[i].queue.len())
            .collect();
        // Empty levels are free to skip; otherwise expand the smaller side.
        let pick = match live[..] {
            [] => return Outcome::NotFound("search space exhausted".into()),
            [only] => only,
            _ => match (sides[0].pending(), sides[1].pending()) {
                (0, _) => 0,
                (_, 0) => 1,
                (a, b) => usize::from(b < a),
            },
        };
        let (mine, theirs) = if pick == 0 {
            let (a, b) = sides.split_at_mut(1);
            (&mut a[0], &b[0])
        } else {
            let (a, b) = sides.split_at_mut(1);
            (&mut b[0], &a[0])
        };
        let level = mine.level;
        let states = std::mem::take(&mut mine.queue[level]);
        for state in states.chunks(len) {
            let here = hash(state);
            if mine.known[&here].cost != level {
                continue;
            }
            for (index, step) in steps.iter().enumerate() {
                let table = if pick == 0 {
                    &step.forward
                } else {
                    &step.backward
                };
                apply(table, state, &mut buffer);
                let key = hash(&buffer);
                let cost = level + step.cost;
                let entry = Entry {
                    cost,
                    parent: here,
                    step: index as u32,
                };
                match mine.known.entry(key) {
                    Slot::Occupied(mut old) => {
                        if old.get().cost <= cost {
                            continue;
                        }
                        old.insert(entry);
                    }
                    Slot::Vacant(slot) => {
                        slot.insert(entry);
                    }
                }
                while mine.queue.len() <= cost {
                    mine.queue.push(Vec::new());
                }
                mine.queue[cost].extend_from_slice(&buffer);
                if let Some(other) = theirs.known.get(&key) {
                    let total = cost + other.cost;
                    if best.is_none_or(|(b, _)| total < b) {
                        best = Some((total, key));
                    }
                }
            }
        }
        mine.level += 1;
    }
    let (cost, key) = best.expect("loop only breaks with a meeting");
    let mut path = chain(&sides[0].known, key);
    path.reverse();
    path.extend(chain(&sides[1].known, key));
    Outcome::Found(path, cost)
}
