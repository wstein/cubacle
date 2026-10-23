//! Recovering a cube after a mistake in a printed solution.
//!
//! A person follows a solution move by move, goes wrong somewhere and holds a
//! cube that is not on the path any more. [`diagnose`] says where the cube is
//! on the path, or which mistakes most plausibly led off it, and finds the
//! moves that bring the cube back; [`Plan`] weighs that against solving the
//! cube again from scratch and writes the advice.
//!
//! Every cube size works the same way: a move is a permutation of the stickers,
//! and the search composes those permutations along the solution. The module
//! depends only on [`crate::puzzle`]; the fresh solution is passed in.

mod plan;
mod resume;
mod search;
mod state;
mod variants;

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::puzzle::{Facelets, Move, MoveSequence, PuzzleKind};

pub use plan::{Advice, Plan, PlanText};

use search::{Explained, Frame};
use state::{permute, rotations, state_of, MoveId, Perms, State};
use variants::{Breadth, Variant};

/// How far the search goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The most moves the search for a way back onto the path may use
    /// (0 to 5; 3 takes milliseconds, 5 seconds and about 130 MB on a 7x7x7).
    pub resume_depth: usize,
    /// The most stray moves after one mistake (0 to 3; 2 takes a fraction of a
    /// second on a 7x7x7). Short solutions use at most one, because on a 2x2x2
    /// more would explain cubes by chance.
    pub strays: usize,
    /// Also look for two mistakes (with the likely kinds only).
    pub two_mistakes: bool,
    /// For two mistakes, try every move and every extra move too: slow and
    /// memory-hungry on the big cubes.
    pub thorough: bool,
    /// Stop searching after this long and report what was found.
    pub max_time: Option<Duration>,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            resume_depth: 3,
            strays: 2,
            two_mistakes: true,
            thorough: false,
            max_time: Some(Duration::from_secs(8)),
        }
    }
}

/// Why nothing could be diagnosed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverError {
    /// The two cubes have different sizes.
    SizeMismatch(PuzzleKind, PuzzleKind),
    /// A move of the solution does not exist on this cube.
    BadMove(Move),
    /// Following the solution from the original cube does not solve it.
    NotASolution,
}

impl fmt::Display for RecoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoverError::SizeMismatch(a, b) => write!(
                f,
                "the original cube is a {0}x{0}x{0} and the current one a {1}x{1}x{1}",
                a.size(),
                b.size()
            ),
            RecoverError::BadMove(mv) => write!(f, "the solution has a move {mv} this cube lacks"),
            RecoverError::NotASolution => {
                write!(f, "the solution does not solve the original cube")
            }
        }
    }
}

impl std::error::Error for RecoverError {}

/// What went wrong at one step of the printed solution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MistakeKind {
    /// The move was not done.
    Skipped,
    /// A quarter turn instead of a half turn, or the other way round.
    WrongAmount { done: Move },
    /// Turned the wrong way.
    WrongDirection { done: Move },
    /// The right face and turn, but other layers.
    WrongLayers { done: Move },
    /// The right layers and turn, but another face.
    WrongFace { done: Move },
    /// Some other move instead.
    Different { done: Move },
    /// The move was done after the one that follows it.
    Swapped { next: Move },
    /// The move was done twice.
    Repeated,
    /// A move that is not in the solution, before the printed move (or after
    /// the last one when `printed` is `None`).
    Extra { done: Move },
}

/// One mistake: the step (counting from 1) and the move printed there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mistake {
    pub step: usize,
    pub printed: Option<Move>,
    pub kind: MistakeKind,
}

/// What the current cube is, against the printed solution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// The cube is exactly after `done` printed moves.
    OnPath { done: usize },
    /// One or two mistakes explain the cube; `done` printed moves had been
    /// used up when the cube was entered. `alternatives` other explanations
    /// fit equally well (commuting moves make the exact step ambiguous); the
    /// earliest is given.
    Mistakes {
        mistakes: Vec<Mistake>,
        done: usize,
        /// Stray moves made after those `done` steps (up to two).
        strays: Vec<Move>,
        alternatives: usize,
    },
    /// No simple explanation, but the cube is `away` moves from the cube after
    /// `done` printed moves.
    Near { done: usize, away: usize },
    /// Nothing close enough was found.
    Unexplained,
}

