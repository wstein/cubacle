mod consts;
mod coordinates;
mod cubie;
mod search;
mod solver;
mod stats;
mod tables;

use rand::RngExt;
use std::sync::atomic::AtomicI8;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use consts::*;
use cubie::*;
use search::*;
use tables::*;

pub use solver::Min2PhaseSolver;
pub use stats::{SolveOutcome, SolveStats};

lazy_static! {
    static ref global_sctx: Box<StaticContext> = StaticContext::box_new();
    static ref global_stbl: Box<StaticTables> = StaticTables::box_new(&global_sctx);
}

/// Solve a Rubik's cube represented in facelet
///
/// # Arguments
///
/// * `facelet` - the Rubik's cube to be solved, represented in facelet
/// * `maxl` - number of moves to solve the cube, included. 21 or 20 is recommended.
///
/// Facelet for the rubik's cube:
/// ```text
///          +--------+
///          |U1 U2 U3|
///          |U4 U5 U6|
///          |U7 U8 U9|
/// +--------+--------+--------+--------+
/// |L1 L2 L3|F1 F2 F3|R1 R2 R3|B1 B2 B3|
/// |L4 L5 L6|F4 F5 F6|R4 R5 R6|B4 B5 B6|
/// |L7 L8 L9|F7 F8 F9|R7 R8 R9|B7 B8 B9|
/// +--------+--------+--------+--------+
///          |D1 D2 D3|
///          |D4 D5 D6|
///          |D7 D8 D9|
///          +--------+
/// ```
/// should be: U1U2...U9R1R2...R9F1..F9D1..D9L1..L9B1..B9
/// Example, facelet of solved cube is UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
///
/// Return solution moves on success, return "Error " + error_code on failure
pub fn solve(facelet: &str, maxl: u8) -> String {
    solve_with_stats(facelet, maxl).solution
}

/// Solve a Rubik's cube and return its solution with phase-two probe statistics.
pub fn solve_with_stats(facelet: &str, maxl: u8) -> SolveStats {
    solve_with_time_limits(facelet, maxl, Duration::ZERO, None)
}

/// Solve a Rubik's cube with an optional wall-clock search budget.
///
/// A timed-out search returns `Error 8` with `SolveStats::timed_out` set to `true`.
pub fn solve_with_budget(facelet: &str, maxl: u8, budget: Duration) -> SolveStats {
    solve_with_time_limits(facelet, maxl, Duration::ZERO, Some(budget))
}

/// Solve a Rubik's cube while retaining and improving solutions for at least `minimum_search_time`.
///
/// `maximum_search_time` is a hard wall-clock limit. The solver returns the best solution found
/// after the minimum has elapsed, or the best available solution when the maximum expires. The
/// minimum is clamped to the maximum.
pub fn solve_with_minimum_time(
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Duration,
) -> SolveStats {
    solve_with_time_limits(
        facelet,
        maxl,
        minimum_search_time,
        Some(maximum_search_time),
    )
}

