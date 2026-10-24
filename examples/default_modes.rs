//! Moves and time of the 3x3x3 solve modes on a fixed, seeded set of cubes.
//!
//! `cargo run --release --example default_modes -- <set> <mode> [value]`
//! with set `random` (100 cubes), `short` (30 scrambles of 6 to 12 moves) or
//! `targets` (superflip, checkerboard), and mode `default`, `probes N` or
//! `improve MS`. The cubes come from a fixed generator, so every run sees
//! the same ones. Prints mean, median and maximum moves, mean, p95 and
//! maximum wall time, and the share of cubes shorter than the first answer.
use std::env;
use std::time::{Duration, Instant};

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind, SolveOptions, Solver};

const FACES: [&str; 6] = ["U", "D", "R", "L", "F", "B"];
const TURNS: [&str; 3] = ["", "2", "'"];

fn scramble(seed: &mut u64, length: usize) -> String {
    let mut next = |bound: usize| {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*seed >> 33) as usize) % bound
    };
    let mut moves: Vec<String> = Vec::new();
    let mut last = usize::MAX;
    while moves.len() < length {
        let face = next(6);
        if face / 2 == last / 2 {
            continue;
        }
        last = face;
        moves.push(format!("{}{}", FACES[face], TURNS[next(3)]));
    }
    moves.join(" ")
}

fn cubes(set: &str) -> Vec<String> {
    let mut seed = 20261024;
    match set {
        "random" => (0..100).map(|_| scramble(&mut seed, 40)).collect(),
        "short" => (0..30).map(|i| scramble(&mut seed, 6 + i % 7)).collect(),
        "targets" => vec![
            "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2".into(),
            "U2 D2 R2 L2 F2 B2".into(),
        ],
        other => panic!("unknown set {other}"),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (set, mode) = (args[0].as_str(), args[1].as_str());
    let value: u64 = args.get(2).map_or(0, |v| v.parse().expect("number"));
    let solver = Min2PhaseSolver::new();
    solver.prepare();
    let options = SolveOptions::default().with_max_moves(21);
    let (mut lengths, mut times, mut shorter, mut failed) = (Vec::new(), Vec::new(), 0, 0);
    let list = cubes(set);
    for text in &list {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&text.parse::<MoveSequence>().expect("moves"))
            .expect("apply");
        let first = solver.solve(&cube, &options).solution().map(|s| s.len());
        let budget = Duration::from_millis(value);
        let started = Instant::now();
        let result = match mode {
            "default" => solver.solve(&cube, &options),
            "probes" => solver.solve(&cube, &options.with_min_probes(value)),
            "improve" => {
                solver.solve_with_minimum_time(&cube, &options.with_time_budget(budget), budget)
            }
            other => panic!("unknown mode {other}"),
        };
        times.push(started.elapsed().as_secs_f64() * 1000.0);
        let Some(solution) = result.solution() else {
            failed += 1;
            continue;
        };
        assert!(cube.apply(solution).unwrap().is_solved());
        lengths.push(solution.len());
        if Some(solution.len()) < first {
            shorter += 1;
        }
    }
    let n = lengths.len();
    if n == 0 {
        println!("{set} {mode} {value}: no solution for any of {failed}");
        return;
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / n as f64;
    lengths.sort_unstable();
    times.sort_by(|a, b| a.total_cmp(b));
    let lens: Vec<f64> = lengths.iter().map(|&l| l as f64).collect();
    println!(
        "{set} {mode} {value}: moves mean {:.2} median {} max {} | ms mean {:.1} p95 {:.1} max {:.1} | improved {:.0}% | failed {}",
        mean(&lens),
        lengths[n / 2],
        lengths[n - 1],
        mean(&times),
        times[(n * 95).div_ceil(100) - 1],
        times[n - 1],
        100.0 * shorter as f64 / n as f64,
        failed
    );
}