/// Moves that bring the cube back onto the printed path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub moves: MoveSequence,
    /// After the route, the cube is after this many printed moves.
    pub continue_after: usize,
    /// The route takes back the moves actually done, rather than correcting.
    pub undo: bool,
}

/// The result of [`diagnose`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    pub finding: Finding,
    /// When only a whole-cube rotation made the cube fit: the `x y z` turns
    /// that rotate the cube into the orientation of the original.
    pub rotation: Option<String>,
    /// Ways back onto the path, in the orientation of the original cube.
    pub routes: Vec<Route>,
    /// The number of moves in the printed solution.
    pub solution_len: usize,
    /// The time limit cut the search short.
    pub timed_out: bool,
}

impl Route {
    /// The moves to do plus the printed moves left after them.
    pub fn total(&self, solution_len: usize) -> usize {
        self.moves.len() + solution_len - self.continue_after
    }
}

/// The number of printed moves after which `current` is exactly the cube, if
/// it is on the path in its own orientation. Cheaper than [`diagnose`].
pub fn locate(
    original: &Facelets,
    current: &Facelets,
    solution: &MoveSequence,
) -> Result<Option<usize>, RecoverError> {
    let kind = original.kind();
    if current.kind() != kind {
        return Err(RecoverError::SizeMismatch(kind, current.kind()));
    }
    let mut cube = original.clone();
    for (done, mv) in std::iter::once(None)
        .chain(solution.moves().iter().map(Some))
        .enumerate()
    {
        if let Some(mv) = mv {
            cube = cube
                .apply(&MoveSequence::new(vec![*mv]))
                .map_err(|e| RecoverError::BadMove(e.mv))?;
        }
        if cube == *current {
            return Ok(Some(done));
        }
    }
    Ok(None)
}

/// Finds out where `current` is on the way from `original` along `solution`.
pub fn diagnose(
    original: &Facelets,
    current: &Facelets,
    solution: &MoveSequence,
    limits: &Limits,
) -> Result<Diagnosis, RecoverError> {
    let kind = original.kind();
    if current.kind() != kind {
        return Err(RecoverError::SizeMismatch(kind, current.kind()));
    }
    if let Some(mv) = solution.moves().iter().find(|m| !m.fits(kind)) {
        return Err(RecoverError::BadMove(*mv));
    }
    let outer_only = solution.moves().iter().all(|m| m.depth == 1 && !m.wide);
    let perms = Perms::new(kind, !outer_only);
    let printed: Vec<MoveId> = solution
        .moves()
        .iter()
        .map(|m| perms.id(m).expect("in the universe"))
        .collect();
    let start = state_of(original);
    let mut trajectory = vec![start];
    for &id in &printed {
        let next = perms.apply(trajectory.last().expect("start"), id);
        trajectory.push(next);
    }
    // A 2x2x2 solution may end turned: only the other sizes are checked.
    if kind != PuzzleKind::Cube2 && trajectory[printed.len()] != state_of(&Facelets::solved(kind)) {
        return Err(RecoverError::NotASolution);
    }
    let started = Instant::now();
    let ctx = Context {
        any: variants::all(&printed, &perms, Breadth::Any),
        likely: variants::all(&printed, &perms, Breadth::Likely),
        perms: &perms,
        printed: &printed,
        trajectory: &trajectory,
        deadline: limits.max_time.map(|t| started + t),
        limits,
    };
    let current = state_of(current);
    let turns = rotations(kind);
    let mut found = ctx.explain(&current, true, limits.resume_depth);
    let mut rotation = None;
    if found.is_none() {
        let turned: Vec<State> = turns
            .iter()
            .map(|r| {
                let mut out = current.clone();
                permute(&mut out, &current, &r.perm);
                out
            })
            .collect();
        // Cheap searches in every orientation first, the two-mistake search after.
        for double in [false, true] {
            if double && !limits.two_mistakes {
                break;
            }
            if let Some((i, f)) = ctx.first_found(&turned[1..], double) {
                rotation = Some(turns[i + 1].word.clone());
                found = Some(f);
                break;
            }
        }
    }
    let timed_out = ctx.expired() && found.is_none();
    let (finding, routes) = match found {
        Some(f) => (f.finding, f.routes),
        None => (Finding::Unexplained, Vec::new()),
    };
    Ok(Diagnosis {
        finding,
        rotation,
        routes,
        solution_len: printed.len(),
        timed_out,
    })
}

