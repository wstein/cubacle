//! Tests of the adapter on depth-capped tables (built in a few milliseconds)
//! with the real 3x3x3 solver and with stubs. No assertion depends on time.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::solver::FourSolver;
use crate::engines::three::Min2PhaseSolver;
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver, Turn,
};
use crate::reduction::improve::Plan;

/// Wing-table layers used here; the search only gets slower with fewer.
const LAYERS: u32 = 8;

fn solver(threads: usize) -> FourSolver {
    FourSolver::new(Arc::new(Min2PhaseSolver::shared()))
        .with_layers(LAYERS)
        .with_threads(threads)
}

/// A reproducible scramble of outer and second-layer turns.
fn scrambled(seed: u64, length: usize) -> Facelets {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let moves: Vec<Move> = (0..length)
        .map(|_| {
            let face = Face::ALL[next(6) as usize];
            let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][next(3) as usize];
            Move::new(face, 1 + next(2) as u8, false, turn)
        })
        .collect();
    Facelets::solved(PuzzleKind::Cube4)
        .apply(&MoveSequence::new(moves))
        .unwrap()
}

fn solves(cube: &Facelets, solution: &MoveSequence) -> bool {
    cube.apply(solution).is_ok_and(|end| end.is_solved())
}

/// Answers every request with a fixed result and records the options.
struct Stub {
    kind: PuzzleKind,
    reply: Result<&'static str, Failure>,
    seen: Mutex<Vec<SolveOptions>>,
    calls: AtomicU64,
}

impl Stub {
    fn new(reply: Result<&'static str, Failure>) -> Arc<Stub> {
        Arc::new(Stub::plain(reply))
    }

    fn plain(reply: Result<&'static str, Failure>) -> Stub {
        Stub {
            kind: PuzzleKind::Cube3,
            reply,
            seen: Mutex::new(Vec::new()),
            calls: AtomicU64::new(0),
        }
    }
}

impl Solver for Stub {
    fn kind(&self) -> PuzzleKind {
        self.kind
    }

