//! A fixed pilot series for `recover`.
//!
//! `cargo run --release --example recover_pilot` runs the cases of
//! tests/fixtures/recover_pilot.txt with fixed mistakes and prints the tables;
//! there are no thresholds. `cargo run --release --example recover_pilot --
//! generate` writes the data file (done once, the file is committed).
//! `PILOT_CASES=a,b,c` sets the cases for the 7x7x7, 5x5x5 and 4x4x4 cubes.

mod cases;
mod data;
mod gen;
mod report;
mod rng;
mod run;
mod solvers;

use std::time::{Duration, Instant};

use solvers::{kind_of, solver_for};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("generate") {
        gen::generate();
        return;
    }
    let started = Instant::now();
    let cubes = data::load();
    let counts: Vec<usize> = std::env::var("PILOT_CASES")
        .ok()
        .map(|v| v.split(',').map(|n| n.parse().expect("number")).collect())
        .unwrap_or_else(|| vec![6, 6, 6]);
    let per_cube: Vec<usize> = cubes
        .iter()
        .map(|c| match c.size {
            7 => counts[0],
            5 => counts[1],
            _ => counts[2],
        })
        .collect();
    let cases = cases::build(&cubes, &per_cube);
    let mut results = Vec::new();
    let mut current = (0, solver_for(kind_of(cubes[0].size)));
    for case in &cases {
        if case.cube != current.0 {
            current = (case.cube, solver_for(kind_of(cubes[case.cube].size)));
        }
        let pilot = &cubes[case.cube];
        let outcome = run::run(case, pilot, current.1.as_ref(), Duration::from_secs(5));
        let result = report::Result {
            case,
            size: pilot.size,
            outcome,
        };
        report::case_line(&result, pilot);
        results.push(result);
    }
    report::tables(&results);
    println!("\n{} cases in {:.0?}", results.len(), started.elapsed());
}
