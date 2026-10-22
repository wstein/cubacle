//! Putting one orbit of pieces in order with 3-cycles.
//!
//! The state of an orbit is a label per slot, and a goal label per slot. Each
//! step picks the library cycle that fixes the most slots per move; when no
//! single cycle gains, a pair of cycles is tried.

use super::library::Cycle;

/// Most steps an orbit may take; far above what any state needs.
const MAX_STEPS: usize = 200;

/// Slots that hold their goal label.
fn fixed(labels: &[u8], goal: &[u8]) -> i32 {
    labels.iter().zip(goal).filter(|(a, b)| a == b).count() as i32
}

/// What performing `cycle` changes in the number of fixed slots.
fn gain(labels: &[u8], goal: &[u8], cycle: &Cycle) -> i32 {
    let [a, b, c] = cycle.slots.map(usize::from);
    let hit = |label: u8, slot: usize| i32::from(label == goal[slot]);
    hit(labels[a], b) + hit(labels[b], c) + hit(labels[c], a)
        - hit(labels[a], a)
        - hit(labels[b], b)
        - hit(labels[c], c)
}

fn perform(labels: &mut [u8], cycle: &Cycle) {
    let [a, b, c] = cycle.slots.map(usize::from);
    let (la, lb, lc) = (labels[a], labels[b], labels[c]);
    labels[b] = la;
    labels[c] = lb;
    labels[a] = lc;
}

/// Whether `gain / length` of the first beats the second's.
fn better(first: (i32, usize), second: (i32, usize)) -> bool {
    i64::from(first.0) * second.1 as i64 > i64::from(second.0) * first.1 as i64
}

/// The cycles, in order, that bring `labels` to `goal`, or `None` if the
/// library cannot (an odd permutation of distinguishable pieces, or a gap).
/// `labels` is left in the goal state on success.
pub(super) fn solve(cycles: &[Cycle], labels: &mut [u8], goal: &[u8]) -> Option<Vec<usize>> {
    let mut chosen = Vec::new();
    while fixed(labels, goal) < labels.len() as i32 {
        if chosen.len() >= MAX_STEPS {
            return None;
        }
        let step = best_single(cycles, labels, goal).or_else(|| best_pair(cycles, labels, goal))?;
        for index in step {
            perform(labels, &cycles[index]);
            chosen.push(index);
        }
    }
    Some(chosen)
}

fn best_single(cycles: &[Cycle], labels: &[u8], goal: &[u8]) -> Option<Vec<usize>> {
    let mut best: Option<(usize, (i32, usize))> = None;
    for (index, cycle) in cycles.iter().enumerate() {
        let score = (gain(labels, goal, cycle), cycle.moves.len());
        if score.0 > 0 && best.is_none_or(|(_, old)| better(score, old)) {
            best = Some((index, score));
        }
    }
    best.map(|(index, _)| vec![index])
}

/// Two cycles that together gain, the first possibly gaining nothing itself.
fn best_pair(cycles: &[Cycle], labels: &[u8], goal: &[u8]) -> Option<Vec<usize>> {
    let mut best: Option<([usize; 2], (i32, usize))> = None;
    let mut after = labels.to_vec();
    for (first, one) in cycles.iter().enumerate() {
        let early = gain(labels, goal, one);
        if early < 0 {
            continue;
        }
        after.copy_from_slice(labels);
        perform(&mut after, one);
        for (second, two) in cycles.iter().enumerate() {
            let total = early + gain(&after, goal, two);
            let score = (total, one.moves.len() + two.moves.len());
            if total > 0 && best.is_none_or(|(_, old)| better(score, old)) {
                best = Some(([first, second], score));
            }
        }
    }
    best.map(|(pair, _)| pair.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All 3-cycles of six slots, one move each.
    fn all_cycles(n: u8) -> Vec<Cycle> {
        let mut cycles = Vec::new();
        for a in 0..n {
            for b in 0..n {
                for c in 0..n {
                    if a < b && a < c && b != c {
                        cycles.push(Cycle {
                            slots: [a, b, c],
                            moves: vec![0],
                        });
                    }
                }
            }
        }
        cycles
    }

    #[test]
    fn identical_labels_are_ordered_by_gain() {
        let cycles = all_cycles(6);
        let goal = [0, 0, 1, 1, 2, 2];
        let mut labels = [2, 1, 0, 2, 1, 0];
        let steps = solve(&cycles, &mut labels, &goal).expect("solvable");
        assert_eq!(labels, goal);
        assert!(steps.len() <= 3);
    }

    #[test]
    fn distinct_labels_need_an_even_permutation() {
        let cycles = all_cycles(5);
        let goal = [0, 1, 2, 3, 4];
        let mut even = [1, 2, 0, 4, 3];
        assert!(solve(&cycles, &mut even, &goal).is_none(), "odd");
        let mut even = [1, 2, 0, 3, 4];
        assert!(solve(&cycles, &mut even, &goal).is_some());
        assert_eq!(even, goal);
    }
}
