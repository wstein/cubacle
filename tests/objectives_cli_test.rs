//! Opt-in objectives use explicit costs and preserve the default output.
use std::process::Command;

fn run(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn moves_objective_reduces_engine_turn_count() {
    let (ok, out, err) = run(&["moves", "--size", "4", "--optimize", "moves", "R L R' L'"]);
    assert!(ok, "{err}");
    assert_eq!(out, "");
}

#[test]
fn easy_objective_absorbs_final_rotation() {
    let (ok, out, err) = run(&["moves", "--size", "4", "--optimize", "easy", "B2 F2 D2"]);
    assert!(ok, "{err}");
    assert_eq!(out, "2-3Fw2 U2");
}

#[test]
fn easy_rejects_exact_target_and_inverse() {
    for extra in [["--to", "R"], ["--inverse", ""]] {
        let mut args = vec!["solve", "--size", "4", "--optimize", "easy"];
        args.push(extra[0]);
        if !extra[1].is_empty() {
            args.push(extra[1]);
        }
        args.push("R");
        let (ok, _, err) = run(&args);
        assert!(!ok, "{err}");
        assert!(err.contains("cannot be combined"), "{err}");
    }
}

#[test]
fn easy_icons_render_the_optimized_steps() {
    let (ok, out, err) = run(&[
        "moves",
        "--size",
        "4",
        "--optimize",
        "easy",
        "--icons",
        "--graphics",
        "plain",
        "B2 F2 D2",
    ]);
    assert!(ok, "{err}");
    assert!(out.contains("2-3Fw2"), "{out}");
    assert!(out.contains("U2"), "{out}");
}

#[test]
fn moves_does_not_silently_ignore_partial_targets() {
    let (ok, _, err) = run(&[
        "solve",
        "--size",
        "3",
        "--optimize",
        "moves",
        "--to",
        "cross",
        "R",
    ]);
    assert!(!ok, "{err}");
    assert!(err.contains("partial or orientation-free target"), "{err}");
}

#[test]
fn objectives_reject_center_twist_goals() {
    for objective in ["moves", "easy"] {
        let (ok, _, err) = run(&[
            "solve",
            "--size",
            "3",
            "--optimize",
            objective,
            "--upright-centers",
            "R",
        ]);
        assert!(!ok, "{err}");
        assert!(err.contains("center twists"), "{err}");
    }
}
