mod consts;
mod coordinates;
mod cubie;
mod search;
mod solver;
mod stats;
mod tables;

use std::sync::atomic::AtomicI8;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cubie::*;
use search::*;
use tables::*;

pub use solver::Min2PhaseSolver;
use stats::{SolveOutcome, SolveStats};

lazy_static! {
    static ref global_sctx: Box<StaticContext> = StaticContext::box_new();
    static ref global_stbl: Box<StaticTables> = StaticTables::box_new(&global_sctx);
}

/// Solve a Rubik's cube and return its solution with phase-two probe statistics.
fn solve_with_stats(facelet: &str, maxl: u8) -> SolveStats {
    solve_with_time_limits(facelet, maxl, Duration::ZERO, None)
}

/// Find a move sequence that transforms `from_facelet` into `to_facelet`.
///
/// The returned sequence has at most `maxl` moves, or is an `Error <code>` result
/// when either facelet representation is invalid or no sequence is found within
/// the requested limit.
fn solve_pattern(from_facelet: &str, to_facelet: &str, maxl: u8) -> SolveStats {
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
fn solve_batch<T: AsRef<str> + Sync>(facelets: &[T], maxl: u8) -> Vec<SolveStats> {
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

/// Experimentally find a parallel solution that transforms `from_facelet` into `to_facelet`.
fn solve_pattern_parallel(from_facelet: &str, to_facelet: &str, maxl: u8) -> SolveStats {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

    /// Applies moves with the engine's own cubie model.
    fn engine_apply(moves: &MoveSequence) -> String {
        let mut cube = Cubie::new();
        let mut next = Cubie::new();
        for mv in moves.moves() {
            let axis = mv.face.index();
            let power = match mv.turn {
                crate::puzzle::Turn::Clockwise => 0,
                crate::puzzle::Turn::Half => 1,
                crate::puzzle::Turn::CounterClockwise => 2,
            };
            let turn = &global_sctx.movecube[axis * 3 + power];
            Cubie::corn_mult(&cube, turn, &mut next);
            Cubie::edge_mult(&cube, turn, &mut next);
            cube = next;
        }
        cube.as_facelet()
    }

    #[test]
    fn facelets_apply_agrees_with_the_engine_cubie_model() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = |bound: u64| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) % bound
        };
        let letters = ["U", "R", "F", "D", "L", "B"];
        let suffixes = ["", "2", "'"];
        for _ in 0..200 {
            let text: Vec<String> = (0..1 + next(30))
                .map(|_| {
                    let face = letters[next(6) as usize];
                    format!("{face}{}", suffixes[next(3) as usize])
                })
                .collect();
            let moves: MoveSequence = text.join(" ").parse().unwrap();
            let typed = Facelets::solved(PuzzleKind::Cube3).apply(&moves).unwrap();
            assert_eq!(typed.to_string(), engine_apply(&moves), "{moves}");
        }
    }

    #[test]
    fn generated_states_are_reachable_and_solve_back() {
        use crate::engines::three::Min2PhaseSolver;
        use crate::puzzle::{SolveOptions, Solver};
        let solver = Min2PhaseSolver::new();
        for _ in 0..100 {
            let cube = crate::generate::random_state(PuzzleKind::Cube3).unwrap();
            let result = solver.solve(&cube, &SolveOptions::default());
            let solution = result.solution().expect("a generated state is reachable");
            assert!(cube.apply(solution).unwrap().is_solved());
        }
    }

    #[test]
    fn stats_count_probes_and_balance_the_pruning_tallies() {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R U R' U'".parse().unwrap())
            .unwrap();
        let result = solve_with_stats(&cube.to_string(), 20);
        assert_eq!(result.outcome, SolveOutcome::Solved);
        assert!(result.probes > 0, "expected phase-two search probes");
        assert!(result.phase1_nodes > 0, "expected phase-one search nodes");
        assert!(result.phase1_children_recurred > 0);
        assert_eq!(
            result.phase1_rejected_slice_twist,
            result.phase1_slice_twist_breaks + result.phase1_slice_twist_equals
        );
        assert_eq!(
            result.phase1_rejected_slice_flip,
            result.phase1_slice_flip_breaks + result.phase1_slice_flip_equals
        );
    }
}
