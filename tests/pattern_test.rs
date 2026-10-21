use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Solver};

fn scrambled(moves: &str) -> Facelets {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves.parse().expect("valid scramble"))
        .expect("moves fit a 3x3")
}

fn random() -> Facelets {
    generate::random_state(PuzzleKind::Cube3).expect("the 3x3x3 has a generator")
}

fn options(max_moves: u8) -> SolveOptions {
    SolveOptions::default().with_max_moves(max_moves)
}

#[test]
fn pattern_solution_transforms_source_into_target() {
    let solver = Min2PhaseSolver::shared();
    for _ in 0..20 {
        let (from, to) = (random(), random());
        let result = solver.solve_pattern(&from, &to, &options(25));

        let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
        assert!(solution.len() <= 25);
        assert_eq!(from.apply(solution).unwrap(), to);
    }
}

#[test]
fn pattern_to_solved_matches_the_regular_solver() {
    let solver = Min2PhaseSolver::shared();
    let from = scrambled("R U R' U' F2 D L2");
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let pattern = solver.solve_pattern(&from, &solved, &options(20));
    let regular = solver.solve(&from, &options(20));

    let solution = pattern.solution().expect("solved");
    assert_eq!(solution.len(), regular.solution().unwrap().len());
    assert!(from.apply(solution).unwrap().is_solved());
}

#[test]
fn identical_pattern_requires_no_moves() {
    let cube = scrambled("R U R' U'");
    let result = Min2PhaseSolver::shared().solve_pattern(&cube, &cube, &options(20));

    let solution = result.solution().expect("solved");
    assert!(solution.is_empty());
    assert_eq!(cube.apply(solution).unwrap(), cube);
}

#[test]
fn a_pattern_endpoint_of_the_wrong_size_is_reported() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let pocket = Facelets::solved(PuzzleKind::Cube2);
    let result = Min2PhaseSolver::shared().solve_pattern(&pocket, &solved, &options(20));

    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn an_impossible_pattern_endpoint_is_reported() {
    let mut stickers: Vec<char> = Facelets::solved(PuzzleKind::Cube3)
        .to_string()
        .chars()
        .collect();
    stickers.swap(5, 10);
    let flipped = Facelets::parse(&stickers.into_iter().collect::<String>()).unwrap();
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let result = Min2PhaseSolver::shared().solve_pattern(&flipped, &solved, &options(20));

    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn parallel_pattern_solution_transforms_source_into_target() {
    let from = scrambled("R U R' U' F2 D L2");
    let to = scrambled("B2 U2 R2 F2");
    let result = Min2PhaseSolver::shared().solve_pattern_parallel(&from, &to, &options(20));

    let solution = result.solution().unwrap_or_else(|| panic!("{result:?}"));
    assert_eq!(from.apply(solution).unwrap(), to);
}
