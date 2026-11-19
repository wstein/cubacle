//! A list of distinct solutions of a 6x6x6 or 7x7x7: [`Solver::solve_many`].
//!
//! New code. The center stages keep one route, so every solution of the list
//! shares the center moves; the list varies the stages after them. Each of
//! the two wing stages lists `k` reductions (of each cube the stage before
//! leaves) and the 3x3x3 stage lists `k` solutions of each. Every combination
//! is one candidate, taken depth first, so the first is the plain solution.
//! Candidates equal after normalizing and sorting the turns about one axis
//! count once. When a round of `k = ceil(cbrt(n))` (at least 2) leaves fewer
//! than `n`, one more round with three times that follows; then the list ends
//! short.

use std::time::{Duration, Instant};

use super::groups;
use super::solver::DwaltonSolver;
use super::wings::{self, Refused, Staged};
use crate::puzzle::{
    normalize, same_up_to_commuting, Facelets, Failure, Move, MoveSequence, PuzzleKind,
    SolveMetrics, SolveOptions, SolveResult,
};

/// The solutions found so far, each with the stage marks of its moves.
struct Found {
    solutions: Vec<Vec<Move>>,
    marks: Vec<Vec<(&'static str, usize)>>,
    raw: Vec<Vec<Move>>,
}

/// The marks of the stages of `parts`, laid after `base` moves.
fn mark_parts(
    marks: &mut Vec<(&'static str, usize)>,
    base: usize,
    parts: &[(&'static str, &Staged)],
) {
    let mut at = base;
    for (name, staged) in parts {
        for end in &staged.ends {
            marks.push((name, at + end));
        }
        at += staged.moves.len();
        if marks.last().is_none_or(|&(_, end)| end != at) {
            marks.push((name, at));
        }
    }
}

impl DwaltonSolver {
    /// Up to `n` distinct solutions, the plain solution first.
    pub(super) fn list(
        &self,
        cube: &Facelets,
        options: &SolveOptions,
        n: usize,
    ) -> Vec<SolveResult> {
        let started = Instant::now();
        let deadline = options
            .time_budget
            .and_then(|budget| started.checked_add(budget));
        let failed = |failure: Failure, message: String| {
            vec![SolveResult {
                outcome: Err(failure),
                message: Some(message),
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    ..SolveMetrics::default()
                },
                stages: Vec::new(),
            }]
        };
        if let Some((failure, why)) = self.refuse(cube) {
            return failed(failure, why);
        }
        let (after, done) = if self.kind == PuzzleKind::Cube6 {
            match self.six().centers(cube, 40, deadline) {
                Some((c, d, true)) => (c, d),
                Some(_) => return failed(Failure::Internal, "step 60 found nothing".into()),
                None => (cube.clone(), Vec::new()),
            }
        } else {
            match self.seven().centers(cube, deadline) {
                Some(found) => found,
                None => (cube.clone(), Vec::new()),
            }
        };
        let timed_out = "the time limit was reached in the center stages";
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return failed(Failure::BudgetExhausted, timed_out.into());
        }
        if done.is_empty() {
            return failed(Failure::Internal, "the center stages found nothing".into());
        }
        let mut head: Vec<Move> = Vec::new();
        let mut head_marks: Vec<(&'static str, usize)> = Vec::new();
        for (name, names) in &done {
            let seq: MoveSequence = names.join(" ").parse().expect("catalog moves");
            head.extend_from_slice(seq.moves());
            head_marks.push((name, head.len()));
        }
        let left = || {
            deadline.map(|d| {
                d.saturating_duration_since(Instant::now())
                    .max(Duration::from_millis(1))
            })
        };
        let spent = || deadline.is_some_and(|d| Instant::now() >= d);
        let mut found = Found {
            solutions: Vec::new(),
            marks: Vec::new(),
            raw: Vec::new(),
        };
        let mut refusal: Option<Refused> = None;
        let root = (n as f64).cbrt().ceil() as usize;
        'rounds: for k in [root.max(2), 3 * root.max(2)] {
            let before = found.solutions.len();
            let wings1 = match self.wing_stage(&after, 0, k, left()) {
                Ok(list) => list,
                Err(r) => {
                    refusal = Some(r);
                    break;
                }
            };
            for first in &wings1 {
                let c1 = apply(&after, &first.moves);
                let wings2 = match self.wing_stage(&c1, 1, k, left()) {
                    Ok(list) => list,
                    Err(r) => {
                        refusal.get_or_insert(r);
                        break 'rounds;
                    }
                };
                for second in &wings2 {
                    let c2 = apply(&c1, &second.moves);
                    let tails = match wings::finish_many(&self.three, &c2, k, left()) {
                        Ok(list) => list,
                        Err(r) => {
                            refusal.get_or_insert(r);
                            break 'rounds;
                        }
                    };
                    for tail in &tails {
                        let mut raw = head.clone();
                        for part in [first, second, tail] {
                            raw.extend_from_slice(&part.moves);
                        }
                        let solution = normalize(self.kind.size(), &raw, false);
                        let new = found
                            .solutions
                            .iter()
                            .all(|g| !same_up_to_commuting(g, &solution));
                        if new {
                            let mut marks = head_marks.clone();
                            let names = self.stage_names();
                            let parts = [(names[0], first), (names[1], second), (names[2], tail)];
                            mark_parts(&mut marks, head.len(), &parts);
                            found.marks.push(marks);
                            found.solutions.push(solution);
                            found.raw.push(raw);
                        }
                        if found.solutions.len() >= n {
                            break 'rounds;
                        }
                    }
                    if spent() {
                        break 'rounds;
                    }
                }
            }
            if found.solutions.len() == before {
                break;
            }
        }
        if found.solutions.is_empty() {
            return match refusal {
                Some(r) => failed(
                    r.failure,
                    format!("{}: {}", r.stage, r.message.unwrap_or_default()),
                ),
                None => failed(Failure::Internal, "no solution was found".into()),
            };
        }
        let elapsed = started.elapsed();
        (0..found.solutions.len())
            .map(|i| {
                let solution = std::mem::take(&mut found.solutions[i]);
                let stages = groups::stages(cube, &found.raw[i], &found.marks[i], &solution);
                SolveResult {
                    outcome: Ok(MoveSequence::new(solution)),
                    message: None,
                    metrics: SolveMetrics {
                        elapsed,
                        ..SolveMetrics::default()
                    },
                    stages,
                }
            })
            .collect()
    }

    /// The names of the three stages after the centers.
    fn stage_names(&self) -> [&'static str; 3] {
        let inner = if self.kind == PuzzleKind::Cube6 {
            "inner wings (4x4x4)"
        } else {
            "inner wings (5x5x5)"
        };
        [inner, "outer wings (5x5x5)", "3x3x3"]
    }

    /// Up to `n` results of wing stage `which` (0 inner, 1 outer).
    fn wing_stage(
        &self,
        cube: &Facelets,
        which: usize,
        n: usize,
        budget: Option<Duration>,
    ) -> Result<Vec<Staged>, Refused> {
        match (which, self.kind) {
            (0, PuzzleKind::Cube6) => wings::inner_wings_many(&self.four, cube, n, budget),
            (0, _) => wings::orbit_wings_many(&self.five, cube, 1, n, budget),
            _ => wings::orbit_wings_many(&self.five, cube, 0, n, budget),
        }
    }
}

fn apply(cube: &Facelets, moves: &[Move]) -> Facelets {
    cube.apply(&MoveSequence::new(moves.to_vec()))
        .expect("the moves fit the cube")
}
