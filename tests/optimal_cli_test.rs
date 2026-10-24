//! `solve --optimal`: a proven shortest solution, with the flags it excludes.

use std::process::Command;

struct Run {
    ok: bool,
    out: String,
    err: String,
}

/// Runs the binary with a cache directory of its own, so that the tables the
/// run needs are built once per machine and not found in a user's cache.
fn cubacle(args: &[&str]) -> Run {
    let cache = std::env::temp_dir().join("cubacle-optimal-cli-test-cache");
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .env("CUBACLE_CACHE_DIR", cache)
        .args(args)
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

#[test]
fn only_the_3x3x3_has_an_optimal_mode() {
    let run = cubacle(&["solve", "--optimal", "--size", "4", "--scramble", "R U"]);
    assert!(!run.ok);
    assert!(
        run.err.contains("--optimal works on the 3x3x3 only"),
        "{}",
        run.err
    );
}

#[test]
fn optimal_excludes_the_other_search_modes() {
    for flag in [
        &["--fast"][..],
        &["--probes", "5"],
        &["--improve"],
        &["--moves", "qtm"],
    ] {
        let mut args = vec!["solve", "--optimal", "--scramble", "R U"];
        args.extend(flag);
        let run = cubacle(&args);
        assert!(!run.ok, "{flag:?}");
        assert!(
            run.err.contains("cannot be used with"),
            "{flag:?}: {}",
            run.err
        );
    }
}

#[test]
fn nodes_need_optimal() {
    let run = cubacle(&["solve", "--nodes", "5", "--scramble", "R U"]);
    assert!(!run.ok);
}

#[test]
fn a_short_scramble_is_proven_and_a_node_budget_is_not() {
    // 90 MB is enough for the small tables only.
    let run = cubacle(&[
        "solve",
        "--optimal",
        "--max-memory",
        "90",
        "--scramble",
        "R U F' D2",
    ]);
    assert!(run.ok, "{}", run.err);
    assert!(
        run.err
            .contains("optimal: no solution is shorter than 4 moves"),
        "{}",
        run.err
    );
    assert_eq!(run.out.split_whitespace().count(), 4, "{}", run.out);

    // The tables are cached now: a node budget ends a long search early.
    let run = cubacle(&[
        "solve",
        "--optimal",
        "--max-memory",
        "90",
        "--nodes",
        "20000",
        "--scramble",
        "R U F' D2 L B' R2 U' F2 D L2 B R' U2 F D' L' B2",
    ]);
    assert!(run.ok, "{}", run.err);
    assert!(run.err.contains("not proven shortest"), "{}", run.err);
    assert!(!run.out.is_empty());
}

#[test]
fn too_little_memory_is_an_error() {
    let run = cubacle(&[
        "solve",
        "--optimal",
        "--max-memory",
        "10",
        "--scramble",
        "R U",
    ]);
    assert!(!run.ok);
    assert!(run.err.contains("needs at least"), "{}", run.err);
}
