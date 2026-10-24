//! The move sets the search runs over.
//!
//! * `plain`: single outer blocks (`outer`) or any block of adjacent layers
//!   (`range`), one move each.
//! * `symmetric`: macro moves that turn a mirror-symmetric set of layers of one
//!   axis at once (layer `i` and layer `n-1-i` together, alike or opposite).
//!   Many well-known patterns are built from such moves, and the set is small
//!   enough to reach the big cubes; the result is exact only inside this set.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::puzzle::{Face, LayerMove, Turn};

use super::bidir::Step;
use super::geometry::{compose, invert, source_table};

const POSITIVE: [Face; 3] = [Face::R, Face::U, Face::F];
const NEGATIVE: [Face; 3] = [Face::L, Face::D, Face::B];

fn turn_of(quarters: usize) -> Turn {
    match quarters % 4 {
        1 => Turn::Clockwise,
        2 => Turn::Half,
        _ => Turn::CounterClockwise,
    }
}

pub fn step_of(n: usize, moves: &[LayerMove]) -> Step {
    let mut table: Vec<usize> = (0..6 * n * n).collect();
    for mv in moves {
        table = compose(&table, &source_table(n, mv));
    }
    Step {
        backward: invert(&table),
        forward: table,
        cost: moves.len(),
    }
}

/// A search step list and, for each step, the moves it is printed as.
pub struct Moves {
    pub steps: Vec<Step>,
    pub text: Vec<Vec<LayerMove>>,
}

pub fn plain(n: usize, range: bool, half_only: bool, face_only: bool) -> Moves {
    let turns: &[Turn] = if half_only {
        &[Turn::Half]
    } else {
        &[Turn::Clockwise, Turn::Half, Turn::CounterClockwise]
    };
    let top = (n - 1) as u8;
    let mut text = Vec::new();
    for turn in turns {
        for face in Face::ALL {
            let positive = (face as usize) < 3;
            for from in 1..=top {
                for to in from..=top {
                    // Ranges count from the positive faces; blocks reaching
                    // the far end are outer blocks of the negative ones.
                    let wanted = if range {
                        positive || from == 1
                    } else {
                        from == 1
                    };
                    if wanted && (!face_only || to == 1) {
                        text.push(vec![LayerMove::new(face, from, to, *turn)]);
                    }
                }
            }
        }
    }
    let steps = text.iter().map(|m| step_of(n, m)).collect();
    Moves { steps, text }
}

/// Fewest outer blocks for every net-turn vector of one axis (base 4, layer 0
/// from the positive face), with the moves, by breadth-first search.
fn outer_table(n: usize) -> HashMap<Vec<u8>, Vec<(bool, usize, usize)>> {
    let mut seen: HashMap<Vec<u8>, Vec<(bool, usize, usize)>> = HashMap::new();
    let zero = vec![0u8; n];
    seen.insert(zero.clone(), Vec::new());
    let mut queue = VecDeque::from([zero]);
    while let Some(net) = queue.pop_front() {
        let path = seen[&net].clone();
        for positive in [true, false] {
            for depth in 1..n {
                for q in 1..4usize {
                    let mut next = net.clone();
                    for i in 0..depth {
                        let layer = if positive { i } else { n - 1 - i };
                        let add = if positive { q } else { (4 - q) % 4 };
                        next[layer] = ((usize::from(next[layer]) + add) % 4) as u8;
                    }
                    if !seen.contains_key(&next) {
                        let mut longer = path.clone();
                        longer.push((positive, depth, q));
                        seen.insert(next.clone(), longer);
                        queue.push_back(next);
                    }
                }
            }
        }
    }
    seen
}

/// A block of equal net turn as a move on the nearer face.
fn block_move(n: usize, axis: usize, first: usize, last: usize, q: usize) -> LayerMove {
    if first <= n - 1 - last {
        LayerMove::new(
            POSITIVE[axis],
            (first + 1) as u8,
            (last + 1) as u8,
            turn_of(q),
        )
    } else {
        LayerMove::new(
            NEGATIVE[axis],
            (n - last) as u8,
            (n - first) as u8,
            turn_of((4 - q) % 4),
        )
    }
}

fn range_moves(n: usize, axis: usize, net: &[u8]) -> Vec<LayerMove> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let q = net[i];
        let mut j = i;
        while j + 1 < n && net[j + 1] == q {
            j += 1;
        }
        if q != 0 {
            out.push(block_move(n, axis, i, j, usize::from(q)));
        }
        i = j + 1;
    }
    out
}

pub fn symmetric(n: usize, range: bool, half_only: bool) -> Moves {
    let outer = if range { None } else { Some(outer_table(n)) };
    let units = n / 2 + n % 2;
    let mut text = Vec::new();
    for axis in 0..3 {
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        for mask in 1..(1usize << units) {
            let qs: &[u8] = if half_only { &[2] } else { &[1, 2, 3] };
            for &q in qs {
                for mirrored in [q, (4 - q) % 4] {
                    let mut net = vec![0u8; n];
                    for u in 0..units {
                        if mask >> u & 1 == 1 {
                            net[u] = q;
                            net[n - 1 - u] = if u == n - 1 - u { q } else { mirrored };
                        }
                    }
                    if net.iter().all(|&v| v == net[0]) || !seen.insert(net.clone()) {
                        continue;
                    }
                    let moves = match &outer {
                        None => range_moves(n, axis, &net),
                        Some(table) => table[&net]
                            .iter()
                            .map(|&(positive, depth, q)| {
                                let face = if positive {
                                    POSITIVE[axis]
                                } else {
                                    NEGATIVE[axis]
                                };
                                LayerMove::new(face, 1, depth as u8, turn_of(q))
                            })
                            .collect(),
                    };
                    text.push(moves);
                }
            }
        }
    }
    let steps = text.iter().map(|m| step_of(n, m)).collect();
    Moves { steps, text }
}
