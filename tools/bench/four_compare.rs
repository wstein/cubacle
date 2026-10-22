//! Measuring tool for the 4x4x4: our solver on a list of cubes, and the check
//! of an external solver's answers against the same cubes.
//!
//! `four_compare ours CUBES [OUT]` solves every cube (one facelet string per
//! line) with `FourSolver` and replays each answer; with OUT it writes
//! `millis<TAB>moves<TAB>moves without the rotation<TAB>solution` per cube.
//! `four_compare external CUBES ANSWERS` reads the lines of the Java driver
//! (`millis l1 l2 l3 l333 solution`, the solution ending with the rotation
//! as `x`, `y`, `z` letters), turns the rotation into wide turns, replays
//! the whole and reports the same statistics.
//! `four_compare phases CUBES ANSWERS OURS` compares the first `l1+l2+l3` moves
//! of each Java answer with ours (OURS is the file `ours` wrote), and the tails.
//! It needs no code of the external solver; see `tools/bench/README.md`.

use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Instant;

use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, MoveSequence, SolveOptions, Solver};

fn stats(name: &str, mut values: Vec<f64>) {
    if values.is_empty() {
        println!("{name}: no values");
        return;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    println!(
        "{name}: n={n} mean={mean:.2} median={:.2} p90={:.2} min={:.2} max={:.2}",
        values[n / 2],
        values[(n * 9 / 10).min(n - 1)],
        values[0],
        values[n - 1],
    );
}

fn histogram(name: &str, values: &[f64]) {
    let mut counts = std::collections::BTreeMap::new();
    for &v in values {
        *counts.entry(v as u32).or_insert(0usize) += 1;
    }
    let text: Vec<String> = counts.iter().map(|(k, c)| format!("{k}:{c}")).collect();
    println!("{name} histogram: {}", text.join(" "));
}

/// The Java rotation letters as wide turns: `x` is `Rw Lw'`, `y` is `Uw Dw'`,
/// `z` is `Fw Bw'`.
fn java_to_notation(solution: &str) -> String {
    solution
        .split_whitespace()
        .map(|token| {
            let (axis, rest) = token.split_at(1);
            let (a, b) = match axis {
                "x" => ("Rw", "Lw"),
                "y" => ("Uw", "Dw"),
                "z" => ("Fw", "Bw"),
                _ => return token.to_owned(),
            };
            let back = match rest {
                "'" => "",
                "2" => "2",
                _ => "'",
            };
            format!("{a}{rest} {b}{back}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn uniform(cube: &Facelets) -> bool {
    let n = cube.kind().face_len();
    cube.stickers()
        .chunks(n)
        .all(|face| face.iter().all(|&s| s == face[0]))
}

/// Moves of the solution without its closing rotation (trailing wide turns
/// after which the faces are uniform).
fn without_rotation(cube: &Facelets, solution: &MoveSequence) -> usize {
    let moves = solution.moves();
    let wide = moves.iter().rev().take(4).take_while(|m| m.wide).count();
    (0..=wide)
        .rev()
        .map(|k| moves.len() - k)
        .find(|&len| {
            cube.apply(&MoveSequence::new(moves[..len].to_vec()))
                .is_ok_and(|c| uniform(&c))
        })
        .unwrap_or(moves.len())
}

fn read_cubes(path: &str) -> Vec<Facelets> {
    fs::read_to_string(path)
        .expect("cubes")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Facelets::parse(l.trim()).expect("facelets"))
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cubes = read_cubes(&args[2]);
    let solver = FourSolver::new(Arc::new(Min2PhaseSolver::shared()));
    let started = Instant::now();
    solver.prepare();
    println!(
        "prepare_ms={} threads={} table_bytes={}",
        started.elapsed().as_millis(),
        solver.threads(),
        solver.table_bytes().unwrap_or(0)
    );
    let (mut total, mut plain, mut millis) = (vec![], vec![], vec![]);
    let mut bad = 0;
    let mut out = String::new();
    match args[1].as_str() {
        "ours" => {
            for cube in &cubes {
                let t = Instant::now();
                let result = solver.solve(cube, &SolveOptions::default());
                let ms = t.elapsed().as_secs_f64() * 1e3;
                match result.solution() {
                    Some(s) if cube.apply(s).is_ok_and(|c| c.is_solved()) => {
                        let bare = without_rotation(cube, s);
                        millis.push(ms);
                        total.push(s.len() as f64);
                        plain.push(bare as f64);
                        out.push_str(&format!("{ms:.3}\t{}\t{bare}\t{s}\n", s.len()));
                    }
                    _ => {
                        bad += 1;
                        out.push_str(&format!("{ms:.3}\t0\t0\tFAILED {:?}\n", result.message));
                    }
                }
            }
        }
        "external" => {
            let answers = fs::read_to_string(&args[3]).expect("answers");
            for (cube, line) in cubes.iter().zip(answers.lines()) {
                let fields: Vec<&str> = line.splitn(6, '\t').collect();
                let [ms, _l1, _l2, _l3, _l333, solution] = fields[..] else {
                    bad += 1;
                    continue;
                };
                let Ok(moves) = java_to_notation(solution).parse::<MoveSequence>() else {
                    bad += 1;
                    continue;
                };
                if !cube.apply(&moves).is_ok_and(|c| c.is_solved()) {
                    bad += 1;
                    continue;
                }
                millis.push(ms.parse().expect("millis"));
                total.push(moves.len() as f64);
                plain.push(without_rotation(cube, &moves) as f64);
            }
        }
        "phases" => {
            let answers = fs::read_to_string(&args[3]).expect("answers");
            let ours = fs::read_to_string(&args[4]).expect("ours");
            let (mut same_head, mut same_all, mut n) = (0, 0, 0);
            for (java, ours) in answers.lines().zip(ours.lines()) {
                let f: Vec<&str> = java.splitn(6, '\t').collect();
                let o: Vec<&str> = ours.splitn(4, '\t').collect();
                let (Some(&solution), Some(&mine)) = (f.get(5), o.get(3)) else {
                    continue;
                };
                let head: usize = f[1..4].iter().map(|v| v.parse::<usize>().unwrap()).sum();
                let java_tokens: Vec<&str> = solution.split_whitespace().collect();
                let our_tokens: Vec<&str> = mine.split_whitespace().collect();
                n += 1;
                let java_head = &java_tokens[..head];
                if our_tokens.len() >= head && &our_tokens[..head] == java_head {
                    same_head += 1;
                }
                // whole answer without the rotation
                let l333: usize = f[4].parse().unwrap();
                let java_plain = &java_tokens[..head + l333];
                if our_tokens.len() >= head + l333 && &our_tokens[..head + l333] == java_plain {
                    same_all += 1;
                }
            }
            println!("cubes={n} same_phases_1_to_3={same_head} same_without_rotation={same_all}");
            return;
        }
        other => panic!("unknown mode {other}"),
    }
    if let Some(path) = args.get(3).filter(|_| args[1] == "ours") {
        fs::write(path, out).expect("write");
    }
    println!("failed_or_unverified={bad}");
    stats("solve_ms", millis);
    stats("moves_with_rotation", total.clone());
    stats("moves_without_rotation", plain.clone());
    histogram("without rotation", &plain);
}
