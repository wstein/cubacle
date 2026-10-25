//! Tests of the 5x5x5 solver: stubs for the hand-off, depth-capped tables.

use std::sync::Mutex;
use std::time::Duration;

use super::*;
use crate::engines::three::Min2PhaseSolver;
use crate::reduction::improve::Plan;

/// A deterministic stream of numbers below a bound.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
}

/// `length` random turns of the outer layer or the two outer layers.
fn scrambled(seed: u64, length: usize) -> Facelets {
    let mut rng = Lcg(seed);
    let moves: Vec<Move> = (0..length).map(|_| move_of(rng.next(36) as u8)).collect();
    Facelets::solved(PuzzleKind::Cube5)
        .apply(&MoveSequence::new(moves))
        .unwrap()
}

/// Records the 3x3x3 it is asked to solve and answers with a fixed result.
struct Stub {
    kind: PuzzleKind,
    seen: Mutex<Vec<Facelets>>,
    answer: Result<&'static str, Failure>,
}

impl Stub {
    fn new(answer: Result<&'static str, Failure>) -> Arc<Stub> {
        Arc::new(Stub {
            kind: PuzzleKind::Cube3,
            seen: Mutex::new(Vec::new()),
            answer,
        })
    }
}

impl Solver for Stub {
    fn kind(&self) -> PuzzleKind {
        self.kind
    }

    fn solve(&self, facelets: &Facelets, _: &SolveOptions) -> SolveResult {
        self.seen.lock().unwrap().push(facelets.clone());
        SolveResult {
            outcome: self
                .answer
                .map(|text| text.parse::<MoveSequence>().unwrap()),
            message: None,
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}

fn small(three: Arc<dyn Solver + Send + Sync>) -> FiveSolver {
    FiveSolver::new(three).with_cap(4).with_threads(4)
}

fn real() -> FiveSolver {
    small(Arc::new(Min2PhaseSolver::shared()))
}

#[test]
fn the_3x3x3_hand_off_gets_the_reduced_cube_through_the_trait() {
    // Outer turns leave the centres and wings paired: nothing to reduce.
    let cube = Facelets::solved(PuzzleKind::Cube5)
        .apply(&"R U".parse().unwrap())
        .unwrap();
    let stub = Stub::new(Ok("U' R'"));
    let result = small(stub.clone()).solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap().to_string(), "U' R'");
    let seen = stub.seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let expected = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U".parse().unwrap())
        .unwrap();
    assert_eq!(seen[0], expected);
}

#[test]
fn a_failing_3x3x3_stage_is_reported_not_hidden() {
    let solver = small(Stub::new(Err(Failure::Exhausted)));
    let result = solver.solve(
        &Facelets::solved(PuzzleKind::Cube5),
        &SolveOptions::default(),
    );
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("3x3x3"));
}

#[test]
fn a_wrong_hand_off_answer_never_becomes_a_solution() {
    // The stub claims "R" solves a cube that "R" does not solve.
    let solver = small(Stub::new(Ok("R")));
    let result = solver.solve(
        &Facelets::solved(PuzzleKind::Cube5),
        &SolveOptions::default(),
    );
    assert!(!result.is_solved());
    assert_eq!(result.outcome, Err(Failure::Internal));
}

#[test]
fn a_solver_for_another_size_is_an_internal_error() {
    let stub = Arc::new(Stub {
        kind: PuzzleKind::Cube4,
        seen: Mutex::new(Vec::new()),
        answer: Ok(""),
    });
    let result = small(stub).solve(
        &Facelets::solved(PuzzleKind::Cube5),
        &SolveOptions::default(),
    );
    assert_eq!(result.outcome, Err(Failure::Internal));
    assert!(result.message.unwrap().contains("3x3x3"));
}

#[test]
fn scrambled_cubes_are_solved_and_replay() {
    let solver = real();
    for seed in 0..6 {
        let cube = scrambled(seed, 4);
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result
            .solution()
            .unwrap_or_else(|| panic!("seed {seed}: {result:?}"));
        assert!(cube.apply(solution).unwrap().is_solved(), "seed {seed}");
        assert!(solution.moves().iter().all(|m| m.depth <= 2));
    }
}