struct Found {
    finding: Finding,
    routes: Vec<Route>,
}

struct Context<'a> {
    /// The variants of every kind, and the likely ones only.
    any: Vec<Vec<Variant>>,
    likely: Vec<Vec<Variant>>,
    perms: &'a Perms,
    printed: &'a [MoveId],
    trajectory: &'a [State],
    deadline: Option<Instant>,
    limits: &'a Limits,
}

impl Context<'_> {
    fn expired(&self) -> bool {
        self.deadline.is_some_and(|d| Instant::now() >= d)
    }

    fn frame(&self, current: &State) -> Frame<'_> {
        Frame {
            perms: self.perms,
            printed: self.printed,
            trajectory: self.trajectory,
            current: current.clone(),
            deadline: self.deadline,
        }
    }

    /// Every search in one orientation, the cheap ones first.
    fn explain(&self, current: &State, with_double: bool, depth: usize) -> Option<Found> {
        let frame = self.frame(current);
        if let Some(done) = search::on_path(&frame) {
            return Some(Found {
                finding: Finding::OnPath { done },
                routes: Vec::new(),
            });
        }
        let fix = resume::search(&frame, depth);
        let mut explained = search::single(&frame, &self.any);
        if explained.is_none() && self.limits.strays > 0 {
            let strays = self
                .limits
                .strays
                .min(if self.printed.len() < 16 { 1 } else { 3 });
            explained = search::with_strays(&frame, &self.likely, strays);
        }
        if explained.is_none() && with_double && self.limits.two_mistakes {
            explained = self.double(&frame);
        }
        Some(self.found(explained, fix)).filter(|f| !matches!(f.finding, Finding::Unexplained))
    }

    /// One pass over a rotated cube: without (`false`) or with (`true`) the
    /// two-mistake search.
    fn explain_pass(&self, current: &State, double: bool) -> Option<Found> {
        // A turned cube is searched with a shallow resume only.
        let depth = self.limits.resume_depth.min(3);
        if !double {
            return self.explain(current, false, depth);
        }
        let frame = self.frame(current);
        let explained = self.double(&frame)?;
        Some(self.found(Some(explained), resume::search(&frame, depth)))
    }

    /// The first orientation in `states` (lowest index) with a finding, the
    /// orientations being searched on several threads.
    fn first_found(&self, states: &[State], double: bool) -> Option<(usize, Found)> {
        let workers = thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(8);
        let next = AtomicUsize::new(0);
        let best = AtomicUsize::new(usize::MAX);
        let results = Mutex::new(Vec::new());
        thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= states.len() || i > best.load(Ordering::Relaxed) || self.expired() {
                        break;
                    }
                    if let Some(f) = self.explain_pass(&states[i], double) {
                        best.fetch_min(i, Ordering::Relaxed);
                        results.lock().expect("no panic").push((i, f));
                    }
                });
            }
        });
        let found = results.into_inner().expect("no panic");
        found.into_iter().min_by_key(|(i, _)| *i)
    }

    fn double(&self, frame: &Frame) -> Option<Explained> {
        let variants = if self.limits.thorough {
            &self.any
        } else {
            &self.likely
        };
        search::double(frame, variants)
    }

    fn found(&self, explained: Option<Explained>, fix: Option<resume::Fix>) -> Found {
        let mut routes = Vec::new();
        let mut finding = Finding::Unexplained;
        if let Some(e) = &explained {
            finding = Finding::Mistakes {
                mistakes: e.mistakes.clone(),
                done: e.done,
                strays: e.strays.iter().map(|&id| self.perms.mv(id)).collect(),
                alternatives: e.alternatives,
            };
            routes.push(self.route(&self.perms.undo_of(&e.performed), e.branch, true));
        }
        if let Some(fix) = fix {
            if explained.is_none() {
                finding = Finding::Near {
                    done: fix.done,
                    away: fix.moves.len(),
                };
            }
            routes.push(self.route(&fix.moves, fix.done, false));
        }
        Found { finding, routes }
    }

    fn route(&self, ids: &[MoveId], continue_after: usize, undo: bool) -> Route {
        Route {
            moves: ids.iter().map(|&id| self.perms.mv(id)).collect(),
            continue_after,
            undo,
        }
    }
}
