//! Moves against probes or time for the 3x3x3 improve modes.
//!
//! `cargo run --release --example improve_curve -- <cubes> <mode> [value]`
//! with mode `default`, `probes N`, `improve MS` or `parallel MS`.
use std::env;
use std::time::{Duration, Instant};

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::puzzle::{PuzzleKind, SolveOptions, Solver};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cubes: usize = args[0].parse().expect("number of cubes");
    let mode = args[1].as_str();
    let value: u64 = args.get(2).map_or(0, |v| v.parse().expect("number"));
    let solver = Min2PhaseSolver::new();
    solver.prepare();
    let (mut lengths, mut total, mut failed) = (Vec::new(), Duration::ZERO, 0);
    for _ in 0..cubes {
        let cube = generate::random_state(PuzzleKind::Cube3).expect("generator");
        let budget = Duration::from_millis(value);
        let options = SolveOptions::default().with_max_moves(21);
        let started = Instant::now();
        let result = match mode {
            "default" => solver.solve(&cube, &options),
            "probes" => solver.solve(&cube, &options.with_min_probes(value)),
            "improve" => {
                solver.solve_with_minimum_time(&cube, &options.with_time_budget(budget), budget)
            }
            "parallel" => solver.solve_parallel_with_minimum_time(
                &cube,
                &options.with_time_budget(budget),
                budget,
            ),
            other => panic!("unknown mode {other}"),
        };
        total += started.elapsed();
        let Some(solution) = result.solution() else {
            failed += 1;
            continue;
        };
        assert!(cube.apply(solution).unwrap().is_solved());
        lengths.push(solution.len());
    }
    lengths.sort_unstable();
    let solved = lengths.len();
    let mean = lengths.iter().sum::<usize>() as f64 / solved as f64;
    let p90 = lengths[(solved * 9).div_ceil(10) - 1];
    println!(
        "{mode} {value}: mean {mean:.2} moves, p90 {p90}, mean time {:.1} ms, {failed} without a solution",
        total.as_secs_f64() * 1000.0 / cubes as f64
    );
}
