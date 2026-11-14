//! The front goal: nine front positions, each with the sticker identities it
//! may show, any whole-cube rotation, optionally the center upright.
//!
//! This is the public entry of the exact target-set search (`exact`) for a
//! goal that colors cannot express, such as a printed picture on the front
//! face. A caller gives:
//!
//! - a start cube (`Facelets` with the centers home) and its center twists;
//! - a [`FrontGoal`]: for each of the nine front positions (row-major, the
//!   facelets 18 to 26 of the standard URFDLB layout) the accepted sticker
//!   identities, as facelet numbers 0 to 53 of the solved cube (the home
//!   position of a sticker), and whether the center picture must stand upright
//!   (center twist 0 on the front, with the whole-cube rotation applied);
//! - [`FrontLimits`]: a time budget, a node budget and a move limit.
//!
//! The goal is met when some whole-cube rotation puts, at every front position,
//! an accepted sticker, and the center twist rule holds. The 24 rotations cost
//! no moves. The result is a shortest route in the half-turn metric and the
//! rotation that was applied at the end ([`FrontRoute::rotation`], a word of
//! `x y z` turns for [`crate::puzzle::rotation_moves`], empty for none); when
//! several rotations fit, the first in the order of [`crate::puzzle::rotation_words`].
//!
//! Failures: [`FrontFailure::BudgetExhausted`] when the time or the node budget
//! ends the search first (no partial route is returned; `nodes` says how far it
//! got); [`FrontFailure::NoRoute`] when no route within the move limit exists or
//! no state meets the goal; [`FrontFailure::InvalidInput`] for a bad goal, a
//! start that is no 3x3x3 or no legal cube, or twists that do not match it.
//!
//! The search builds about 44 MB of pruning tables per call (1.7 to 2.7 s on
//! one native core, measured on the calendar goals); nothing is cached. The
//! time budget covers the search only, not the table build. Visibility: only
//! the types and the function below are public; goal, search and tables stay
//! private to `partial`.

use std::time::Duration;

use super::super::cubie::Cubie;
use super::exact::{corner_parity_even, solve_pos_counted};
use super::goal::{Goal, Masks, EVERY_STICKER};
use super::model::Pos;
use crate::puzzle::{
    center_rotations, rotation_maps, rotation_words, CenterTwists, Facelets, Failure, MoveSequence,
    PuzzleKind, SolveOptions, FREE_TWIST,
};

/// Facelet of the first front position.
const FRONT: usize = 18;

/// What the front must show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontGoal {
    accepted: [u64; 9],
    center_up: bool,
}

/// Why a [`FrontGoal`] was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontGoalError(pub String);

impl FrontGoal {
    /// `accepted[i]` lists the sticker identities (facelet numbers 0 to 53 of
    /// the solved cube) allowed at front position `i`; exactly nine lists, none
    /// empty. `center_up` asks for the center picture upright.
    pub fn new(accepted: &[&[u8]], center_up: bool) -> Result<FrontGoal, FrontGoalError> {
        if accepted.len() != 9 {
            return Err(FrontGoalError(format!(
                "a front goal has nine positions, not {}",
                accepted.len()
            )));
        }
        let mut masks = [0u64; 9];
        for (i, list) in accepted.iter().enumerate() {
            if list.is_empty() {
                return Err(FrontGoalError(format!("position {i} accepts no sticker")));
            }
            for &id in *list {
                if id >= 54 {
                    return Err(FrontGoalError(format!(
                        "position {i}: sticker identity {id} is not below 54"
                    )));
                }
                masks[i] |= 1 << id;
            }
        }
        Ok(FrontGoal {
            accepted: masks,
            center_up,
        })
    }
}

/// The budgets of one search. The default is the search's own: 60 s, no node
/// cap, at most 21 moves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrontLimits {
    time: Option<Duration>,
    max_nodes: Option<u64>,
    max_moves: Option<u8>,
}