    fn solve(&self, _: &Facelets, options: &SolveOptions) -> SolveResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.seen.lock().unwrap().push(*options);
        SolveResult {
            outcome: self.reply.map(|text| text.parse().expect("stub moves")),
            message: None,
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}

fn with_stub(stub: &Arc<Stub>) -> FourSolver {
    FourSolver::new(stub.clone())
        .with_layers(LAYERS)
        .with_threads(1)
}

#[test]
fn scrambled_cubes_are_solved_and_the_answer_replays() {
    let four = solver(2);
    for seed in 1..=6 {
        let cube = scrambled(seed, 40);
        let result = four.solve(&cube, &SolveOptions::default());
        let solution = result
            .solution()
            .unwrap_or_else(|| panic!("seed {seed}: {:?} {:?}", result.outcome, result.message));
        assert!(solves(&cube, solution), "seed {seed}");
    }
}

/// Whether the four centre stickers of every face show one colour.
fn centres_uniform(cube: &Facelets) -> bool {
    let s = cube.stickers();
    (0..6).all(|f| [6, 9, 10].iter().all(|&i| s[f * 16 + i] == s[f * 16 + 5]))
}

#[test]
fn the_stages_cover_the_answer_and_the_reduction_ends_with_solved_centres() {
    let four = solver(2);
    let names = [
        "centres staged",
        "centres sorted and wings split",
        "centres and wings paired",
        "3x3 stage",
    ];
    for seed in 1..=8 {
        let cube = scrambled(seed, 40);
        for slices in [false, true] {
            let options = SolveOptions::default().with_slices(slices);
            let result = four.solve(&cube, &options);
            let solution = result.solution().expect("a solution");
            let got: Vec<&str> = result.stages.iter().map(|s| s.name).collect();
            assert_eq!(got, names, "seed {seed}");
            let ends: Vec<usize> = result.stages.iter().map(|s| s.end).collect();
            assert!(ends.windows(2).all(|w| w[0] <= w[1]), "{ends:?}");
            assert_eq!(ends[3], solution.len(), "seed {seed}");
            let reduced = cube
                .apply(&MoveSequence::new(solution.moves()[..ends[2]].to_vec()))
                .unwrap();
            assert!(
                centres_uniform(&reduced),
                "seed {seed} slices {slices}: {ends:?}"
            );
            // the first stage is real work on a scrambled cube
            assert!(ends[0] > 0, "seed {seed}");
        }
    }
}

#[test]
fn a_reduction_only_answer_has_the_three_reduction_stages() {
    let cube = scrambled(3, 40);
    let result = solver(2).solve(&cube, &SolveOptions::default().with_reduce_only(true));
    let solution = result.solution().expect("a reduction");
    let got: Vec<(&str, usize)> = result.stages.iter().map(|s| (s.name, s.end)).collect();
    assert_eq!(got.len(), 3);
    assert_eq!(got[2], ("centres and wings paired", solution.len()));
}

#[test]
fn slices_are_only_written_when_asked_for_and_never_lengthen_the_answer() {
    let four = solver(2);
    for seed in 1..=6 {
        let cube = scrambled(seed, 40);
        let plain = four.solve(&cube, &SolveOptions::default());
        let sliced = four.solve(&cube, &SolveOptions::default().with_slices(true));
        let (plain, sliced) = (plain.solution().unwrap(), sliced.solution().unwrap());
        assert!(solves(&cube, plain) && solves(&cube, sliced), "seed {seed}");
        assert!(
            plain.moves().iter().all(|m| m.wide || m.depth == 1),
            "{plain}"
        );
        assert!(sliced.len() <= plain.len(), "seed {seed}");
    }
}

#[test]
fn the_solved_cube_needs_no_move() {
    let result = solver(1).solve(
        &Facelets::solved(PuzzleKind::Cube4),
        &SolveOptions::default(),
    );
    assert!(result.solution().unwrap().is_empty());
}

#[test]
fn a_rotated_solved_cube_is_brought_back_with_at_most_four_turns() {
    for rotation in ["Rw Lw'", "Uw Dw'", "Fw Bw'", "Rw Lw' Uw Dw'", "Fw2 Bw2"] {
        let cube = Facelets::solved(PuzzleKind::Cube4)
            .apply(&rotation.parse().unwrap())
            .unwrap();
        let result = solver(1).solve(&cube, &SolveOptions::default());
        let solution = result
            .solution()
            .unwrap_or_else(|| panic!("{rotation}: {result:?}"));
        assert!(solution.len() <= 4, "{rotation}: {solution}");
        assert!(solves(&cube, solution), "{rotation}");
    }
}

#[test]
fn the_answer_does_not_depend_on_the_number_of_threads() {
    let (one, three) = (solver(1), solver(3));
    for seed in 10..16 {
        let cube = scrambled(seed, 35);
        let options = SolveOptions::default();
        let a = one.solve(&cube, &options);
        let b = three.solve(&cube, &options);
        assert_eq!(a.outcome, b.outcome, "seed {seed}");
    }
}

#[test]
fn impossible_cubes_are_invalid_input_not_a_hang_or_a_wrong_answer() {
    let four = solver(1);
    let good = scrambled(3, 30);
    let invalid = |cube: &Facelets| {
        assert_eq!(
            four.solve(cube, &SolveOptions::default()).outcome,
            Err(Failure::InvalidInput)
        );
    };

    let mut text: Vec<char> = good.to_string().chars().collect();
    // Another size.
    invalid(&Facelets::solved(PuzzleKind::Cube3));
    // Two wing stickers exchanged. Most of these are impossible; an exchange
    // that happens to give a reachable cube must be solved correctly.
    let (mut tried, mut invalid_count) = (0, 0);
    for face in 1..6 {
        for place in [1, 2, 4, 7, 8, 11, 13, 14] {
            let mut stickers = good.stickers().to_vec();
            let other = face * 16 + place;
            if stickers[1] != stickers[other] {
                stickers.swap(1, other);
                let cube = Facelets::from_stickers(stickers).unwrap();
                match four.solve(&cube, &SolveOptions::default()).outcome {
                    Err(Failure::InvalidInput) => invalid_count += 1,
                    Ok(solution) => assert!(solves(&cube, &solution)),
                    other => panic!("{other:?}"),
                }
                tried += 1;
            }
        }
    }
    assert!(tried > 20 && invalid_count * 2 > tried);
    // Mirror image: up and down exchanged everywhere.
    for c in &mut text {
        *c = match *c {
            'U' => 'D',
            'D' => 'U',
            other => other,
        };
    }
    invalid(&Facelets::parse(&text.iter().collect::<String>()).unwrap());
    // A twisted corner: three stickers of one corner rotated.
    let mut stickers = good.stickers().to_vec();
    stickers.swap(0, 3 * 16 + 15);
    invalid(&Facelets::from_stickers(stickers).unwrap());
}

#[test]
fn a_failing_stub_is_reported_as_a_stage_failure() {
    let stub = Stub::new(Err(Failure::Exhausted));
    let four = with_stub(&stub);
    let result = four.solve(&scrambled(5, 30), &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().contains("3x3x3 stage"));
}

#[test]
fn an_answer_that_does_not_solve_is_never_returned() {
    let stub = Stub::new(Ok("R2"));
    let four = with_stub(&stub);
    for seed in [1, 2] {
        let result = four.solve(&scrambled(seed, 30), &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::Internal), "seed {seed}");
    }
}

#[test]
fn the_hand_off_asks_for_the_probe_minimum_and_a_bounded_length() {
    let stub = Stub::new(Err(Failure::Exhausted));
    let cube = scrambled(7, 30);
    with_stub(&stub).solve(&cube, &SolveOptions::default());
    with_stub(&stub)
        .with_hand_off_probes(7)
        .solve(&cube, &SolveOptions::default());
    with_stub(&stub).solve(&cube, &SolveOptions::default().with_min_probes(9));
    let seen = stub.seen.lock().unwrap();
    let probes: Vec<_> = seen.iter().map(|o| o.min_probes).collect();
    assert_eq!(probes, [Some(500), Some(7), Some(9)]);
    assert!(seen.iter().all(|o| o.max_moves == Some(21)));
    assert!(seen.iter().all(|o| o.time_budget.is_some()));
}

