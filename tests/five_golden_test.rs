//! Golden record of the 5x5x5 engine.
//!
//! `tests/fixtures/five.txt` holds cubes with the length and a hash of the
//! solution the engine produced, and a few cubes it must refuse. The engine and
//! the 3x3x3 solver behind it are deterministic without a time budget, so a
//! change that alters the output shows up here. Never edit the fixture to make
//! a refactor pass.

use std::sync::Arc;

use cubacle::engines::five::FiveSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, SolveOptions, Solver};

const GOLDEN: &str = include_str!("fixtures/five.txt");

/// FNV-1a, 64 bit: stable across platforms, unlike the std hasher.
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[test]
fn five_output_matches_the_golden_record() {
    let solver = FiveSolver::new(Arc::new(Min2PhaseSolver::shared()));
    let mut rows = 0;
    for (number, line) in GOLDEN.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [max_moves, facelets, outcome] = fields[..] else {
            panic!("line {}: malformed", number + 1);
        };
        let mut options = SolveOptions::default();
        if max_moves != "-" {
            options = options.with_max_moves(max_moves.parse().expect("max moves"));
        }
        let cube = Facelets::parse(facelets).expect("golden facelets");
        let result = solver.solve(&cube, &options);
        let got = match &result.outcome {
            Ok(solution) => {
                assert!(
                    cube.apply(solution).unwrap().is_solved(),
                    "line {}: the solution does not replay",
                    number + 1
                );
                format!("ok {} {:016x}", solution.len(), fnv(&solution.to_string()))
            }
            Err(failure) => format!("err {failure:?}"),
        };
        assert_eq!(got, outcome, "line {}", number + 1);
        rows += 1;
    }
    assert!(rows >= 60, "only {rows} golden rows");
}
