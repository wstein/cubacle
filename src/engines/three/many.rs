//! A list of distinct 3x3x3 solutions: [`Solver::solve_many`].
//!
//! New code. The two-phase search answers one solution per cube. The list is
//! the plain solution, then the answers for the cube after one more turn
//! (every face, every turn), that turn put in front, then after two turns:
//! start variants, in a fixed order, so that the list is the same on every
//! machine. Each is searched under the options of a plain solve, so it is
//! about as short as the plain one plus the turns in front; a variant that
//! cancels into a solution already listed, or goes over the move limit, is
//! left out.

use std::time::Instant;

use super::solver::Min2PhaseSolver;
use crate::puzzle::{
    normalize, same_up_to_commuting, Face, Facelets, Move, MoveSequence, SolveOptions, SolveResult,
    Solver, Turn,
};

/// The most turns put in front of a variant.
const MAX_PREFIX: usize = 2;

/// The turns that start a variant: one turn, then two turns of different faces.
fn prefixes() -> Vec<Vec<Move>> {
    let turns = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];
    let singles: Vec<Move> = Face::ALL
        .iter()
        .flat_map(|&face| {
            turns
                .iter()
                .map(move |&turn| Move::new(face, 1, false, turn))
        })
        .collect();
    let mut all: Vec<Vec<Move>> = singles.iter().map(|&m| vec![m]).collect();
    if MAX_PREFIX >= 2 {
        for &a in &singles {
            for &b in &singles {
                if a.face != b.face {
                    all.push(vec![a, b]);
                }
            }
        }
    }
    all
}

impl Min2PhaseSolver {
    /// Up to `n` distinct solutions, the plain solution first.
    pub(super) fn list(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        n: usize,
    ) -> Vec<SolveResult> {
        let started = Instant::now();
        let first = self.solve(facelets, options);
        if n <= 1 || first.outcome.is_err() {
            return vec![first];
        }
        let mut found = vec![first];
        let limit = options.max_moves.map(usize::from);
        for prefix in prefixes() {
            if found.len() >= n {
                break;
            }
            let remaining = match options.time_budget {
                Some(budget) => match budget.checked_sub(started.elapsed()) {
                    Some(left) if !left.is_zero() => Some(left),
                    _ => break,
                },
                None => None,
            };
            let Ok(turned) = facelets.apply(&MoveSequence::new(prefix.clone())) else {
                continue;
            };
            let options = SolveOptions {
                time_budget: remaining,
                ..*options
            };
            let result = self.solve(&turned, &options);
            let Some(tail) = result.solution() else {
                continue;
            };
            let mut moves = prefix;
            moves.extend_from_slice(tail.moves());
            let solution = MoveSequence::new(normalize(3, &moves, false));
            if limit.is_some_and(|max| solution.len() > max)
                || found.iter().any(|r| {
                    r.solution()
                        .is_some_and(|f| same_up_to_commuting(f.moves(), solution.moves()))
                })
            {
                continue;
            }
            found.push(SolveResult {
                outcome: Ok(solution),
                message: None,
                metrics: result.metrics,
                stages: Vec::new(),
            });
        }
        found
    }
}