#[test]
fn a_move_limit_below_the_reduction_fails_before_the_hand_off() {
    let stub = Stub::new(Err(Failure::Exhausted));
    let result = with_stub(&stub).solve(
        &scrambled(8, 30),
        &SolveOptions::default().with_max_moves(5),
    );
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert_eq!(stub.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn no_time_means_a_budget_failure_not_a_hang() {
    let four = solver(1);
    four.prepare();
    let result = four.solve(
        &scrambled(9, 30),
        &SolveOptions::default().with_time_budget(Duration::ZERO),
    );
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn a_solver_that_is_not_a_3x3_solver_is_an_internal_error() {
    let stub = Arc::new(Stub {
        kind: PuzzleKind::Cube2,
        ..Stub::plain(Err(Failure::Exhausted))
    });
    let four = FourSolver::new(stub).with_layers(LAYERS);
    let result = four.solve(&scrambled(1, 20), &SolveOptions::default());
    assert_eq!(result.outcome, Err(Failure::Internal));
}

#[test]
fn a_move_limit_caps_the_hand_off_and_its_failure_is_exhausted_not_a_budget() {
    let cube = scrambled(11, 30);
    let stub = Stub::new(Err(Failure::BudgetExhausted));
    // A limit and a long time budget: the tail search is capped at one second
    // and its timeout is a limit failure.
    let options = SolveOptions::default()
        .with_max_moves(120)
        .with_time_budget(Duration::from_secs(30));
    let result = with_stub(&stub).solve(&cube, &options);
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    let seen = stub.seen.lock().unwrap();
    assert_eq!(seen[0].time_budget, Some(Duration::from_secs(1)));
    drop(seen);

    // No limit: the caller's budget governs and its end is a budget failure.
    let stub = Stub::new(Err(Failure::BudgetExhausted));
    let options = SolveOptions::default().with_time_budget(Duration::from_secs(30));
    let result = with_stub(&stub).solve(&cube, &options);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
    let seen = stub.seen.lock().unwrap();
    assert!(seen[0].time_budget.unwrap() > Duration::from_secs(1));
}

#[test]
fn a_spent_clock_stops_every_task_before_it_searches() {
    use super::candidates::Cand;
    use super::control::{Control, Stop};
    use super::cube::Cube;
    use super::tables::Tables;
    use super::{phase2, phase3};
    use std::time::Instant;

    let t = Tables::build(1, 4);
    let spent = Control::new(Some(Instant::now()));
    std::thread::sleep(Duration::from_millis(2));
    let cand = Cand::empty();
    let never = || false;
    let r2 = phase2::one(
        &t,
        &spent,
        &never,
        &cand,
        &Cube::solved(),
        5,
        10,
        usize::MAX,
    );
    assert_eq!(r2.err(), Some(Stop::Timeout));
    let r3 = phase3::one(&t, &spent, &never, &cand, 5, usize::MAX);
    assert_eq!(r3.err(), Some(Stop::Timeout));
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
    let four = solver(1);
    for seed in 1..=4 {
        let cube = scrambled(seed, 40);
        let options = SolveOptions::default();
        let plain = four.solve(&cube, &options);
        let one = four.run(&cube, &options, Some(plan(1, 500)));
        assert_eq!(plain.outcome, one.outcome, "seed {seed}");
    }
}

#[test]
fn the_best_of_several_candidates_is_valid_never_longer_and_thread_independent() {
    let one = solver(1);
    let many = solver(3);
    for seed in 1..=4 {
        let cube = scrambled(seed, 40);
        let options = SolveOptions::default();
        let plain = one.solve(&cube, &options);
        let a = one.run(&cube, &options, Some(plan(6, 800)));
        let b = many.run(&cube, &options, Some(plan(6, 800)));
        let solution = a.solution().expect("a solution");
        assert!(solves(&cube, solution), "seed {seed}");
        assert!(
            solution.len() <= plain.solution().unwrap().len(),
            "seed {seed}"
        );
        assert_eq!(a.outcome, b.outcome, "seed {seed}");
    }
}

#[test]
fn improving_with_a_budget_answers_and_a_spent_budget_fails_cleanly() {
    let four = solver(2);
    let cube = scrambled(5, 40);
    let options = SolveOptions::default().with_time_budget(Duration::from_secs(60));
    let result = four.solve_improving(&cube, &options);
    assert!(solves(&cube, result.solution().expect("a solution")));
    let spent = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result = four.solve_improving(&cube, &spent);
    assert_eq!(result.outcome, Err(Failure::BudgetExhausted));
}

#[test]
fn a_failing_first_tail_is_the_answer_of_the_improving_solve() {
    let stub = Stub::new(Err(Failure::Exhausted));
    let result = with_stub(&stub).run(
        &scrambled(3, 30),
        &SolveOptions::default(),
        Some(plan(4, 800)),
    );
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert_eq!(stub.calls.load(Ordering::SeqCst), 1);
    let seen = stub.seen.lock().unwrap();
    assert_eq!(seen[0].min_probes, Some(800));
}
