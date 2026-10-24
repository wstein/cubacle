//! Measures the optimal 3x3x3 solver on fixed-seed cubes.
//!
//! ```sh
//! cargo run --release --example optimal_measure -- --tables large --cubes 100 --budget 60
//! cargo run --release --example optimal_measure -- --tables small --kind short --cubes 30
//! ```
//!
//! Options:
//!
//! * `--tables small|large`: the table set (default small).
//! * `--kind random|short|N`: random is an 80-move random walk standing in for
//!   a random state, short is a walk of 8 to 14 moves, N is a walk of N moves.
//! * `--cubes N`, `--seed S`: how many cubes, from which seed (default 100, 1).
//! * `--from N`: skip the first N cubes of the sequence, to repeat one.
//! * `--budget SECONDS`: CPU time per cube (default 60).
//! * `--nodes N`: stop each cube after N nodes, so that every run does the same
//!   work (for comparing speed).
//! * `--threads N`: threads that build the tables (default 2).
//!
//! One line per cube, then a summary. The time is the CPU time of the process
//! (user plus system), not wall time, so a loaded machine does not distort it
//! much.

use std::sync::Arc;

use cubacle::engines::optimal::{Limits, OptimalSolver, Tables};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

#[repr(C)]
struct Timespec {
    seconds: i64,
    nanoseconds: i64,
}

extern "C" {
    fn clock_gettime(clock: i32, out: *mut Timespec) -> i32;
}

#[cfg(target_os = "macos")]
const PROCESS_CPU_CLOCK: i32 = 12;
#[cfg(not(target_os = "macos"))]
const PROCESS_CPU_CLOCK: i32 = 2;

/// CPU seconds used by the process so far.
fn cpu_seconds() -> f64 {
    let mut t = Timespec {
        seconds: 0,
        nanoseconds: 0,
    };
    // SAFETY: `t` is a valid, writable `timespec`.
    unsafe { clock_gettime(PROCESS_CPU_CLOCK, &mut t) };
    t.seconds as f64 + t.nanoseconds as f64 * 1e-9
}

fn argument<T: std::str::FromStr>(name: &str, default: T) -> T {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// A walk of `length` moves with no move on the face before it.
fn walk(rng: &mut StdRng, length: usize) -> String {
    let faces = ["U", "R", "F", "D", "L", "B"];
    let turns = ["", "2", "'"];
    let mut last = 9;
    let mut out = Vec::new();
    while out.len() < length {
        let face = rng.random_range(0..6);
        if face == last {
            continue;
        }
        last = face;
        out.push(format!("{}{}", faces[face], turns[rng.random_range(0..3)]));
    }
    out.join(" ")
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let tables = match argument("--tables", "small".to_owned()).as_str() {
        "large" => Tables::Large,
        _ => Tables::Small,
    };
    let kind: String = argument("--kind", "random".to_owned());
    let cubes: usize = argument("--cubes", 100);
    let seed: u64 = argument("--seed", 1);
    let budget: f64 = argument("--budget", 60.0);
    let threads: usize = argument("--threads", 2);
    let nodes: u64 = argument("--nodes", u64::MAX);
    let first: usize = argument("--from", 0);

    let solver = OptimalSolver::new(Arc::new(Min2PhaseSolver::new()), tables)
        .with_threads(threads)
        .with_notes(|line| eprintln!("{line}"));
    let before = cpu_seconds();
    let wall = std::time::Instant::now();
    let built = solver.prepare_timed();
    println!(
        "tables {:?}: {:.0} MiB, build {:.1} s wall (0 = loaded from the cache), load+build {:.1} s CPU, {:.1} s wall",
        tables,
        solver.table_mib(),
        built,
        cpu_seconds() - before,
        wall.elapsed().as_secs_f64()
    );

    let mut rng = StdRng::seed_from_u64(seed);
    let mut times = Vec::new();
    let (mut proven, mut within10, mut nodes_proven) = (0usize, 0usize, 0u64);
    let (mut sum_h0, mut sum_upper, mut sum_final) = (0usize, 0usize, 0usize);
    let mut sum_proven_len = 0usize;
    for n in 0..cubes {
        let length = match kind.as_str() {
            "short" => rng.random_range(8..=14),
            "random" => 80,
            fixed => fixed
                .parse()
                .expect("--kind random, short or a walk length"),
        };
        let text = walk(&mut rng, length);
        if n < first {
            continue;
        }
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&text.parse().expect("moves"))
            .expect("a turn");
        let h0 = solver.lower_bound(&cube).expect("a cube");
        let start = cpu_seconds();
        let stop = || cpu_seconds() - start >= budget;
        let limits = Limits {
            nodes: (nodes != u64::MAX).then_some(nodes),
            cancel: Some(&stop),
            ..Limits::default()
        };
        let report = solver
            .solve_report(&cube, &SolveOptions::default().with_max_moves(21), &limits)
            .expect("solvable");
        let seconds = cpu_seconds() - start;
        let length = report.solution.len();
        println!(
            "{n:3} h0 {h0:2} upper {:2} found {length:2} {} nodes {:>12} cpu {seconds:8.2} s",
            report.upper_bound,
            if report.proven {
                "optimal  "
            } else {
                "not proven"
            },
            report.nodes
        );
        sum_h0 += usize::from(h0);
        sum_upper += usize::from(report.upper_bound);
        sum_final += length;
        if report.proven {
            proven += 1;
            sum_proven_len += length;
            nodes_proven += report.nodes;
            times.push(seconds);
            if seconds <= 10.0 {
                within10 += 1;
            }
        }
    }
    times.sort_by(f64::total_cmp);
    let mean = |sum: usize, count: usize| sum as f64 / count.max(1) as f64;
    println!("--- {cubes} cubes ({kind}, seed {seed}), budget {budget} s CPU each");
    println!(
        "mean bound h0 {:.2}; mean length min2phase {:.2}; mean length returned {:.2}",
        mean(sum_h0, cubes),
        mean(sum_upper, cubes),
        mean(sum_final, cubes)
    );
    println!(
        "proven optimal: {proven}/{cubes} within {budget} s, {within10}/{cubes} within 10 s; mean optimal length {:.2}",
        mean(sum_proven_len, proven)
    );
    println!(
        "CPU s of the proven: mean {:.2}, p50 {:.2}, p95 {:.2}, max {:.2}; mean nodes {:.0}",
        times.iter().sum::<f64>() / times.len().max(1) as f64,
        percentile(&times, 0.5),
        percentile(&times, 0.95),
        times.last().copied().unwrap_or(f64::NAN),
        nodes_proven as f64 / proven.max(1) as f64
    );
}
