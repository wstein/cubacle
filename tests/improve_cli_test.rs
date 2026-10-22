use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use std::process::Command;
use std::time::{Duration, Instant};

const SCRAMBLE: &str = "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn solves(solution: &str) -> usize {
    let moves: MoveSequence = solution.parse().expect("moves");
    let cube = Facelets::solved(PuzzleKind::Cube3)
        .apply(&SCRAMBLE.parse().unwrap())
        .unwrap()
        .apply(&moves)
        .unwrap();
    assert!(cube.is_solved(), "{solution}");
    moves.len()
}

#[test]
fn probes_are_deterministic_and_never_longer() {
    let (ok, first, error) = cubacle(&["solve", "-s", SCRAMBLE]);
    assert!(ok, "{error}");
    let (ok, a, error) = cubacle(&["solve", "-s", SCRAMBLE, "--probes", "500"]);
    assert!(ok, "{error}");
    let (_, b, _) = cubacle(&["solve", "-s", SCRAMBLE, "--probes", "500"]);
    assert_eq!(a, b);
    assert!(solves(&a) <= solves(&first));
}

#[test]
fn improve_and_parallel_return_a_valid_solution_within_the_budget() {
    for extra in [&["--improve"][..], &["--improve", "--parallel"]] {
        let mut args = vec!["solve", "-s", SCRAMBLE, "-b", "200", "-v"];
        args.extend(extra);
        let started = Instant::now();
        let (ok, solution, error) = cubacle(&args);
        assert!(ok, "{extra:?}: {error}");
        solves(&solution);
        assert!(error.contains("improved from"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}

#[test]
fn improve_flags_are_refused_where_they_do_not_fit() {
    let cases: [(&[&str], &str); 5] = [
        (&["--parallel", "-b", "100"], "together with --improve"),
        (&["--improve", "--parallel"], "needs a time budget"),
        (&["--probes", "5", "--improve"], "cannot be used"),
        (&["--size", "2", "--improve"], "already optimal"),
        (&["--size", "4", "--probes", "5"], "not implemented yet"),
    ];
    for (flags, message) in cases {
        let mut args = vec!["solve", "-s", "R U"];
        for flag in flags {
            args.push(flag);
        }
        let (ok, _, error) = cubacle(&args);
        assert!(!ok, "{flags:?}");
        assert!(error.contains(message), "{flags:?}: {error}");
    }
}
