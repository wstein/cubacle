//! Compressed real solutions replay to the same state.

use std::sync::Arc;

use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate::scramble;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};
use cubacle::ranges::{compress, expand};

#[test]
fn a_compressed_4x4x4_solution_still_solves_the_cube() {
    let solver = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    for _ in 0..3 {
        let kind = PuzzleKind::Cube4;
        let cube = Facelets::solved(kind).apply(&scramble(kind, 60)).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("solvable");
        let packed = compress(kind, solution);
        assert!(packed.len() <= solution.len());
        let replayed = cube.apply(&expand(&packed)).unwrap();
        assert!(replayed.is_solved());
    }
}

#[test]
fn moves_objective_reduces_engine_turn_count_exactly() {
    use cubacle::puzzle::MoveSequence;
    use cubacle::ranges::{optimize, Optimize};
    let kind = PuzzleKind::Cube4;
    let input: MoveSequence = "R Rw' U U'".parse().unwrap();
    let output = optimize(kind, &input, Optimize::Moves);
    assert_eq!(output.len(), 1);
    let start = Facelets::solved(kind)
        .apply(&"F L2".parse().unwrap())
        .unwrap();
    assert_eq!(
        start.apply(&input).unwrap(),
        start.apply(&expand(&output)).unwrap()
    );
}

#[test]
fn easy_objective_rotates_the_following_move() {
    use cubacle::notation::Notation;
    use cubacle::puzzle::{MoveSequence, Target};
    use cubacle::ranges::{optimize, write, Optimize};
    let kind = PuzzleKind::Cube4;
    let input: MoveSequence = "B2 F2 D2".parse().unwrap();
    let output = optimize(kind, &input, Optimize::Easy);
    assert_eq!(write(Notation::Wca, kind, &output), "2-3Fw2 U2");
    let start = Facelets::solved(kind)
        .apply(&"R U Fw".parse().unwrap())
        .unwrap();
    let expected = start.apply(&input).unwrap();
    let actual = start.apply(&expand(&output)).unwrap();
    assert!(Target::complete(&expected).matches_any_rotation(&actual));
    assert_ne!(expected, actual);
}

#[test]
fn easy_objective_replays_modulo_rotation_on_all_sizes() {
    use cubacle::puzzle::{MoveSequence, Target};
    use cubacle::ranges::{optimize, Optimize};
    for kind in PuzzleKind::ALL {
        let start = Facelets::solved(kind)
            .apply(&"R U F L2".parse().unwrap())
            .unwrap();
        for text in [
            "F B' D R U' L F2 B2 D2",
            "R L' U F D B R2 L2 U2",
            "U D' R F L B U2 D2 F2",
            "R R' U F F' U'",
            "B2 F2 D2",
        ] {
            let input: MoveSequence = text.parse().unwrap();
            let output = optimize(kind, &input, Optimize::Easy);
            assert!(output.len() <= input.len(), "{kind:?} {text}");
            let expected = start.apply(&input).unwrap();
            let actual = start.apply(&expand(&output)).unwrap();
            assert!(
                Target::complete(&expected).matches_any_rotation(&actual),
                "{kind:?} {text}"
            );
        }
    }
}

#[test]
fn objectives_replay_generated_axis_runs() {
    use cubacle::metrics::{count, count_layer_moves, Metric};
    use cubacle::puzzle::{Face, Move, MoveSequence, Target, Turn};
    use cubacle::ranges::{optimize, Optimize};
    let mut seed = 19u64;
    for kind in PuzzleKind::ALL {
        let start = Facelets::solved(kind)
            .apply(&"R U F L2".parse().unwrap())
            .unwrap();
        for _ in 0..100 {
            let mut input = Vec::new();
            for _ in 0..32 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let face = Face::ALL[((seed >> 32) % 6) as usize];
                let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise]
                    [((seed >> 40) % 3) as usize];
                let depth = 1 + ((seed >> 45) as usize % (kind.size() - 1));
                let wide = depth > 1 && (seed >> 50) & 1 == 0;
                input.push(Move::new(face, depth as u8, wide, turn));
            }
            let input = MoveSequence::new(input);
            let expected = start.apply(&input).unwrap();
            let moves = optimize(kind, &input, Optimize::Moves);
            assert!(moves.len() <= input.len());
            assert_eq!(expected, start.apply(&expand(&moves)).unwrap());
            let easy = optimize(kind, &input, Optimize::Easy);
            assert!(
                count_layer_moves(&easy, kind, Metric::Btm) <= count(&input, kind, Metric::Btm)
            );
            let actual = start.apply(&expand(&easy)).unwrap();
            assert!(
                Target::complete(&expected).matches_any_rotation(&actual),
                "{kind:?} {input}"
            );
        }
    }
}

#[test]
fn easy_uses_overlapping_blocks_when_they_cost_less() {
    use cubacle::metrics::{count_layer_moves, Metric};
    use cubacle::puzzle::{MoveSequence, Target};
    use cubacle::ranges::{optimize, Optimize};
    let kind = PuzzleKind::Cube4;
    let input: MoveSequence = "2R 3R' L2".parse().unwrap();
    let output = optimize(kind, &input, Optimize::Easy);
    assert_eq!(count_layer_moves(&output, kind, Metric::Btm), 2);
    let start = Facelets::solved(kind)
        .apply(&"U F L".parse().unwrap())
        .unwrap();
    let expected = start.apply(&input).unwrap();
    let actual = start.apply(&expand(&output)).unwrap();
    assert!(Target::complete(&expected).matches_any_rotation(&actual));
}
