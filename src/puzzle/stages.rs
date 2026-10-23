//! The stages an engine reports with its answer, and how their ends follow
//! the answer through post-processing.
//!
//! An engine knows where its own stages end in the raw moves it appended. The
//! answer is then normalised (axis runs rewritten, turns merged), and some
//! engines rotate it into the standard frame, so the raw indices no longer
//! point at the same places. [`stage_ends`] finds, for each raw end, the
//! position in the final moves where the cube is in the same state, or in
//! the closest one if the rewrite left none.

use super::{Facelets, Move, MoveSequence};

/// One stage of an engine's answer, as the engine itself saw it.
///
/// The stages of a result are in order and cover the solution without gaps:
/// each ends where the next begins, the first begins at move 0 and the last
/// ends at the end of the solution. A stage with no moves is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolveStage {
    /// What the engine did in this stage, in a few words a learner can read.
    pub name: &'static str,
    /// The number of moves of the solution up to the end of this stage.
    pub end: usize,
}

/// The positions in `fin` that correspond to the raw ends `raw_ends` of
/// `raw`, never decreasing; the last raw end maps to the end of `fin`.
///
/// `start` is the cube before the first move; `raw` and `fin` are equivalent
/// move lists for it, up to one of the whole-cube `rotations` (lists of
/// moves; empty for none) at their ends. For each end, the first position at
/// or after the previous one where the state equals the raw state (turned by
/// one of the rotations) is taken. If no position matches, the one with the
/// most equal stickers is, which happens only when the rewrite of a run of
/// moves on one axis straddled the end.
pub(crate) fn stage_ends(
    start: &Facelets,
    raw: &[Move],
    raw_ends: &[usize],
    fin: &[Move],
    rotations: &[Vec<Move>],
) -> Vec<usize> {
    let clamp = |end: usize| end.min(fin.len());
    let (Some(fin_states), Some(raw_states)) =
        (prefix_states(start, fin), prefix_states(start, raw))
    else {
        return raw_ends.iter().map(|&e| clamp(e)).collect();
    };
    let mut out: Vec<usize> = Vec::with_capacity(raw_ends.len());
    let mut from = 0;
    for &end in raw_ends {
        let end = end.min(raw.len());
        let at = if end == raw.len() {
            fin.len()
        } else {
            let target = &raw_states[end];
            let turned: Vec<Facelets> = if rotations.is_empty() {
                vec![target.clone()]
            } else {
                rotations
                    .iter()
                    .filter_map(|r| target.apply(&MoveSequence::new(r.clone())).ok())
                    .collect()
            };
            closest(&fin_states, from, &turned)
        };
        let at = at.max(from);
        out.push(at);
        from = at;
    }
    out
}

fn prefix_states(start: &Facelets, moves: &[Move]) -> Option<Vec<Facelets>> {
    let mut states = Vec::with_capacity(moves.len() + 1);
    let mut state = start.clone();
    states.push(state.clone());
    for mv in moves {
        state = state.apply(&MoveSequence::new(vec![*mv])).ok()?;
        states.push(state.clone());
    }
    Some(states)
}

/// The first index from `from` whose state is one of `targets`, else the one
/// that shares the most stickers with any of them.
fn closest(states: &[Facelets], from: usize, targets: &[Facelets]) -> usize {
    if let Some(i) = (from..states.len()).find(|&i| targets.contains(&states[i])) {
        return i;
    }
    let score = |state: &Facelets| {
        targets
            .iter()
            .map(|t| {
                t.stickers()
                    .iter()
                    .zip(state.stickers())
                    .filter(|(a, b)| a == b)
                    .count()
            })
            .max()
            .unwrap_or(0)
    };
    let mut best = from;
    let mut best_score = 0;
    for (i, state) in states.iter().enumerate().skip(from) {
        let s = score(state);
        if s > best_score {
            best = i;
            best_score = s;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{normalise, PuzzleKind};

    fn seq(text: &str) -> Vec<Move> {
        text.parse::<MoveSequence>().unwrap().moves().to_vec()
    }

    #[test]
    fn ends_follow_a_rewrite_that_keeps_the_state() {
        let start = Facelets::solved(PuzzleKind::Cube5);
        // "R R" merges into "R2"; the ends before and after it survive.
        let raw = seq("U R R U' F");
        let fin = normalise(5, &raw, false);
        assert_eq!(fin.len(), 4);
        let ends = stage_ends(&start, &raw, &[1, 3, 5], &fin, &[]);
        assert_eq!(ends, vec![1, 2, 4]);
    }

    #[test]
    fn an_end_inside_a_merged_run_is_the_closest_position() {
        let start = Facelets::solved(PuzzleKind::Cube5);
        let raw = seq("U R R U'");
        let fin = normalise(5, &raw, false);
        let ends = stage_ends(&start, &raw, &[2, 4], &fin, &[]);
        assert!(ends[0] <= ends[1] && ends[1] == fin.len());
    }

    #[test]
    fn an_empty_stage_stays_empty_and_ends_never_decrease() {
        let start = Facelets::solved(PuzzleKind::Cube4);
        let raw = seq("U R U R'");
        let ends = stage_ends(&start, &raw, &[0, 0, 2, 4], &raw, &[]);
        assert_eq!(ends, vec![0, 0, 2, 4]);
    }
}
