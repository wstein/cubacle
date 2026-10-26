//! Targets with mixed centers on the 4x4x4 to 7x7x7, through the public API:
//! the route takes the start to the target exactly (color by color).
//!
//! Every test prints its route lengths and times with `--nocapture`. The
//! targets use inner slices and wide turns; the middle layer of the 5x5x5 and
//! 7x7x7 is left alone, since the engines keep the face centers where they are.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use cubacle::centers;
use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_expression;
use cubacle::puzzle::{reaches, Facelets, PuzzleKind, SolveOptions, Solver};

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).expect("moves");
    Facelets::solved(kind).apply(&moves).expect("moves fit")
}

fn four() -> &'static FourSolver {
    static ENGINE: OnceLock<FourSolver> = OnceLock::new();
    ENGINE.get_or_init(|| FourSolver::new(Arc::new(Min2PhaseSolver::shared())))
}

fn five() -> &'static FiveSolver {
    static ENGINE: OnceLock<FiveSolver> = OnceLock::new();
    ENGINE.get_or_init(|| {
        // never the user's real table cache
        let cache = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
        FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(cache))
    })
}

fn big(kind: PuzzleKind) -> DwaltonSolver {
    let three = Arc::new(Min2PhaseSolver::shared());
    let cache = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    DwaltonSolver::new(
        kind,
        three.clone(),
        Arc::new(FourSolver::new(three.clone())),
        Arc::new(FiveSolver::new(three).with_cache_dir(Some(cache))),
    )
}

/// Routes `start` to `target` with `engine` and checks by replay; returns the
/// length. With `back` also routes from the target to the start.
fn check(engine: &dyn Solver, kind: PuzzleKind, start: &str, target: &str, back: bool) -> usize {
    let (a, b) = (cube(kind, start), cube(kind, target));
    let hint = parse_expression(target, kind).ok();
    let mut total = 0;
    for (from, to, hint) in [(&a, &b, hint.as_ref()), (&b, &a, None)]
        .into_iter()
        .take(if back { 2 } else { 1 })
    {
        let began = Instant::now();
        let result = centers::solve_to(engine, from, to, &SolveOptions::default(), hint);
        let route = result.solution().unwrap_or_else(|| {
            panic!(
                "{kind:?} {start} -> {target}: {:?} {:?}",
                result.outcome, result.message
            )
        });
        assert!(reaches(from, route, to), "{kind:?} {start} -> {target}");
        eprintln!(
            "{}x{0}: [{start}] -> [{target}]: {} moves in {:.1?}",
            kind.size(),
            route.len(),
            began.elapsed()
        );
        total += route.len();
    }
    total
}

#[test]
fn a_4x4_reaches_targets_built_with_slices_from_solved_and_scrambled_starts() {
    for start in ["", "R U F' D2 L B 2R' U2 Rw'"] {
        for target in [
            "2R",
            "2R 2U' 2F",
            "Rw 2U Fw' 2L",
            "Rw2 Lw 2U'",
            "2R 2L' 2U 2D' 2F 2B'",
            "Rw 2U' Fw2 Lw' 2D Bw",
        ] {
            check(four(), PuzzleKind::Cube4, start, target, true);
        }
    }
}

#[test]
fn a_4x4_target_that_is_a_rotated_frame_with_slices_is_reached() {
    check(four(), PuzzleKind::Cube4, "R U", "Rw2 Lw2 2U", false);
}

#[test]
fn a_5x5_reaches_targets_built_with_slices() {
    for start in ["", "R U F2 D' L B 2R' Uw"] {
        for target in [
            "2R 4U' 2F",
            "Rw 2U' Fw 4L2 2D",
            "2R 4R' 2U 4D'",
            "Rw2 4U Fw' 2L",
        ] {
            check(five(), PuzzleKind::Cube5, start, target, false);
        }
    }
}

#[test]
fn a_6x6_reaches_targets_built_with_slices() {
    let engine = big(PuzzleKind::Cube6);
    for (start, target) in [
        ("", "2R 3U 2F"),
        ("R U F2 D' L B", "Rw 3U' Fw2 4L"),
        ("3Rw U 2F' D", "2R 5U' 3Fw 3L2 2D"),
    ] {
        check(&engine, PuzzleKind::Cube6, start, target, false);
    }
}

/// The owner's target, in SSE notation: slice moves of the 7x7x7.
const OWNER: &str = "VB VD VL VD2 VL' VD VB2 VD' · VL' VR' · VU' VB VU2 VB' VU2 VF'";
const SCRAMBLE_7: &str = "B L2 3R' 3U' 3L 3R2 2F2 U 3R' 3D' 2R U 3F 2R2 3B2 2L 3U2 3R' 2B' \
                          2R2 3D2 3F' 2D' 3F' 2D' 2R' 3D' U' 3R' 2B 2U2 2B' F2 3L 3R F2 L 3U' B' 3D2";

#[test]
fn a_7x7_reaches_the_owners_slice_target_and_other_slice_targets() {
    let engine = big(PuzzleKind::Cube7);
    let kind = PuzzleKind::Cube7;
    check(&engine, kind, SCRAMBLE_7, OWNER, false);
    check(&engine, kind, "", "2R 3U' 2F 5L2 3D' 2B", false);
    // the middle slice turns the face centers
    check(&engine, kind, "R U", "2R 4U' 3F", false);
    check(
        &engine,
        kind,
        "R U",
        "3Rw 2U' 3Fw 2L2 3D' 2Bw' 3R2 2F",
        false,
    );
}