/// Whether the centre stickers of every face show one colour.
fn centres_uniform(cube: &Facelets) -> bool {
    let s = cube.stickers();
    (0..6).all(|f| (1..4).all(|r| (1..4).all(|c| s[f * 25 + r * 5 + c] == s[f * 25 + 12])))
}

#[test]
fn the_stages_cover_the_answer_and_the_reduction_ends_reduced() {
    let solver = real();
    let names = [
        "centres staged",
        "wings staged",
        "centres and wings paired",
        "3x3 stage",
    ];
    for seed in 0..12 {
        let cube = scrambled(seed + 40, 6);
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("a solution");
        let got: Vec<&str> = result.stages.iter().map(|s| s.name).collect();
        assert_eq!(got, names, "seed {seed}");
        let ends: Vec<usize> = result.stages.iter().map(|s| s.end).collect();
        assert!(ends.windows(2).all(|w| w[0] <= w[1]), "{ends:?}");
        assert_eq!(ends[3], solution.len(), "seed {seed}");
        let reduced = cube
            .apply(&MoveSequence::new(solution.moves()[..ends[2]].to_vec()))
            .unwrap();
        assert!(centres_uniform(&reduced), "seed {seed}: {ends:?}");
    }
}

#[test]
fn a_reduction_only_answer_has_the_three_reduction_stages() {
    let cube = scrambled(77, 6);
    let result = real().solve(&cube, &SolveOptions::default().with_reduce_only(true));
    let solution = result.solution().expect("a reduction");
    let got: Vec<(&str, usize)> = result.stages.iter().map(|s| (s.name, s.end)).collect();
    assert_eq!(got.len(), 3);
    assert_eq!(got[2], ("centres and wings paired", solution.len()));
}

#[test]
fn the_solution_does_not_depend_on_the_thread_count() {
    let cube = scrambled(9, 5);
    let one = real()
        .with_threads(1)
        .solve(&cube, &SolveOptions::default());
    // with_threads keeps the tables only per instance: share them
    let solver = real();
    let many = solver
        .clone()
        .with_threads(6)
        .solve(&cube, &SolveOptions::default());
    let four = solver.solve(&cube, &SolveOptions::default());
    assert!(one.is_solved());
    assert_eq!(one.outcome, many.outcome);
    assert_eq!(one.outcome, four.outcome);
}

#[test]
fn a_solved_cube_needs_no_moves() {
    let solved = Facelets::solved(PuzzleKind::Cube5);
    let result = real().solve(&solved, &SolveOptions::default());
    assert!(result.solution().unwrap().is_empty());
}

#[test]
fn wrong_size_and_impossible_cubes_fail_as_invalid_input() {
    let solver = real();
    let three = Facelets::solved(PuzzleKind::Cube3);
    assert_eq!(
        solver.solve(&three, &SolveOptions::default()).outcome,
        Err(Failure::InvalidInput)
    );
    // A corner twisted in place (URF becomes FUR).
    let mut stickers = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
    (stickers[24], stickers[25], stickers[54]) = (Face::F, Face::U, Face::R);
    let cube = Facelets::from_stickers(stickers).unwrap();
    let result = solver.solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
    // Stickers of different cubies swapped.
    let mut stickers = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
    (stickers[24], stickers[25], stickers[50]) = (Face::R, Face::F, Face::U);
    let cube = Facelets::from_stickers(stickers).unwrap();
    let result = solver.solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
    // Moved face centres.
    let cube = Facelets::solved(PuzzleKind::Cube5)
        .apply(&"3R".parse().unwrap())
        .unwrap();
    assert_eq!(
        solver.solve(&cube, &SolveOptions::default()).outcome,
        Err(Failure::CentersOffFace)
    );
}

