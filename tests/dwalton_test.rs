use std::sync::Arc;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::puzzle::{Facelets, PuzzleKind};

fn solver() -> DwaltonSolver {
    let three = Arc::new(Min2PhaseSolver::new());
    DwaltonSolver::new(
        PuzzleKind::Cube6,
        three.clone(),
        Arc::new(FourSolver::new(three.clone())),
        Arc::new(FiveSolver::new(three)),
    )
}

#[test]
fn six_by_six_scrambles_are_solved_and_replay() {
    let solver = solver();
    for _ in 0..3 {
        let cube: Facelets = Facelets::solved(PuzzleKind::Cube6)
            .apply(&generate::scramble(PuzzleKind::Cube6, 80))
            .unwrap();
        let (result, account) = solver.solve_accounted(&cube);
        println!("{account:?}");
        let moves = result
            .outcome
            .unwrap_or_else(|f| panic!("{f:?} {:?}", result.message));
        assert!(cube.apply(&moves).unwrap().is_solved());
    }
}

/// Solves the reference cubes in `$DWALTON_CUBES` (`N-1.txt` ... `N-20.txt`,
/// N from `$DWALTON_SIZE`, 6 or 7);
/// run with `--ignored --nocapture`.
#[test]
#[ignore]
fn reference_cubes_are_solved() {
    let Ok(dir) = std::env::var("DWALTON_CUBES") else {
        return;
    };
    let size: usize = std::env::var("DWALTON_SIZE").map_or(6, |s| s.parse().unwrap());
    let solver = if size == 7 { solver7() } else { solver() };
    for i in 1..=20 {
        let text = std::fs::read_to_string(format!("{dir}/{size}-{i}.txt")).unwrap();
        let cube = Facelets::parse(text.trim()).unwrap();
        let (result, account) = solver.solve_accounted(&cube);
        let moves = result
            .outcome
            .unwrap_or_else(|f| panic!("{i}: {f:?} {:?}", result.message));
        assert!(cube.apply(&moves).unwrap().is_solved(), "cube {i}");
        println!(
            "CUBE {i} total {} ms {:?} {account:?}",
            moves.len(),
            result.metrics.elapsed
        );
    }
}

fn solver7() -> DwaltonSolver {
    let three = Arc::new(Min2PhaseSolver::new());
    DwaltonSolver::new(
        PuzzleKind::Cube7,
        three.clone(),
        Arc::new(FourSolver::new(three.clone())),
        Arc::new(FiveSolver::new(three)),
    )
}

#[test]
fn seven_by_seven_scrambles_are_solved_and_replay() {
    let solver = solver7();
    for _ in 0..2 {
        let cube: Facelets = Facelets::solved(PuzzleKind::Cube7)
            .apply(&generate::scramble(PuzzleKind::Cube7, 100))
            .unwrap();
        let (result, account) = solver.solve_accounted(&cube);
        println!("{account:?}");
        let moves = result
            .outcome
            .unwrap_or_else(|f| panic!("{f:?} {:?}", result.message));
        assert!(cube.apply(&moves).unwrap().is_solved());
    }
}