impl FrontLimits {
    /// Stops the search (not the table build) after `time`.
    pub fn with_time(mut self, time: Duration) -> FrontLimits {
        self.time = Some(time);
        self
    }

    /// Stops the search when it would visit more than `nodes` nodes.
    pub fn with_max_nodes(mut self, nodes: u64) -> FrontLimits {
        self.max_nodes = Some(nodes);
        self
    }

    /// Looks for routes of at most `moves` moves.
    pub fn with_max_moves(mut self, moves: u8) -> FrontLimits {
        self.max_moves = Some(moves);
        self
    }
}

/// A shortest route and the rotation that completes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontRoute {
    /// Face turns only, shortest in the half-turn metric.
    pub moves: MoveSequence,
    /// The whole-cube rotation to apply after `moves` (`x y z` word, empty for none).
    pub rotation: String,
    /// Search nodes visited.
    pub nodes: u64,
    /// Time to build the pruning tables.
    pub tables: Duration,
}

/// Why no route was returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontFailure {
    /// The input is unusable; the text says why.
    InvalidInput(String),
    /// No state meets the goal, or none within the move limit.
    NoRoute(String),
    /// The time or node budget ended the search; no route is returned.
    BudgetExhausted {
        /// Nodes visited when it stopped.
        nodes: u64,
    },
}

/// Finds a shortest route from `start` to `goal`, with the rotation that
/// completes it. See the module documentation.
pub fn solve_front_goal(
    start: &Facelets,
    twists: &CenterTwists,
    goal: &FrontGoal,
    limits: &FrontLimits,
) -> Result<FrontRoute, FrontFailure> {
    let invalid = |text: &str| Err(FrontFailure::InvalidInput(text.to_owned()));
    if start.kind() != PuzzleKind::Cube3 || twists.kind() != PuzzleKind::Cube3 {
        return invalid("expected a 3x3x3 start and its twists");
    }
    let mut cube = Cubie::new();
    if cube.set_from_facelet(&start.to_string()) != 0 || cube.verify() != 0 {
        return invalid("the start cube state is impossible");
    }
    if twists.sum_is_even() != corner_parity_even(&cube) {
        return invalid("the center twists do not belong to this cube: their sum must have the parity of the corner permutation");
    }
    let mut masks: Masks = [EVERY_STICKER; 54];
    masks[FRONT..FRONT + 9].copy_from_slice(&goal.accepted);
    let mut twist = FREE_TWIST;
    if goal.center_up {
        twist[2] = 0b0001;
    }
    let rotations: Vec<_> = rotation_maps(PuzzleKind::Cube3)
        .into_iter()
        .zip(center_rotations())
        .collect();
    let whole = Goal::rotated_twisted(&masks, &twist, &rotations);
    if whole.is_impossible() {
        return Err(FrontFailure::NoRoute(
            "no state meets the front goal in any rotation".to_owned(),
        ));
    }
    let options = SolveOptions {
        time_budget: limits.time,
        max_moves: limits.max_moves,
        ..SolveOptions::default()
    };
    let pos = Pos::from_cubie(&cube).with_twist(*twists);
    let run = solve_pos_counted(
        crate::puzzle::Instant::now(),
        pos,
        &whole,
        &options,
        limits.max_nodes,
    );
    match run.result.outcome {
        Ok(moves) => {
            let end = run.end.expect("a found route has an end position");
            let words = rotation_words();
            let rotation = rotations
                .iter()
                .position(|r| {
                    Goal::rotated_twisted(&masks, &twist, std::slice::from_ref(r)).is_met(&end)
                })
                .map(|i| words[i].clone())
                .expect("the end meets the goal in some rotation");
            Ok(FrontRoute {
                moves,
                rotation,
                nodes: run.nodes,
                tables: run.tables,
            })
        }
        Err(Failure::BudgetExhausted) => Err(FrontFailure::BudgetExhausted { nodes: run.nodes }),
        Err(_) => Err(FrontFailure::NoRoute(
            run.result.message.unwrap_or_default(),
        )),
    }
}