#[test]
fn limits_are_honoured() {
    let solver = real();
    let cube = scrambled(3, 6);
    let tight = SolveOptions::default().with_max_moves(1);
    assert_eq!(solver.solve(&cube, &tight).outcome, Err(Failure::Exhausted));
    let none = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = solver.solve(&cube, &none);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
    // Tables exist now: the message is about the search, not a build.
    assert!(!result.message.unwrap().contains("building"));
    // A cold solver says it was the table build.
    let cold = small(Stub::new(Ok("")));
    let result = cold.solve(&cube, &none);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
    assert!(result
        .message
        .unwrap()
        .contains("building the 5x5x5 tables"));
}

#[test]
fn the_limit_counts_the_moves_of_the_whole_solution() {
    let solver = real();
    let cube = Facelets::solved(PuzzleKind::Cube5)
        .apply(&"R U R' U'".parse().unwrap())
        .unwrap();
    // Four moves are exactly enough, three are not.
    let four = solver.solve(&cube, &SolveOptions::default().with_max_moves(4));
    assert_eq!(four.solution().map(MoveSequence::len), Some(4));
    let three = solver.solve(&cube, &SolveOptions::default().with_max_moves(3));
    assert_eq!(three.outcome, Err(Failure::Exhausted));
}

#[test]
fn turns_of_the_same_layer_cancel_where_reduction_and_tail_meet() {
    let m = |s: &str| s.parse::<Move>().unwrap();
    let joined = join(vec![m("R"), m("U")], &[m("U"), m("R")]);
    assert_eq!(MoveSequence::new(joined).to_string(), "R U2 R");
    let joined = join(vec![m("R"), m("U")], &[m("U'"), m("U'"), m("R'")]);
    assert_eq!(MoveSequence::new(joined).to_string(), "R U' R'");
    let joined = join(vec![m("Rw")], &[m("R")]);
    assert_eq!(MoveSequence::new(joined).to_string(), "Rw R");
}

#[test]
fn a_hand_off_that_runs_out_of_time_without_a_budget_is_exhausted() {
    let stub = Stub::new(Err(Failure::BudgetExhausted));
    let cube = Facelets::solved(PuzzleKind::Cube5);
    let result = small(stub.clone()).solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("10 s"));
    let budgeted = SolveOptions {
        time_budget: Some(Duration::from_secs(60)),
        ..SolveOptions::default()
    };
    let result = small(stub).solve(&cube, &budgeted);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

fn plan(candidates: usize, probes: u64) -> Plan {
    Plan {
        candidates,
        probes,
        collect: Duration::from_secs(600),
    }
}

#[test]
fn one_candidate_with_the_default_probes_is_the_plain_answer() {
    let solver = real();
    for seed in 0..4 {
        let cube = scrambled(seed, 4);
        let options = SolveOptions::default();
        let plain = solver.solve(&cube, &options);
        let one = solver.run(&cube, &options, Some(plan(1, 500)));
        assert_eq!(plain.outcome, one.outcome, "seed {seed}");
    }
}

#[test]
fn the_best_of_several_candidates_is_valid_never_longer_and_thread_independent() {
    let solver = real();
    for seed in 0..4 {
        let cube = scrambled(seed, 4);
        let options = SolveOptions::default();
        let plain = solver.solve(&cube, &options);
        let a = solver
            .clone()
            .with_threads(1)
            .run(&cube, &options, Some(plan(5, 800)));
        let b = solver
            .clone()
            .with_threads(4)
            .run(&cube, &options, Some(plan(5, 800)));
        let solution = a.solution().expect("a solution");
        assert!(cube.apply(solution).unwrap().is_solved(), "seed {seed}");
        assert!(
            solution.len() <= plain.solution().unwrap().len(),
            "seed {seed}"
        );
        assert_eq!(a.outcome, b.outcome, "seed {seed}");
    }
}

#[test]
fn improving_with_a_budget_answers_and_a_spent_budget_fails_cleanly() {
    let solver = real();
    let cube = scrambled(2, 4);
    let options = SolveOptions::default().with_time_budget(Duration::from_secs(60));
    let result = solver.solve_improving(&cube, &options);
    assert!(cube
        .apply(result.solution().expect("a solution"))
        .unwrap()
        .is_solved());
    let spent = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = solver.solve_improving(&cube, &spent);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}