/// Find a move sequence that transforms `from_facelet` into `to_facelet`.
///
/// The returned sequence has at most `maxl` moves, or is an `Error <code>` result
/// when either facelet representation is invalid or no sequence is found within
/// the requested limit.
pub fn solve_pattern(from_facelet: &str, to_facelet: &str, maxl: u8) -> SolveStats {
    let from = match parse_facelet(from_facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    let to = match parse_facelet(to_facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    let mut to_inverse = Cubie::new();
    let mut relative = Cubie::new();
    Cubie::inv(&to, &mut to_inverse);
    Cubie::corn_mult(&to_inverse, &from, &mut relative);
    Cubie::edge_mult(&to_inverse, &from, &mut relative);
    solve_cubie_entry(relative, maxl, Duration::ZERO, None)
}

/// Solve independent cube states concurrently and return results in input order.
///
/// This API is intended for throughput-oriented bulk work. It uses a persistent
/// process-wide worker pool, whose size is read once from `CUBACLE_THREADS` on
/// first use; otherwise the available parallelism is used. Calls made from one
/// of those workers run serially to avoid nested-pool oversubscription.
pub fn solve_batch<T: AsRef<str> + Sync>(facelets: &[T], maxl: u8) -> Vec<SolveStats> {
    if facelets.len() < 2 || crate::pool::worker_count() <= 1 || crate::pool::in_pool_worker() {
        return facelets
            .iter()
            .map(|facelet| solve_with_stats(facelet.as_ref(), maxl))
            .collect();
    }

    let (sender, receiver) = mpsc::channel();
    for (index, facelet) in facelets.iter().enumerate() {
        let sender = sender.clone();
        let facelet = facelet.as_ref().to_owned();
        crate::pool::global().execute(move || {
            let stats = solve_with_stats(&facelet, maxl);
            let _ = sender.send((index, stats));
        });
    }
    drop(sender);

    let mut results = vec![None; facelets.len()];
    for (index, stats) in receiver {
        results[index] = Some(stats);
    }
    results
        .into_iter()
        .map(|result| {
            result.expect(
                "cubacle batch job produced no result (a worker panicked; see message above)",
            )
        })
        .collect()
}

/// Experimentally solve one cube by searching its six URF orientations concurrently.
///
/// The result is selected deterministically from all orientation searches. Use
/// [`solve_batch`] for throughput-oriented work on independent cubes.
pub fn solve_parallel(facelet: &str, maxl: u8) -> SolveStats {
    solve_parallel_with_time_limits(facelet, maxl, Duration::ZERO, None)
}

/// Experimentally solve one cube in parallel subject to a wall-clock search budget.
pub fn solve_parallel_with_budget(facelet: &str, maxl: u8, budget: Duration) -> SolveStats {
    solve_parallel_with_time_limits(facelet, maxl, Duration::ZERO, Some(budget))
}

/// Experimentally solve one cube in parallel while retaining solutions until `minimum_search_time`.
pub fn solve_parallel_with_minimum_time(
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Duration,
) -> SolveStats {
    solve_parallel_with_time_limits(
        facelet,
        maxl,
        minimum_search_time,
        Some(maximum_search_time),
    )
}

/// Experimentally find a parallel solution that transforms `from_facelet` into `to_facelet`.
pub fn solve_pattern_parallel(from_facelet: &str, to_facelet: &str, maxl: u8) -> SolveStats {
    let from = match parse_facelet(from_facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    let to = match parse_facelet(to_facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    let mut to_inverse = Cubie::new();
    let mut relative = Cubie::new();
    Cubie::inv(&to, &mut to_inverse);
    Cubie::corn_mult(&to_inverse, &from, &mut relative);
    Cubie::edge_mult(&to_inverse, &from, &mut relative);
    solve_parallel_cubie_entry(relative, maxl, Duration::ZERO, None)
}

fn solve_with_time_limits(
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let cc = match parse_facelet(facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    solve_cubie_entry(cc, maxl, minimum_search_time, maximum_search_time)
}

fn solve_parallel_with_time_limits(
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let cc = match parse_facelet(facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    solve_parallel_cubie_entry(cc, maxl, minimum_search_time, maximum_search_time)
}

fn solve_parallel_cubie_entry(
    cc: Cubie,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    if crate::pool::worker_count() <= 1 || crate::pool::in_pool_worker() {
        return solve_cubie_entry(cc, maxl, minimum_search_time, maximum_search_time);
    }

    let started = Instant::now();
    let deadline = maximum_search_time.and_then(|maximum| started.checked_add(maximum));
    let minimum_deadline = maximum_search_time
        .map(|maximum| minimum_search_time.min(maximum))
        .and_then(|minimum| started.checked_add(minimum));
    let urf_cubies = make_urf_cubies(&cc);
    let best_len = Arc::new(AtomicI8::new(std::cmp::min(25, maxl) as i8 + 1));
    let task_count = urf_cubies.len();
    let (sender, receiver) = mpsc::channel();

    for (urf_idx, urf_cubie) in urf_cubies.into_iter().enumerate() {
        let sender = sender.clone();
        let best_len = Arc::clone(&best_len);
        crate::pool::global().execute(move || {
            let mut ctx = IdaContext::new();
            ctx.improve = true;
            ctx.shared_best = Some(best_len);
            ctx.set_deadlines(deadline, minimum_deadline);
            let solution = ctx.solve_one_urf(
                &global_sctx,
                &global_stbl,
                &urf_cubie,
                urf_idx as u8,
                std::cmp::min(25, maxl) as i8,
            );
            let _ = sender.send((urf_idx, stats_from_context(&ctx, solution)));
        });
    }
    drop(sender);

    let mut results = vec![None; task_count];
    for (urf_idx, stats) in receiver {
        results[urf_idx] = Some(stats);
    }
    let mut first_error = None;
    for stats in results.into_iter().map(|result| {
        result.expect(
            "parallel solver worker produced no result (a worker panicked; see message above)",
        )
    }) {
        if !stats.solution.starts_with("Error ") {
            return stats;
        }
        if first_error.is_none() {
            first_error = Some(stats);
        }
    }
    first_error.expect("parallel solver did not return a result")
}

fn make_urf_cubies(cc: &Cubie) -> [Cubie; 6] {
    let mut cubies = [Cubie::new(); 6];
    let mut current = *cc;
    let mut next = Cubie::new();
    for (index, cubie) in cubies.iter_mut().enumerate() {
        *cubie = current;
        Cubie::corn_mult(&global_sctx.symurfi, &current, &mut next);
        Cubie::edge_mult(&global_sctx.symurfi, &current, &mut next);
        Cubie::corn_mult(&next, &global_sctx.symurf, &mut current);
        Cubie::edge_mult(&next, &global_sctx.symurf, &mut current);
        if index == 2 {
            Cubie::inv(&current, &mut next);
            current = next;
        }
    }
    cubies
}

fn parse_facelet(facelet: &str) -> Result<Cubie, SolveStats> {
    let mut cc = Cubie::new();
    if cc.set_from_facelet(facelet) < 0 {
        return Err(unsolvable_stats(String::from("Error 1")));
    }
    let verify = cc.verify();
    if verify < 0 {
        return Err(unsolvable_stats(
            String::from("Error ") + &(-verify).to_string(),
        ));
    }
    Ok(cc)
}

fn unsolvable_stats(solution: String) -> SolveStats {
    SolveStats {
        solution,
        probes: 0,
        phase1_nodes: 0,
        phase1_rejected_slice_twist: 0,
        phase1_rejected_slice_flip: 0,
        phase1_slice_twist_breaks: 0,
        phase1_slice_twist_equals: 0,
        phase1_slice_flip_breaks: 0,
        phase1_slice_flip_equals: 0,
        phase1_children_recurred: 0,
        timed_out: false,
        outcome: SolveOutcome::Unsolvable,
    }
}

fn solve_cubie_entry(
    cc: Cubie,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let mut ctx = IdaContext::new();
    ctx.set_time_limits(minimum_search_time, maximum_search_time);
    let solution = ctx.solve_cubie(
        &global_sctx,
        &global_stbl,
        &cc,
        std::cmp::min(25, maxl) as i8,
    );
    stats_from_context(&ctx, solution)
}

fn stats_from_context(ctx: &IdaContext, solution: String) -> SolveStats {
    let outcome = if ctx.deadline_with_solution {
        SolveOutcome::DeadlineWithSolution
    } else if ctx.timed_out {
        SolveOutcome::DeadlineNoSolution
    } else if solution.starts_with("Error ") {
        SolveOutcome::Unsolvable
    } else {
        SolveOutcome::Solved
    };
    SolveStats {
        solution,
        probes: ctx.probes,
        phase1_nodes: ctx.phase1_nodes,
        phase1_rejected_slice_twist: ctx.phase1_rejected_slice_twist,
        phase1_rejected_slice_flip: ctx.phase1_rejected_slice_flip,
        phase1_slice_twist_breaks: ctx.phase1_slice_twist_breaks,
        phase1_slice_twist_equals: ctx.phase1_slice_twist_equals,
        phase1_slice_flip_breaks: ctx.phase1_slice_flip_breaks,
        phase1_slice_flip_equals: ctx.phase1_slice_flip_equals,
        phase1_children_recurred: ctx.phase1_children_recurred,
        timed_out: ctx.timed_out && !ctx.has_solution,
        outcome,
    }
}

/// Generate a random cube represented in facelet
pub fn random_cube() -> String {
    let mut cc = Cubie::new();
    cc.random_reset();
    cc.as_facelet()
}

/// Apply moves to a solved Rubik's cube
///
/// # Arguments
///
/// * `cube_moves` - should match ```([URFDLB][123'] ?)*```
///
/// Return ```facelet``` on success
pub fn from_moves(cube_moves: &str) -> Option<String> {
    apply_moves(&Cubie::new().as_facelet(), cube_moves)
}

/// Apply moves to a Rubik's cube represented by facelet
///
/// # Arguments
///
/// * `facelet` - the Rubik's cube to be moved, must be a solvable Rubik's cube
/// * `cube_moves` - should match ```([URFDLB][123'] ?)*```
///
/// Return ```facelet``` of the moved cube on success
pub fn apply_moves(facelet: &str, cube_moves: &str) -> Option<String> {
    let mut cc = Cubie::new();
    if cc.set_from_facelet(facelet) < 0 {
        return None;
    }
    let verify = cc.verify();
    if verify < 0 {
        return None;
    }
    let s = cube_moves.trim().chars();
    let mut axis = 0;
    let mut pow = 0;
    let mut cd = Cubie::new();
    for c in s {
        match c {
            'U' | 'R' | 'F' | 'D' | 'L' | 'B' => {
                if pow != 0 {
                    Cubie::corn_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
                    Cubie::edge_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
                    cc = cd;
                }
                pow = 1;
                match c {
                    'U' => {
                        axis = 0;
                    }
                    'R' => {
                        axis = 1;
                    }
                    'F' => {
                        axis = 2;
                    }
                    'D' => {
                        axis = 3;
                    }
                    'L' => {
                        axis = 4;
                    }
                    'B' => {
                        axis = 5;
                    }
                    _ => (),
                }
            }
            '\'' | '-' => pow = (4 - pow) % 4,
            '3' => pow = pow * 3 % 4,
            '2' => pow = pow * 2 % 4,
            '+' | '1' | ' ' | '\t' => (),
            _ => {
                return None;
            }
        }
    }

    if pow != 0 {
        Cubie::corn_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
        Cubie::edge_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
        cc = cd;
    }
    Some(cc.as_facelet())
}

/// Generate a random move sequence in specific number of moves
///
/// # Arguments
///
/// * `n_moves` - number of moves
///
/// Return moves, ensure no redaudant moves exists, e.g. "R R", "R L R", etc.
///
/// Call ```from_moves(cube_moves)``` to obtain the scrambled cube
pub fn random_moves(n_moves: u16) -> String {
    let mut rng = rand::rng();
    let mut last_axis = 18;
    let mut scramble = String::new();
    let mut i = 0;
    while i < n_moves {
        let mv = rng.random_range(0..18);
        let axis = mv / 3;
        if axis == last_axis || (axis % 3 == last_axis % 3 && axis > last_axis) {
            continue;
        }
        last_axis = axis;
        scramble.push_str(MOVE2STR[mv]);
        scramble.push(' ');
        i += 1
    }
    scramble
}
