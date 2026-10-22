//! Measuring tool for the 5x5x5: our solver on a list of cubes, and the check
//! of an external reduction solver's answers against the same cubes.
//!
//! `five_compare ours CUBES` solves every cube (one facelet string per line)
//! with `FiveSolver` and replays each answer.
//! `five_compare external CUBES ANSWERS` takes lines `millis<TAB>moves` (the
//! external solver's reduction moves, lowercase letters are two-layer turns),
//! replays them, finishes with our 3x3x3 hand-off and replays the whole.
//! It needs no code of the external solver; see `tools/bench/README.md`.

use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Instant;

use cubacle::engines::five::FiveSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, MoveSequence, SolveOptions, Solver};

fn stats(name: &str, mut values: Vec<f64>) {
    values.sort_by(f64::total_cmp);
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    println!(
        "{name}: n={n} mean={mean:.1} median={:.1} min={:.1} max={:.1} p90={:.1}",
        values[n / 2],
        values[0],
        values[n - 1],
        values[(n * 9 / 10).min(n - 1)]
    );
}

/// `u` becomes `Uw`: the external notation writes two-layer turns in lowercase.
fn to_notation(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            let mut chars = token.chars();
            match chars.next() {
                Some(c) if c.is_ascii_lowercase() => {
                    format!("{}w{}", c.to_ascii_uppercase(), chars.as_str())
                }
                _ => token.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_inner(sequence: &MoveSequence) -> bool {
    sequence.moves().iter().any(|m| m.depth > 1 || m.wide)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cubes: Vec<Facelets> = fs::read_to_string(&args[2])
        .expect("cubes")
        .lines()
        .map(|l| Facelets::parse(l.trim()).expect("facelets"))
        .collect();
    let solver = FiveSolver::new(Arc::new(Min2PhaseSolver::shared()));
    let started = Instant::now();
    solver.prepare();
    println!("prepare_ms={}", started.elapsed().as_millis());
    let (mut total, mut reduction, mut tail, mut millis) = (vec![], vec![], vec![], vec![]);
    let mut bad = 0;
    match args[1].as_str() {
        "ours" => {
            for cube in &cubes {
                let t = Instant::now();
                let result = solver.solve(cube, &SolveOptions::default());
                millis.push(t.elapsed().as_secs_f64() * 1e3);
                match result.solution() {
                    Some(s) if cube.apply(s).is_ok_and(|c| c.is_solved()) => {
                        total.push(s.len() as f64)
                    }
                    _ => bad += 1,
                }
            }
        }
        "external" => {
            let answers = fs::read_to_string(&args[3]).expect("answers");
            for (cube, line) in cubes.iter().zip(answers.lines()) {
                let (ms, moves) = line.split_once('\t').expect("millis<TAB>moves");
                millis.push(ms.parse().expect("millis"));
                let Ok(head) = to_notation(moves).parse::<MoveSequence>() else {
                    bad += 1;
                    continue;
                };
                let Ok(mid) = cube.apply(&head) else {
                    bad += 1;
                    continue;
                };
                let result = solver.solve(&mid, &SolveOptions::default());
                let ok = result.solution().is_some_and(|t| {
                    !is_inner(t)
                        && mid.apply(t).is_ok_and(|c| c.is_solved())
                        && cube
                            .apply(&head)
                            .and_then(|c| c.apply(t))
                            .is_ok_and(|c| c.is_solved())
                });
                if !ok {
                    bad += 1;
                    continue;
                }
                let t = result.solution().map_or(0, MoveSequence::len);
                reduction.push(head.len() as f64);
                tail.push(t as f64);
                total.push((head.len() + t) as f64);
            }
        }
        other => panic!("unknown mode {other}"),
    }
    println!("failed_or_unverified={bad}");
    stats("solve_ms", millis);
    if !reduction.is_empty() {
        stats("reduction_moves", reduction);
        stats("three_moves", tail);
    }
    stats("total_moves", total);
}
