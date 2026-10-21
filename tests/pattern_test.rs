use cubacle::{
    apply_moves, from_moves, random_cube, solve_pattern, solve_pattern_parallel, solve_with_stats,
    SolveOutcome,
};

const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

#[test]
fn pattern_solution_transforms_source_into_target() {
    for _ in 0..20 {
        let from = random_cube();
        let to = random_cube();
        let result = solve_pattern(&from, &to, 25);

        assert_eq!(result.outcome, SolveOutcome::Solved, "{result:?}");
        assert!(result.solution.split_whitespace().count() <= 25);
        assert_eq!(apply_moves(&from, &result.solution), Some(to));
    }
}

#[test]
fn pattern_to_solved_matches_the_regular_solver() {
    let from = from_moves("R U R' U' F2 D L2").expect("valid scramble");
    let pattern = solve_pattern(&from, SOLVED, 20);
    let regular = solve_with_stats(&from, 20);

    assert_eq!(pattern.outcome, SolveOutcome::Solved);
    assert_eq!(
        pattern.solution.split_whitespace().count(),
        regular.solution.split_whitespace().count()
    );
    assert_eq!(
        apply_moves(&from, &pattern.solution).as_deref(),
        Some(SOLVED)
    );
}

#[test]
fn identical_pattern_requires_no_moves() {
    let cube = from_moves("R U R' U'").expect("valid scramble");
    let result = solve_pattern(&cube, &cube, 20);

    assert_eq!(result.outcome, SolveOutcome::Solved);
    assert!(result.solution.trim().is_empty());
    assert_eq!(apply_moves(&cube, &result.solution), Some(cube));
}

#[test]
fn invalid_pattern_endpoint_is_reported() {
    let result = solve_pattern("not a cube", SOLVED, 20);

    assert_eq!(result.solution, "Error 1");
    assert_eq!(result.outcome, SolveOutcome::Unsolvable);
}

#[test]
fn parallel_pattern_solution_transforms_source_into_target() {
    let from = from_moves("R U R' U' F2 D L2").expect("valid source scramble");
    let to = from_moves("B2 U2 R2 F2").expect("valid target scramble");
    let result = solve_pattern_parallel(&from, &to, 20);

    assert_eq!(result.outcome, SolveOutcome::Solved, "{result:?}");
    assert_eq!(apply_moves(&from, &result.solution), Some(to));
}
