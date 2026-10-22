use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::cubie::Cubie;
use super::reduction::{reduce, Stuck};
use super::tables::{Config, Tables};
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver, Turn,
};
use crate::reduction::cache::default_dir;
use crate::reduction::par;

/// The longest 3x3x3 hand-off the search is asked for.
const THREE_MAX_MOVES: u8 = 25;

/// What the 3x3x3 stage may take when the caller sets no time budget. A tight
/// move limit can make the 3x3x3 search run for minutes; it ends as `Exhausted`.
const HAND_OFF_BUDGET: Duration = Duration::from_secs(10);

/// The 5x5x5 reduction solver, a port of the five-phase solver cube555.
///
/// Five phases bring the centres and wings into place (about 52 moves), then
/// the solver it was built with solves the 3x3x3 that is left (about 22). The
/// 3x3x3 solver is injected through the [`Solver`] trait, so any engine (or a
/// stub) will do.
///
/// The pruning tables (237 MB) belong to the instance and are shared by
/// its clones. They are read from the cache directory
/// (`CUBACLE_CACHE_DIR`, else `$XDG_CACHE_HOME/cubacle`, else
/// `~/.cache/cubacle`) or built on first use or by [`Solver::prepare`]: a few
/// seconds on many cores, tens of seconds on one, once per machine. A solve
/// that arrives before the tables exist builds them first and counts that time
/// against its budget. The number of threads is `CUBACLE_THREADS` or the
/// available parallelism, for the table build and the search; without a time
/// budget the solution does not depend on it.
#[derive(Clone)]
pub struct FiveSolver {
    three: Arc<dyn Solver + Send + Sync>,
    tables: Arc<OnceLock<Tables>>,
    config: Config,
}

impl fmt::Debug for FiveSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FiveSolver")
            .field("tables_ready", &self.tables_ready())
            .field("threads", &self.config.threads)
            .field("cache_dir", &self.config.cache_dir)
            .finish()
    }
}

impl FiveSolver {
    /// A solver that finishes with `three`, which must handle the 3x3x3.
    pub fn new(three: Arc<dyn Solver + Send + Sync>) -> FiveSolver {
        FiveSolver {
            three,
            tables: Arc::new(OnceLock::new()),
            config: Config {
                threads: par::threads(),
                cache_dir: default_dir(),
                cap: 15,
            },
        }
    }

    /// Uses `threads` workers instead of the configured number.
    pub fn with_threads(mut self, threads: usize) -> FiveSolver {
        self.config.threads = threads.max(1);
        self
    }

    /// Keeps the table files in `dir` (none: do not cache), instead of the
    /// default directory. Tables built already are dropped.
    pub fn with_cache_dir(mut self, dir: Option<PathBuf>) -> FiveSolver {
        self.config.cache_dir = dir;
        self.tables = Arc::new(OnceLock::new());
        self
    }

    /// Small tables for tests: layers only down to `cap`.
    #[cfg(test)]
    pub(super) fn with_cap(mut self, cap: u32) -> FiveSolver {
        self.config.cap = cap;
        self.config.cache_dir = None;
        self.tables = Arc::new(OnceLock::new());
        self
    }

    /// Whether the tables are in memory, so that a solve starts at once.
    pub fn tables_ready(&self) -> bool {
        self.tables.get().is_some()
    }

    /// The directory of the table files, if any.
    pub fn cache_dir(&self) -> Option<&Path> {
        self.config.cache_dir.as_deref()
    }

    /// After the tables are ready: whether some could not be saved to the
    /// cache directory (they are in memory, and will be built again next time).
    pub fn cache_write_failed(&self) -> Option<bool> {
        self.tables.get().map(|tables| tables.cache_write_failed)
    }

    /// Bytes held by the pruning tables, once they are in memory.
    pub fn table_bytes(&self) -> Option<usize> {
        self.tables.get().map(Tables::pruning_bytes)
    }

    /// The number of worker threads.
    pub fn threads(&self) -> usize {
        self.config.threads
    }

    fn tables(&self) -> &Tables {
        self.tables.get_or_init(|| Tables::build(&self.config))
    }
}

/// The colour index of every sticker (the face it belongs to), provided the
/// six face centres are where they started.
fn colours_of(facelets: &Facelets) -> Result<[u8; 150], &'static str> {
    let stickers = facelets.stickers();
    if (0..6).any(|face| stickers[face * 25 + 12] != Face::ALL[face]) {
        return Err(
            "the face centres have moved (a middle-slice turn?); the engine keeps them fixed",
        );
    }
    let mut out = [0u8; 150];
    for (slot, face) in out.iter_mut().zip(stickers) {
        *slot = face.index() as u8;
    }
    Ok(out)
}

/// A move number of the five-phase solver as a move.
fn move_of(code: u8) -> Move {
    let face = Face::ALL[usize::from(code % 18 / 3)];
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][usize::from(code % 3)];
    if code >= 18 {
        Move::new(face, 2, true, turn)
    } else {
        Move::new(face, 1, false, turn)
    }
}

fn quarter_turns(turn: Turn) -> u8 {
    match turn {
        Turn::Clockwise => 1,
        Turn::Half => 2,
        Turn::CounterClockwise => 3,
    }
}

