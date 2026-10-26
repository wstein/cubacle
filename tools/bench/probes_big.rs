//! Measures `--probes` on the 6x6x6 and 7x7x7: length and time of the plain
//! solution against `--probes N`, on a fixed, seeded set of scrambles.
//!
//! `cargo run --release --example probes_big -- <size> <cubes> <probes>...`
//!
//! Prints, per cube, the plain length and each probe count's length in OBTM
//! and BTM (the metrics `solve` prints) with the seconds each took, then the
//! sums. The scrambles use outer and wide turns of at most three layers, so a
//! 7x7x7 keeps its face centers. Needs the wing tables (built on first use).

use std::env;
use std::sync::Arc;
use std::time::Instant;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::metrics::{count, Metric};
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};

/// A small deterministic generator (xorshift64).
struct Rng(u64);

impl Rng {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

fn scrambled(kind: PuzzleKind, rng: &mut Rng) -> Facelets {
    let mut text = Vec::new();
    for _ in 0..(kind.size() * 12) {
        let face = ["U", "R", "F", "D", "L", "B"][rng.next(6)];
        let suffix = ["", "'", "2"][rng.next(3)];
        let turn = match 1 + rng.next(3) {
            1 => face.to_owned(),
            2 => format!("{face}w"),
            _ => format!("3{face}w"),
        };
        text.push(format!("{turn}{suffix}"));
    }
    Facelets::solved(kind)
        .apply(&text.join(" ").parse().expect("valid moves"))
        .expect("moves exist on this cube")
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let kind = PuzzleKind::of_size(args[0].parse().expect("size")).expect("a size");
    let cubes: usize = args[1].parse().expect("number of cubes");
    let probes: Vec<u64> = args[2..]
        .iter()
        .map(|a| a.parse().expect("probes"))
        .collect();
    let three = Arc::new(Min2PhaseSolver::new());
    let four = FourSolver::new(three.clone());
    let five = FiveSolver::new(three.clone());
    four.prepare();
    five.prepare();
    let solver = DwaltonSolver::new(kind, three, Arc::new(four), Arc::new(five));
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    // sums per column: plain, then each probe count; (obtm, btm, seconds)
    let mut sums = vec![(0usize, 0usize, 0f64); probes.len() + 1];
    for index in 0..cubes {
        let cube = scrambled(kind, &mut rng);
        let mut row = format!("cube {index}:");
        for (column, setting) in std::iter::once(None)
            .chain(probes.iter().copied().map(Some))
            .enumerate()
        {
            let options = match setting {
                Some(n) => SolveOptions::default().with_min_probes(n),
                None => SolveOptions::default(),
            };
            let started = Instant::now();
            let result = solver.solve(&cube, &options);
            let seconds = started.elapsed().as_secs_f64();
            let solution = result.solution().expect("a solution");
            assert!(cube.apply(solution).unwrap().is_solved());
            let (obtm, btm) = (
                count(solution, kind, Metric::Obtm),
                count(solution, kind, Metric::Btm),
            );
            sums[column].0 += obtm;
            sums[column].1 += btm;
            sums[column].2 += seconds;
            let name = setting.map_or("plain".to_owned(), |n| format!("probes {n}"));
            row += &format!("  {name} {obtm}/{btm} {seconds:.1}s");
        }
        println!("{row}");
    }
    println!("sums (obtm/btm, seconds) over {cubes} cubes:");
    for (column, (obtm, btm, seconds)) in sums.iter().enumerate() {
        let name = column
            .checked_sub(1)
            .map_or("plain".to_owned(), |i| format!("probes {}", probes[i]));
        println!("  {name}: {obtm}/{btm}, {seconds:.1} s");
    }
}
