//! The exact route: a proven shortest answer to a target set.
//!
//! Used on request (`SolveOptions::exact`) and always for a free orientation.
//! See `docs/design/dont-care-targets.md`, "Target sets on the 3x3x3".

use std::time::Duration;

use crate::puzzle::Instant;

use super::super::cubie::Cubie;
use super::goal::{color_masks, Goal};
use super::ida::{Bound, Ended, Search};
use super::model::{Pos, Turns};
use super::tables::{plan, Built, CornerTable, EdgeTable};
use super::twist::TwistTable;
use super::{finish, proven};
use crate::puzzle::{
    center_rotations, rotation_maps, CenterTwists, Facelets, Failure, MoveSequence, PuzzleKind,
    SolveOptions, SolveResult, Target,
};

/// Megabytes for the tables when the caller sets no cap.
const DEFAULT_MEMORY_MB: usize = 1500;
/// Milliseconds for the search when the caller sets no budget.
const DEFAULT_BUDGET_MS: u64 = 60_000;
/// The deepest answer looked for when the caller sets no move limit.
const DEFAULT_MAX_MOVES: usize = 21;

pub(super) fn by_goal_set(
    started: Instant,
    start: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> SolveResult {
    let mut cube = Cubie::new();
    if cube.set_from_facelet(&start.to_string()) != 0 || cube.verify() != 0 {
        return finish(
            started,
            Err(Failure::InvalidInput),
            "the start cube state is impossible".into(),
        );
    }
    let masks = color_masks(target.sets());
    let twist = target.twist_masks();
    let (goal, frame) = if options.free_orientation && target.has_twist_condition() {
        let rotations: Vec<_> = rotation_maps(PuzzleKind::Cube3)
            .into_iter()
            .zip(center_rotations())
            .collect();
        (
            Goal::rotated_twisted(&masks, twist, &rotations),
            crate::puzzle::SolveFrame::AnyOrientation,
        )
    } else if options.free_orientation {
        (
            Goal::rotated(&masks, &rotation_maps(PuzzleKind::Cube3)),
            crate::puzzle::SolveFrame::AnyOrientation,
        )
    } else {
        (
            Goal::with_twists(&[(masks, *twist)]),
            crate::puzzle::SolveFrame::Fixed,
        )
    };
    if goal.is_impossible() {
        let why = if options.free_orientation {
            "no orientation of the cube fits the centers of the target"
        } else {
            "the target wants another color on a center, and the centers of a 3x3x3 never move"
        };
        return finish(
            started,
            Err(Failure::Exhausted),
            format!("unreachable: {why}"),
        );
    }
    let twisted = options.start_twist.is_some() || target.has_twist_condition();
    let twist_start = options
        .start_twist
        .or_else(|| CenterTwists::untwisted(PuzzleKind::Cube3))
        .expect("a 3x3x3 has fixed centers");
    if twisted && twist_start.sum_is_even() != corner_parity_even(&cube) {
        return finish(
            started,
            Err(Failure::InvalidInput),
            "the center twist does not belong to this cube: the sum of the six twists must have the parity of the corner permutation (even for an even permutation)".into(),
        );
    }
    let pos = Pos::from_cubie(&cube).with_twist(twist_start);
    let mut result = solve_pos(started, pos, &goal, options);
    result.metrics.evidence.frame = frame;
    result
}

/// Whether the corner permutation of `cube` is even.
fn corner_parity_even(cube: &Cubie) -> bool {
    let perm: Vec<usize> = cube.ca.iter().map(|&c| usize::from(c & 7)).collect();
    let mut seen = [false; 8];
    let mut swaps = 0;
    for i in 0..8 {
        let (mut j, mut length) = (i, 0);
        while !seen[j] {
            seen[j] = true;
            j = perm[j];
            length += 1;
        }
        swaps += length.max(1) - 1;
    }
    swaps % 2 == 0
}

/// Searches `goal` from `cube` (the entry of the tests).
#[cfg(test)]
pub(super) fn solve_goal(
    started: Instant,
    cube: &Cubie,
    goal: &Goal,
    options: &SolveOptions,
) -> SolveResult {
    solve_pos(started, Pos::from_cubie(cube), goal, options)
}

/// Searches `goal` from `start`.
fn solve_pos(started: Instant, start: Pos, goal: &Goal, options: &SolveOptions) -> SolveResult {
    let turns = Turns::new();
    if goal.is_met(&start) {
        return proven(
            started,
            MoveSequence::new(Vec::new()),
            "shortest: 0 moves, the start already meets the target".into(),
        );
    }
    let budget_mb = options.max_memory_mb.unwrap_or(DEFAULT_MEMORY_MB);
    let (with_corners, pieces) = plan(goal, budget_mb.saturating_mul(1 << 20));
    let built = Instant::now();
    let unreachable = |note: &str| {
        finish(
            started,
            Err(Failure::Exhausted),
            format!("unreachable: no state meets the target ({note} cannot be met by any cube)"),
        )
    };
    let corners = if with_corners {
        match CornerTable::build(goal, &turns) {
            Built::Table(table) => Some(table),
            Built::NothingToPrune => None,
            Built::Unsatisfiable => return unreachable("the corners"),
        }
    } else {
        None
    };
    let edges = if pieces.is_empty() {
        None
    } else {
        match EdgeTable::build(goal, &turns, &pieces) {
            Built::Table(table) => Some(table),
            Built::NothingToPrune => None,
            Built::Unsatisfiable => return unreachable("the tracked edges"),
        }
    };
    let twist = TwistTable::build(goal);
    let table_bytes = corners.as_ref().map_or(0, |_| CornerTable::bytes())
        + edges.as_ref().map_or(0, |t| EdgeTable::bytes(t.tracked()));
    let table_note = format!(
        "{} MB of tables: {}, {}{}, built in {:.1} s",
        table_bytes >> 20,
        if corners.is_some() {
            "all corners".to_owned()
        } else {
            "no corner table".to_owned()
        },
        match &edges {
            Some(t) => format!("{} edges", t.tracked()),
            None => "no edge table".to_owned(),
        },
        if twist.is_some() {
            ", the center twists"
        } else {
            ""
        },
        built.elapsed().as_secs_f64()
    );
    let bound = Bound {
        corners,
        edges,
        twist,
    };
    let budget = options
        .time_budget
        .unwrap_or(Duration::from_millis(DEFAULT_BUDGET_MS));
    let max = options.max_moves.map_or(DEFAULT_MAX_MOVES, usize::from);
    let mut search = Search::new(&turns, goal, &bound, Instant::now() + budget);
    let from = search.start_bound(&start);
    let ended = search.run(&start, from, max);
    let nodes = search.nodes;
    match ended {
        Ended::Found(path) => {
            let solution: MoveSequence = path.iter().map(|&m| turns.moves[m]).collect();
            proven(
                started,
                solution,
                format!(
                    "shortest: proved by an iterative-deepening search to the target set ({nodes} nodes; {table_note})"
                ),
            )
        }
        Ended::Empty(depth) => finish(
            started,
            Err(Failure::Exhausted),
            format!(
                "no solution within {depth} moves: searched out, so the shortest answer is longer ({nodes} nodes; {table_note}); raise --max-moves"
            ),
        ),
        Ended::Stopped(depth) => {
            finish(
                started,
                Err(Failure::BudgetExhausted),
                format!(
                    "not proven: no solution has fewer than {depth} moves, but the search of {depth} moves did not finish in {} ms ({nodes} nodes; {table_note}); the target may be too loose for the tables: give more time with -b or more memory with --max-memory",
                    budget.as_millis()
                ),
            )
        }
    }
}
