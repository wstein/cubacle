//! The 5x5x5 engine with the full tables, through the public API.
//!
//! Ignored by default: it needs 237 MB of tables (read from the cache
//! directory, or built in a few seconds on many cores, about 20 s on one).
//! Run with `cargo test --release -- --ignored five_full`.
//!
//! `tests/fixtures/five_transfer.txt` holds the reduction the Java solver
//! cs0x7f/cube555 made of 24 cubes. The solution of `FiveSolver` must start
//! with those moves (its last move may merge with the 3x3x3 tail). The answer
//! is normalized per axis (`src/puzzle/axis.rs`), which may merge and reorder
//! moves of one axis, so the head is compared by the cube it leads to: some
//! prefix of the solution must reach the state the Java head reaches.

use std::sync::Arc;

use cubacle::engines::five::FiveSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, MoveSequence, SolveOptions, Solver};

const TRANSFER: &str = include_str!("fixtures/five_transfer.txt");

/// Lowercase letters are two-layer turns in the Java notation.
fn to_notation(token: &str) -> String {
    let mut chars = token.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {
            format!("{}w{}", c.to_ascii_uppercase(), chars.as_str())
        }
        _ => token.to_owned(),
    }
}

#[test]
#[ignore = "needs the full 5x5x5 tables"]
fn five_full_reproduces_the_java_reduction_and_the_pruning_tables_fit_300_mb() {
    let cache = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-full");
    let solver = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(cache));
    solver.prepare();
    let bytes = solver.table_bytes().expect("tables are ready");
    assert!(bytes <= 300_000_000, "tables hold {bytes} bytes");

    let mut cubes = 0;
    for line in TRANSFER
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (facelets, moves) = line.split_once('\t').expect("facelets TAB moves");
        let cube = Facelets::parse(facelets).expect("facelets");
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("a solution");
        assert!(cube.apply(solution).expect("replays").is_solved());

        let expected: Vec<String> = moves.split_whitespace().map(to_notation).collect();
        let head = expected[..expected.len() - 1]
            .join(" ")
            .parse::<MoveSequence>()
            .expect("the Java head parses");
        let java_state = cube.apply(&head).expect("the Java head replays");
        let moves = solution.moves();
        let on_the_way = (0..=moves.len()).any(|k| {
            cube.apply(&MoveSequence::new(moves[..k].to_vec()))
                .is_ok_and(|state| state == java_state)
        });
        assert!(
            on_the_way,
            "reduction differs from the Java one for {facelets}"
        );
        cubes += 1;
    }
    assert!(cubes >= 20, "{cubes} cubes");
}
