//! The shortest known generator of a cube state: the moves that build the
//! state from the solved cube.
//!
//! Two searches run one after the other, both under the same memory cap:
//!
//! 1. the macro search (see [`PatternSearch`]): the cube as a product of up to
//!    four pattern macros, found by meeting in the middle;
//! 2. the exact search of `bidir`: a bidirectional search over plain moves
//!    (the face turns on the 3x3x3; every single layer and every outer block
//!    on bigger cubes) that looks for a sequence *shorter* than the first
//!    answer, or for any sequence when the first search found none. When it
//!    finishes without a shorter one, the answer is the shortest sequence of
//!    those moves.
//!
//! A search that hits the cap says so: "nothing found within the cap" is not
//! "no generator exists".

use std::time::{Duration, Instant};

use super::bidir::{self, Limits, Outcome};
use super::geometry;
use super::steps;
use super::{Cap, PatternSearch, Probe};
use crate::puzzle::{Facelets, LayerMove, Move, MoveSequence, Turn};

/// A generator and what is known about its length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generator {
    /// Moves that turn the solved cube into the state.
    pub moves: MoveSequence,
    /// No generator is shorter in the moves of the exact search (exhaustively
    /// checked): the face turns on the 3x3x3, every plain move on bigger cubes.
    pub proven: bool,
    /// What limited the search, when `proven` is false.
    pub note: Option<String>,
}

/// Why nothing was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotFound {
    /// The memory cap stopped a search before it finished.
    Cap(String),
    /// The time budget stopped a search.
    Time(String),
    /// The search space was searched out: no product of macros and no short
    /// short sequence of plain moves builds the state.
    Searched(String),
}

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotFound::Cap(text) | NotFound::Time(text) | NotFound::Searched(text) => {
                f.write_str(text)
            }
        }
    }
}

fn inverse(moves: &MoveSequence) -> MoveSequence {
    MoveSequence::new(
        moves
            .moves()
            .iter()
            .rev()
            .map(|mv| {
                let turn = match mv.turn {
                    Turn::Clockwise => Turn::CounterClockwise,
                    Turn::Half => Turn::Half,
                    Turn::CounterClockwise => Turn::Clockwise,
                };
                Move::new(mv.face, mv.depth, mv.wide, turn)
            })
            .collect(),
    )
}

/// Bytes one state of the exact search costs, with the room that growing
/// vectors and hash maps need while they are rebuilt.
fn state_bytes(len: usize) -> usize {
    2 * len + 112
}

/// The shortest generator of `state` that the searches find within `cap` and
/// `time`.
pub fn find_generator(
    state: &Facelets,
    cap: Cap,
    time: Option<Duration>,
) -> Result<Generator, NotFound> {
    find_generator_with(state, cap, time, None)
}

/// Like [`find_generator`], with every move of the answer taken from `allow`
/// (all moves when `None`). A macro is used only when all its moves and those
/// of its inverse are allowed.
pub fn find_generator_with(
    state: &Facelets,
    cap: Cap,
    time: Option<Duration>,
    allow: Option<&dyn Fn(&Move) -> bool>,
) -> Result<Generator, NotFound> {
    let started = Instant::now();
    let kind = state.kind();
    if state.is_solved() {
        return Ok(Generator {
            moves: MoveSequence::default(),
            proven: true,
            note: None,
        });
    }
    let deadline = time.map(|t| started + t);
    // 1. Products of macros.
    let mut macro_note = None;
    let mut best: Option<MoveSequence> = None;
    if kind.size() >= 4 {
        let search = PatternSearch::new(kind, cap, allow).map_err(NotFound::Searched)?;
        match search.solve(state, deadline) {
            Probe::Found(solution) => best = Some(inverse(&solution)),
            Probe::Missing { complete } => {
                if !complete {
                    macro_note = Some(format!(
                        "the macro search was cut short ({})",
                        search.limit_text(cap)
                    ));
                }
            }
        }
    }
    // 2. Outer blocks, shorter than the first answer.
    let n = kind.size();
    let mut set = if n == 3 {
        steps::plain(n, false, false, true)
    } else {
        steps::layer_moves(n, false)
    };
    if let Some(allow) = allow {
        let keep: Vec<bool> = set
            .text
            .iter()
            .map(|word| {
                word.iter()
                    .flat_map(LayerMove::to_moves)
                    .all(|mv| allow(&mv))
            })
            .collect();
        let mut flags = keep.iter();
        set.steps.retain(|_| *flags.next().unwrap_or(&false));
        let mut flags = keep.iter();
        set.text.retain(|_| *flags.next().unwrap_or(&false));
        if set.steps.is_empty() {
            return Err(NotFound::Searched(
                "the move set has no move this search can use".to_owned(),
            ));
        }
    }
    let len = n * n * 6;
    let max_states = cap.max_bytes / state_bytes(len);
    let max_cost = best.as_ref().map_or(
        match n {
            3 => 14,
            _ => 10,
        },
        |g| g.len().saturating_sub(1),
    );
    let remaining = deadline.map(|d| d.saturating_duration_since(Instant::now()));
    let limits = Limits {
        max_cost,
        time: remaining.unwrap_or(Duration::from_secs(3600)),
        max_states,
    };
    let target = geometry::colours(state);
    let solved = geometry::colours(&Facelets::solved(kind));
    let outcome = bidir::search(&set.steps, vec![solved], vec![target], &limits);
    let mut proven = false;
    let mut exact_note = None;
    match outcome {
        Outcome::Found(path, _) => {
            let layers: Vec<LayerMove> = path.iter().flat_map(|&i| set.text[i].clone()).collect();
            let plain: Vec<Move> = layers.iter().flat_map(LayerMove::to_moves).collect();
            let found = MoveSequence::new(plain);
            let built = Facelets::solved(kind).apply(&found).map_err(|e| {
                NotFound::Searched(format!("internal error: the exact search failed ({e})"))
            })?;
            if &built == state {
                proven = true;
                best = Some(found);
            } else {
                exact_note = Some("the exact search gave a wrong answer, ignored".to_owned());
            }
        }
        Outcome::NotFound(reason) => {
            if reason.starts_with("none within cost") {
                proven = true;
                if best.is_none() {
                    return Err(NotFound::Searched(format!(
                        "no generator of up to {max_cost} moves, and no product of macros, builds this state"
                    )));
                }
            } else if reason.starts_with("budget") {
                let timed = deadline.is_some_and(|d| Instant::now() >= d);
                let text = if timed {
                    "the time budget ran out in the exact search".to_owned()
                } else {
                    format!(
                        "the exact search reached the memory cap ({} MB)",
                        cap.max_bytes >> 20
                    )
                };
                exact_note = Some(text);
            } else {
                exact_note = Some(reason);
            }
        }
    }
    match best {
        Some(moves) => Ok(Generator {
            moves,
            proven,
            note: if proven {
                None
            } else {
                exact_note.or(macro_note)
            },
        }),
        None => {
            let text = format!(
                "nothing found within the cap of {} MB ({}); this does not mean the state has no short generator",
                cap.max_bytes >> 20,
                exact_note.unwrap_or_else(|| "no product of macros builds it".to_owned())
            );
            if text.contains("time budget") {
                Err(NotFound::Time(text))
            } else {
                Err(NotFound::Cap(text))
            }
        }
    }
}
