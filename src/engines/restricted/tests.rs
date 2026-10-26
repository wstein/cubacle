use std::time::Duration;

use super::*;
use crate::notation::parse_alphabet;
use crate::puzzle::{SolveStage, Turn};

fn solver(set: &str) -> SubsetSolver {
    SubsetSolver::new(parse_alphabet(set, PuzzleKind::Cube3).unwrap()).unwrap()
}

fn cube(moves: &str) -> Facelets {
    let sequence: MoveSequence = moves.parse().unwrap();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&sequence)
        .unwrap()
}

fn solve(set: &str, moves: &str) -> SolveResult {
    solver(set).solve(&cube(moves), &SolveOptions::default())
}

fn uses_only(solution: &MoveSequence, set: &str) -> bool {
    let alphabet = parse_alphabet(set, PuzzleKind::Cube3).unwrap();
    solution.moves().iter().all(|mv| alphabet.contains(mv))
}

#[test]
fn a_cube_in_the_half_turn_group_is_solved_in_the_fewest_half_turns() {
    let state = cube("R2 U2 F2 D2 L2");
    let result = solver("half-turns").solve(&state, &SolveOptions::default());
    let solution = result.solution().expect("in the group");
    assert!(state.apply(solution).unwrap().is_solved());
    assert!(solution.len() <= 5);
    assert!(uses_only(solution, "half-turns"));
    // The superflip-free checkerboard needs 6 half turns, none fewer.
    let checkerboard = cube("R2 L2 U2 D2 F2 B2");
    let result = solver("half-turns").solve(&checkerboard, &SolveOptions::default());
    assert_eq!(result.solution().unwrap().len(), 6);
}

#[test]
fn two_generators_solve_with_their_own_moves_only() {
    let scramble = "R U R' U' R U2 R' U R U' R U R' U2";
    let state = cube(scramble);
    let result = solver("<R,U>").solve(&state, &SolveOptions::default());
    let solution = result.solution().expect("the cube is in <R,U>");
    assert!(state.apply(solution).unwrap().is_solved());
    assert!(uses_only(solution, "<R,U>"));
    assert!(solution.len() <= scramble.split(' ').count());
}

#[test]
fn the_answer_is_shortest_against_a_plain_search() {
    // Every state within 4 moves of the solved cube in <R,U>: the answer must
    // have exactly the length of the shortest word, found by brute force.
    let alphabet = ["R", "R2", "R'", "U", "U2", "U'"];
    let mut seen = std::collections::HashMap::new();
    seen.insert(Facelets::solved(PuzzleKind::Cube3).to_string(), 0usize);
    let mut layer = vec![String::new()];
    for depth in 1..=4 {
        let mut next = Vec::new();
        for word in &layer {
            for letter in alphabet {
                let longer = format!("{word} {letter}").trim().to_owned();
                let state = cube(&longer).to_string();
                if let std::collections::hash_map::Entry::Vacant(slot) = seen.entry(state) {
                    slot.insert(depth);
                    next.push(longer);
                }
            }
        }
        layer = next;
    }
    let solver = solver("<R,U>");
    for word in layer.iter().step_by(7) {
        let state = cube(word);
        let result = solver.solve(&state, &SolveOptions::default());
        assert_eq!(
            result.solution().unwrap().len(),
            seen[&state.to_string()],
            "{word}"
        );
    }
}

#[test]
fn a_cube_outside_the_group_is_reported_not_searched() {
    let result = solve("half-turns", "R");
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result
        .message
        .unwrap()
        .starts_with("not solvable with half-turns"));
    let result = solve("<R,U>", "F");
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("not solvable"));
}

#[test]
fn a_state_the_invariants_cannot_tell_is_found_out_by_the_table() {
    // R U2 R' is in no half-turn group although every sum of it fits... the
    // complete half-turn table decides it exactly.
    let result = solve("half-turns", "R U2 R'");
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("not solvable"));
}

#[test]
fn an_impossible_coloring_is_invalid_input() {
    let mut text = cube("").to_string();
    text.replace_range(0..1, "R");
    let facelets = Facelets::parse(&text);
    // Color counts are wrong, so it does not even parse as a cube.
    assert!(facelets.is_err());
    let mut stickers = cube("").stickers().to_vec();
    stickers.swap(0, 9);
    let swapped = Facelets::from_stickers(stickers).unwrap();
    let result = solver("<R,U>").solve(&swapped, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
}

#[test]
fn other_sizes_are_refused() {
    let result = solver("<R,U>").solve(
        &Facelets::solved(PuzzleKind::Cube4),
        &SolveOptions::default(),
    );
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
    let alphabet = parse_alphabet("<R,U>", PuzzleKind::Cube4).unwrap();
    assert!(SubsetSolver::new(alphabet).is_err());
    let none = Alphabet::unrestricted(PuzzleKind::Cube3);
    assert!(SubsetSolver::new(none).is_err());
}

#[test]
fn a_spent_budget_is_a_budget_failure() {
    let options = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = solver("<R,U>").solve(&cube("R U R' U' R U2 R' U R U' R U R' U2"), &options);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn a_move_limit_below_the_distance_is_reported() {
    let options = SolveOptions::default().with_max_moves(3);
    let result = solver("<R,U>").solve(&cube("R U R' U' R U2"), &options);
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("at most 3"));
}

#[test]
fn a_solved_cube_needs_no_move() {
    let result = solve("<R,U>", "");
    assert_eq!(result.solution().map(MoveSequence::len), Some(0));
}

#[test]
fn a_small_table_still_gives_the_shortest_answer() {
    // With room for 2000 states the table stops at depth 3 or so and the
    // search does the rest; the answer has the length of the big table's.
    let state = cube("R U R' U' R U2 R' U R U'");
    let small = solver("<R,U>").with_table_entries(2_000);
    let big = solver("<R,U>");
    let a = small.solve(&state, &SolveOptions::default());
    let b = big.solve(&state, &SolveOptions::default());
    assert_eq!(a.solution().unwrap().len(), b.solution().unwrap().len());
}

#[test]
fn a_big_alphabet_rewrites_the_helpers_answer() {
    /// An engine that answers with the inverse of a known scramble.
    struct Stub(MoveSequence);
    impl Solver for Stub {
        fn kind(&self) -> PuzzleKind {
            PuzzleKind::Cube3
        }
        fn solve(&self, _: &Facelets, _: &SolveOptions) -> SolveResult {
            SolveResult {
                outcome: Ok(self.0.clone()),
                message: None,
                metrics: SolveMetrics::default(),
                stages: Vec::<SolveStage>::new(),
            }
        }
    }
    let state = cube("R2 U F2 D'");
    let answer: MoveSequence = "D F2 U' R2".parse().unwrap();
    let solver = solver("qtm").with_helper(Arc::new(Stub(answer)));
    let result = solver.solve(&state, &SolveOptions::default());
    let solution = result.solution().expect("rewritten");
    assert!(state.apply(solution).unwrap().is_solved());
    assert!(uses_only(solution, "qtm"));
    // D, two quarter turns for F2, U', two for R2.
    assert_eq!(solution.len(), 6);
    assert!(result
        .message
        .as_deref()
        .unwrap()
        .contains("not proven shortest"));
    assert!(solution.moves().iter().all(|mv| mv.turn != Turn::Half));
}
