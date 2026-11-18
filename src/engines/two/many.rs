//! A list of distinct 2x2x2 solutions: [`Solver::solve_many`].
//!
//! New code. The distance table knows how far every state is from solved, so
//! the solutions of length `L` are found by a depth-first walk that only takes
//! moves that can still finish in the moves left. The list is every optimal
//! solution in move order (the first is the one of a plain solve), then the
//! solutions one move longer, and so on.

use std::time::Instant;

use super::{normalize, search_move, tables, PocketSolver, Tables};
use crate::puzzle::{
    same_up_to_commuting, Face, Facelets, Move, MoveSequence, SolveEvidence, SolveFrame,
    SolveMetrics, SolveOptions, SolveResult, Solver,
};

/// Paths of exactly `left` more moves from `state` to the solved state that
/// never turn one face twice in a row, appended to `out` until it holds `want`.
fn walk(
    tables: &Tables,
    state: u32,
    left: usize,
    last_face: Option<usize>,
    path: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
    want: usize,
) {
    if out.len() >= want {
        return;
    }
    if left == 0 {
        if tables.depth(state) == 0 {
            out.push(path.clone());
        }
        return;
    }
    for m in 0..9 {
        if last_face == Some(m / 3) {
            continue;
        }
        let next = tables.step(state, m);
        if usize::from(tables.depth(next)) < left {
            path.push(m);
            walk(tables, next, left - 1, Some(m / 3), path, out, want);
            path.pop();
        }
    }
}

impl PocketSolver {
    /// Up to `n` distinct solutions, the plain solution first.
    pub(super) fn list(
        &self,
        facelets: &Facelets,
        options: &SolveOptions,
        n: usize,
    ) -> Vec<SolveResult> {
        let started = Instant::now();
        let first = self.solve(facelets, options);
        if first.outcome.is_err() {
            return vec![first];
        };
        if n <= 1 {
            return vec![first];
        }
        let Ok(normalized) = normalize(facelets) else {
            return vec![first];
        };
        let tables = tables();
        let shortest = usize::from(tables.depth(normalized.state));
        let longest = options.max_moves.map(usize::from);
        let mut found: Vec<SolveResult> = vec![first.clone()];
        let mut length = shortest;
        // Every state is solved in at most 11 moves, so a few more lengths give
        // many solutions; the loop ends at `n` or at the move limit.
        while found.len() < n && length <= shortest + 3 && longest.is_none_or(|max| length <= max) {
            let mut paths = Vec::new();
            walk(
                tables,
                normalized.state,
                length,
                None,
                &mut Vec::new(),
                &mut paths,
                n + found.len(),
            );
            for path in paths {
                if found.len() >= n {
                    break;
                }
                let moves: Vec<Move> = path
                    .iter()
                    .map(|&m| {
                        let (face, turn) = search_move(m);
                        Move::new(Face::ALL[normalized.origin[face]], 1, false, turn)
                    })
                    .collect();
                let solution = MoveSequence::new(moves);
                if found.iter().any(|r| {
                    r.solution()
                        .is_some_and(|f| same_up_to_commuting(f.moves(), solution.moves()))
                }) {
                    continue;
                }
                let evidence = if length == shortest {
                    SolveEvidence::optimal(SolveFrame::AnyOrientation, length)
                } else {
                    SolveEvidence::default()
                };
                found.push(SolveResult {
                    outcome: Ok(solution),
                    message: None,
                    metrics: SolveMetrics {
                        elapsed: started.elapsed(),
                        probes: None,
                        evidence,
                    },
                    stages: Vec::new(),
                });
            }
            length += 1;
        }
        found
    }
}