/// Joins the reduction and the 3x3x3 solution, cancelling turns of the same
/// layer where they meet.
fn join(mut head: Vec<Move>, tail: &[Move]) -> Vec<Move> {
    for &next in tail {
        match head.last() {
            Some(last)
                if last.face == next.face && last.depth == next.depth && last.wide == next.wide =>
            {
                let total = (quarter_turns(last.turn) + quarter_turns(next.turn)) % 4;
                let last = head.pop().expect("a last move");
                let turn = match total {
                    1 => Some(Turn::Clockwise),
                    2 => Some(Turn::Half),
                    3 => Some(Turn::CounterClockwise),
                    _ => None,
                };
                if let Some(turn) = turn {
                    head.push(Move { turn, ..last });
                }
            }
            _ => head.push(next),
        }
    }
    head
}

/// The 3x3x3 hidden in rows and columns 0, 2 and 4 of a 5x5x5.
fn three_by_three(cube: &Facelets) -> Facelets {
    let mut out = Vec::with_capacity(54);
    for face in 0..6 {
        for row in [0, 2, 4] {
            for col in [0, 2, 4] {
                out.push(cube.stickers()[face * 25 + row * 5 + col]);
            }
        }
    }
    Facelets::from_stickers(out).expect("nine stickers of each colour")
}

impl Solver for FiveSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube5
    }

    fn prepare(&self) {
        self.tables();
        self.three.prepare();
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let done = |outcome: Result<MoveSequence, Failure>, message: Option<String>| SolveResult {
            outcome,
            message,
            metrics: SolveMetrics {
                elapsed: started.elapsed(),
                probes: None,
            },
        };
        let fail = |failure: Failure, message: &str| done(Err(failure), Some(message.to_owned()));

        if facelets.kind() != PuzzleKind::Cube5 {
            let size = facelets.kind().size();
            return fail(
                Failure::InvalidInput,
                &format!("expected a 5x5x5, got a {size}x{size}x{size}"),
            );
        }
        if self.three.kind() != PuzzleKind::Cube3 {
            return fail(
                Failure::Internal,
                "the solver for the 3x3x3 stage does not solve 3x3x3 cubes",
            );
        }
        let colours = match colours_of(facelets) {
            Ok(colours) => colours,
            Err(why) => return fail(Failure::InvalidInput, why),
        };
        let cube = match Cubie::from_colours(&colours) {
            Ok(cube) => cube,
            Err(why) => return fail(Failure::InvalidInput, why.describe()),
        };

        let deadline = options.time_budget.map(|budget| started + budget);
        let expired = |now: Instant| deadline.is_some_and(|d| now >= d);
        let cold = !self.tables_ready();
        let tables = self.tables();
        if expired(Instant::now()) {
            return fail(
                Failure::BudgetExhausted,
                if cold {
                    "building the 5x5x5 tables (first use; prepare() builds them ahead) used up the time budget"
                } else {
                    "the time budget was used up before the search started"
                },
            );
        }
        let reduction = match reduce(tables, &cube, self.config.threads, deadline) {
            Ok(moves) => moves,
            Err(Stuck::TimedOut) => {
                return fail(
                    Failure::BudgetExhausted,
                    "the reduction ran out of time before the 3x3x3 stage",
                )
            }
            Err(Stuck::NoWay) => return fail(Failure::Internal, "the reduction found no solution"),
        };
        let head: Vec<Move> = reduction.iter().map(|&code| move_of(code)).collect();
        let Ok(reduced) = facelets.apply(&MoveSequence::new(head.clone())) else {
            return fail(Failure::Internal, "the reduction moves do not fit the cube");
        };

        let limit = options.max_moves.map(usize::from);
        if limit.is_some_and(|max| head.len() > max) {
            return fail(
                Failure::Exhausted,
                "the reduction alone is longer than the move limit",
            );
        }
        let remaining = match deadline {
            Some(d) => {
                let now = Instant::now();
                if expired(now) {
                    return fail(
                        Failure::BudgetExhausted,
                        "the reduction used up the time budget before the 3x3x3 stage",
                    );
                }
                Some(d - now)
            }
            None => None,
        };
        let tail_limit = limit.map_or(THREE_MAX_MOVES, |max| {
            u8::try_from(max - head.len()).map_or(THREE_MAX_MOVES, |n| n.min(THREE_MAX_MOVES))
        });
        let hand_off = SolveOptions {
            time_budget: Some(
                remaining.map_or(HAND_OFF_BUDGET, |r| r.max(Duration::from_millis(1))),
            ),
            max_moves: Some(tail_limit),
        };
        let finish = self.three.solve(&three_by_three(&reduced), &hand_off);
        let tail = match finish.outcome {
            Ok(tail) => tail,
            Err(Failure::BudgetExhausted) if deadline.is_none() => {
                return fail(
                    Failure::Exhausted,
                    "no 3x3x3 solution within the move limit was found in 10 s (set a time budget to allow longer)",
                );
            }
            Err(failure) => {
                let message = finish.message.unwrap_or_else(|| "the 3x3x3 failed".into());
                return fail(failure, &format!("3x3x3 stage: {message}"));
            }
        };
        if tail.moves().iter().any(|m| m.depth != 1 || m.wide) {
            return fail(
                Failure::Internal,
                "the 3x3x3 stage returned a move this cube cannot make as a 3x3x3 turn",
            );
        }
        let solution = MoveSequence::new(join(head, tail.moves()));
        if !facelets.apply(&solution).is_ok_and(|end| end.is_solved()) {
            return fail(
                Failure::Internal,
                "the solution does not solve the cube (the 3x3x3 stage was given an unsolvable state?)",
            );
        }
        if limit.is_some_and(|max| solution.len() > max) {
            return fail(
                Failure::Exhausted,
                "the solution is longer than the move limit",
            );
        }
        done(Ok(solution), None)
    }
}

#[cfg(test)]
mod tests;
