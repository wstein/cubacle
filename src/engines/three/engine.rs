use std::sync::atomic::AtomicI8;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::cubie::*;
use super::search::*;
use super::stats::{SolveOutcome, SolveStats};
use super::tables::{self, SharedTables, Tables};

/// Solve a Rubik's cube and return its solution with phase-two probe statistics.
pub(super) fn solve_with_stats(tables: &SharedTables, facelet: &str, maxl: u8) -> SolveStats {
    solve_with_time_limits(tables, facelet, maxl, Duration::ZERO, None)
}

/// Find a move sequence that transforms `from_facelet` into `to_facelet`.
///
/// The returned sequence has at most `maxl` moves, or is an `Error <code>` result
/// when either facelet representation is invalid or no sequence is found within
/// the requested limit.
pub(super) fn solve_pattern(
    tables: &SharedTables,
    from_facelet: &str,
    to_facelet: &str,
    maxl: u8,
) -> SolveStats {
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
    solve_cubie_entry(tables, relative, maxl, Duration::ZERO, None)
}

/// Solve independent cube states concurrently and return results in input order.
///
/// This API is intended for throughput-oriented bulk work. It uses a persistent
/// process-wide worker pool, whose size is read once from `CUBACLE_THREADS` on
/// first use; otherwise the available parallelism is used. Calls made from one
/// of those workers run serially to avoid nested-pool oversubscription.
pub(super) fn solve_batch<T: AsRef<str> + Sync>(
    tables: &SharedTables,
    facelets: &[T],
    maxl: u8,
) -> Vec<SolveStats> {
    if facelets.len() < 2 || super::pool::worker_count() <= 1 || super::pool::in_pool_worker() {
        return facelets
            .iter()
            .map(|facelet| solve_with_stats(tables, facelet.as_ref(), maxl))
            .collect();
    }

    let (sender, receiver) = mpsc::channel();
    for (index, facelet) in facelets.iter().enumerate() {
        let sender = sender.clone();
        let facelet = facelet.as_ref().to_owned();
        let tables = Arc::clone(tables);
        super::pool::global().execute(move || {
            let stats = solve_with_stats(&tables, &facelet, maxl);
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

/// Experimentally find a parallel solution that transforms `from_facelet` into `to_facelet`.
pub(super) fn solve_pattern_parallel(
    tables: &SharedTables,
    from_facelet: &str,
    to_facelet: &str,
    maxl: u8,
) -> SolveStats {
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
    solve_parallel_cubie_entry(tables, relative, maxl, Duration::ZERO, None)
}

pub(super) fn solve_with_time_limits(
    tables: &SharedTables,
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let cc = match parse_facelet(facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    solve_cubie_entry(tables, cc, maxl, minimum_search_time, maximum_search_time)
}

pub(super) fn solve_parallel_with_time_limits(
    tables: &SharedTables,
    facelet: &str,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let cc = match parse_facelet(facelet) {
        Ok(cubie) => cubie,
        Err(stats) => return stats,
    };
    solve_parallel_cubie_entry(tables, cc, maxl, minimum_search_time, maximum_search_time)
}

fn solve_parallel_cubie_entry(
    tables: &SharedTables,
    cc: Cubie,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    if super::pool::worker_count() <= 1 || super::pool::in_pool_worker() {
        return solve_cubie_entry(tables, cc, maxl, minimum_search_time, maximum_search_time);
    }

    let started = Instant::now();
    let deadline = maximum_search_time.and_then(|maximum| started.checked_add(maximum));
    let minimum_deadline = maximum_search_time
        .map(|maximum| minimum_search_time.min(maximum))
        .and_then(|minimum| started.checked_add(minimum));
    let urf_cubies = make_urf_cubies(tables::get(tables), &cc);
    let best_len = Arc::new(AtomicI8::new(std::cmp::min(25, maxl) as i8 + 1));
    let task_count = urf_cubies.len();
    let (sender, receiver) = mpsc::channel();

    for (urf_idx, urf_cubie) in urf_cubies.into_iter().enumerate() {
        let sender = sender.clone();
        let best_len = Arc::clone(&best_len);
        let tables = Arc::clone(tables);
        super::pool::global().execute(move || {
            let mut ctx = IdaContext::new();
            ctx.improve = true;
            ctx.shared_best = Some(best_len);
            ctx.set_deadlines(deadline, minimum_deadline);
            let built = tables::get(&tables);
            let solution = ctx.solve_one_urf(
                &built.sctx,
                &built.stbl,
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

fn make_urf_cubies(tables: &Tables, cc: &Cubie) -> [Cubie; 6] {
    let mut cubies = [Cubie::new(); 6];
    let mut current = *cc;
    let mut next = Cubie::new();
    for (index, cubie) in cubies.iter_mut().enumerate() {
        *cubie = current;
        Cubie::corn_mult(&tables.sctx.symurfi, &current, &mut next);
        Cubie::edge_mult(&tables.sctx.symurfi, &current, &mut next);
        Cubie::corn_mult(&next, &tables.sctx.symurf, &mut current);
        Cubie::edge_mult(&next, &tables.sctx.symurf, &mut current);
        if index == 2 {
            Cubie::inv(&current, &mut next);
            current = next;
        }
    }
    cubies
}

fn parse_facelet(facelet: &str) -> Result<Cubie, SolveStats> {
    let mut cc = Cubie::new();
    let parsed = cc.set_from_facelet(facelet);
    if parsed < 0 {
        return Err(unsolvable_stats(
            String::from("Error ") + &(-parsed).to_string(),
        ));
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
    tables: &SharedTables,
    cc: Cubie,
    maxl: u8,
    minimum_search_time: Duration,
    maximum_search_time: Option<Duration>,
) -> SolveStats {
    let mut ctx = IdaContext::new();
    ctx.set_time_limits(minimum_search_time, maximum_search_time);
    let built = tables::get(tables);
    let solution = ctx.solve_cubie(&built.sctx, &built.stbl, &cc, std::cmp::min(25, maxl) as i8);
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
