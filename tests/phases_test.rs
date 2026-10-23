//! The phases of real engine solutions, through the public API.
//!
//! The boundaries are checked against the cube itself and against what the
//! engines promise (the 3x3x3 stage of a reduction uses only outer turns, the
//! second phase of the two-phase search only `U D R2 L2 F2 B2`).

use std::sync::Arc;

use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::phases::{phases, Phase};
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn};

fn state_after(start: &Facelets, moves: &[Move], count: usize) -> Facelets {
    start
        .apply(&MoveSequence::new(moves[..count].to_vec()))
        .expect("replays")
}

fn covered(found: &[Phase], total: usize) {
    let mut at = 0;
    for phase in found {
        assert_eq!(phase.moves.start, at, "{found:?}");
        assert!(phase.moves.end >= at, "{found:?}");
        at = phase.moves.end;
    }
    assert_eq!(at, total, "{found:?}");
}

/// Every centre sticker of every face shows one colour (the cube is `n` wide).
fn centres_uniform(cube: &Facelets) -> bool {
    let n = cube.kind().size();
    let stickers = cube.stickers();
    (0..6).all(|face| {
        let colours: Vec<Face> = (1..n - 1)
            .flat_map(|row| (1..n - 1).map(move |col| (row, col)))
            .map(|(row, col)| stickers[face * n * n + row * n + col])
            .collect();
        colours.windows(2).all(|w| w[0] == w[1])
    })
}

fn g1_move(mv: &Move) -> bool {
    match mv.face {
        Face::U | Face::D => true,
        _ => mv.turn == Turn::Half,
    }
}

#[test]
fn a_3x3_solution_is_cut_into_g1_and_a_g1_tail() {
    let solver = Min2PhaseSolver::shared();
    let mut cut_somewhere = 0;
    for _ in 0..200 {
        let cube = generate::random_state(PuzzleKind::Cube3).expect("generator");
        let result = solver.solve(&cube, &SolveOptions::default().with_max_moves(21));
        let solution = result.solution().expect("solved");
        let found = phases(&cube, solution).expect("replays");
        covered(&found, solution.len());
        let moves = solution.moves();
        // the second phase of the search uses only G1 moves, so the cube is in
        // G1 after the last move that is not one
        let tail = moves.iter().rposition(|m| !g1_move(m)).map_or(0, |i| i + 1);
        match found.as_slice() {
            [] => assert!(solution.is_empty()),
            [one, two] => {
                assert_eq!((one.number, two.number), (1, 2));
                assert!(one.moves.end <= tail.max(1), "{found:?} tail {tail}");
                assert!(state_after(&cube, moves, one.moves.end).kind() == PuzzleKind::Cube3);
                cut_somewhere += 1;
            }
            _ => panic!("{found:?}"),
        }
    }
    assert!(cut_somewhere > 100, "{cut_somewhere}");
}

#[test]
fn a_4x4_solution_has_its_reduction_then_a_3x3_stage() {
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(2);
    for _ in 0..2 {
        let cube = generate::random_state(PuzzleKind::Cube4).expect("generator");
        let result = four.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("solved");
        let found = phases(&cube, solution).expect("replays");
        covered(&found, solution.len());
        // this engine makes the centres and the edges true at the same move,
        // so the edges phase is empty
        let names: Vec<&str> = found.iter().map(|p| p.name).collect();
        assert_eq!(names, ["centres", "edges", "3x3 stage"], "{found:?}");
        assert!(found[1].moves.is_empty(), "{found:?}");
        let moves = solution.moves();
        let cut = found[0].moves.end;
        assert!(centres_uniform(&state_after(&cube, moves, cut)));
        assert!(!centres_uniform(&state_after(&cube, moves, cut - 1)));
        // the 3x3x3 stage turns outer layers only
        for mv in &moves[cut..] {
            assert!(
                mv.depth == 1 && !mv.wide,
                "{mv} in the 3x3 stage of {solution}"
            );
        }
    }
}

#[test]
fn a_5x5_solution_is_cut_by_the_state() {
    let cache = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(cache));
    let scramble: MoveSequence = "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F".parse().unwrap();
    let cube = Facelets::solved(PuzzleKind::Cube5)
        .apply(&scramble)
        .unwrap();
    let result = five.solve(&cube, &SolveOptions::default());
    let solution = result.solution().expect("solved");
    let found = phases(&cube, solution).expect("replays");
    covered(&found, solution.len());
    let names: Vec<&str> = found.iter().map(|p| p.name).collect();
    assert_eq!(names, ["centres", "edges", "3x3 stage"], "{found:?}");
    let moves = solution.moves();
    let cut = found[0].moves.end;
    assert!(centres_uniform(&state_after(&cube, moves, cut)));
    assert!(!centres_uniform(&state_after(&cube, moves, cut - 1)));
    let stage = found.iter().find(|p| p.name == "3x3 stage");
    for mv in stage.map_or(&[][..], |p| &moves[p.moves.clone()]) {
        assert!(
            mv.depth == 1 && !mv.wide,
            "{mv} in the 3x3 stage of {solution}"
        );
    }
}
