//! `recover` on synthetic mistakes: a printed solution is the inverse of a
//! scramble, a mistake is introduced at a chosen step, and the diagnosis and
//! the route back are checked by replaying them.

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn};
use cubacle::recover::{
    diagnose, diagnose_staged, Advice, Diagnosis, Finding, Limits, MistakeKind, Plan, Scope,
    Widening,
};

struct Rng(u64);

impl Rng {
    fn next(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as usize % bound
    }
}

fn inverse(mv: &Move) -> Move {
    let turn = match mv.turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    };
    Move::new(mv.face, mv.depth, mv.wide, turn)
}

fn random_move(rng: &mut Rng, kind: PuzzleKind) -> Move {
    let face = Face::ALL[rng.next(6)];
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][rng.next(3)];
    if kind.size() <= 3 || rng.next(3) == 0 {
        return Move::new(face, 1, false, turn);
    }
    let depth = 2 + rng.next(kind.size() - 2) as u8;
    Move::new(face, depth, rng.next(2) == 0, turn)
}

/// A cube and a solution for it, with neighbours that do not commute.
fn setup(size: usize, len: usize, seed: u64) -> (Facelets, Vec<Move>) {
    let kind = PuzzleKind::of_size(size).unwrap();
    let mut rng = Rng(seed);
    let mut scramble: Vec<Move> = Vec::new();
    while scramble.len() < len {
        let mv = random_move(&mut rng, kind);
        if scramble.last().is_none_or(|last| last.face != mv.face) {
            scramble.push(mv);
        }
    }
    let original = Facelets::solved(kind)
        .apply(&MoveSequence::new(scramble.clone()))
        .unwrap();
    let solution = scramble.iter().rev().map(inverse).collect();
    (original, solution)
}

fn seq(moves: &[Move]) -> MoveSequence {
    MoveSequence::new(moves.to_vec())
}

fn run(original: &Facelets, moves: &[Move]) -> Facelets {
    original.apply(&seq(moves)).unwrap()
}

/// Replays the best route from the current cube and then the rest of the
/// printed solution: the cube must end up solved.
fn assert_recovers(diagnosis: &Diagnosis, current: &Facelets, solution: &[Move]) {
    assert!(!diagnosis.routes.is_empty(), "no route in {diagnosis:?}");
    for route in &diagnosis.routes {
        let after = current.apply(&route.moves).unwrap();
        let end = after
            .apply(&seq(&solution[route.continue_after..]))
            .unwrap();
        assert!(end.is_solved(), "route {} does not recover", route.moves);
    }
}

fn limits() -> Limits {
    Limits {
        max_time: None,
        ..Limits::default()
    }
}

/// Skips, changes, swaps or repeats the move at `at`, then keeps following
/// the printed solution until `stop` moves are used up.
fn go_wrong(solution: &[Move], at: usize, stop: usize, how: &str) -> Vec<Move> {
    let mv = solution[at];
    let mut done: Vec<Move> = solution[..at].to_vec();
    let mut from = at + 1;
    match how {
        "skip" => {}
        "amount" => {
            let turn = if mv.turn == Turn::Half {
                Turn::Clockwise
            } else {
                Turn::Half
            };
            done.push(Move::new(mv.face, mv.depth, mv.wide, turn));
        }
        "direction" => done.push(inverse(&mv)),
        "swap" => {
            done.push(solution[at + 1]);
            done.push(mv);
            from = at + 2;
        }
        "repeat" => {
            done.push(mv);
            done.push(mv);
        }
        _ => unreachable!(),
    }
    done.extend_from_slice(&solution[from..stop]);
    done
}

#[test]
fn a_cube_on_the_path_is_recognised() {
    for size in [2, 3, 5] {
        let (original, solution) = setup(size, 12, 7 + size as u64);
        for j in [0, 1, 6, 12] {
            let current = run(&original, &solution[..j]);
            let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
            assert_eq!(d.finding, Finding::OnPath { done: j }, "size {size}");
        }
    }
}

#[test]
fn every_kind_of_mistake_is_named_and_recovered() {
    for size in [3, 4, 5] {
        let (original, solution) = setup(size, 24, 31 * size as u64);
        let kinds = ["skip", "amount", "direction", "swap", "repeat"];
        for how in kinds {
            let at = (9..).find(|&i| solution[i].turn != Turn::Half).unwrap();
            let stop = at + 4;
            let done = go_wrong(&solution, at, stop, how);
            let current = run(&original, &done);
            let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
            let Finding::Mistakes { mistakes, .. } = &d.finding else {
                panic!("{how} on {size}x{size}: {:?}", d.finding);
            };
            assert_eq!(mistakes.len(), 1, "{how}: {mistakes:?}");
            assert!(
                mistakes[0].step <= at + 1,
                "{how} on {size}x{size}: step {}",
                mistakes[0].step
            );
            if mistakes[0].step == at + 1 || how == "repeat" {
                let wanted = match how {
                    "amount" => matches!(mistakes[0].kind, MistakeKind::WrongAmount { .. }),
                    "direction" => matches!(mistakes[0].kind, MistakeKind::WrongDirection { .. }),
                    "swap" => matches!(mistakes[0].kind, MistakeKind::Swapped { .. }),
                    "skip" => mistakes[0].kind == MistakeKind::Skipped,
                    // A quarter turn done twice is also a half turn.
                    _ => matches!(
                        mistakes[0].kind,
                        MistakeKind::Repeated | MistakeKind::WrongAmount { .. }
                    ),
                };
                assert!(wanted, "{how} on {size}x{size}: {:?}", mistakes[0]);
            }
            assert_recovers(&d, &current, &solution);
        }
    }
}

#[test]
fn two_mistakes_are_found_when_asked_for() {
    let (original, solution) = setup(3, 22, 5);
    let mut done = go_wrong(&solution, 4, 9, "amount");
    // Second mistake: skip the move at 12, then stop at 15.
    done.extend_from_slice(&solution[9..12]);
    done.extend_from_slice(&solution[13..15]);
    let current = run(&original, &done);
    let two = limits();
    let d = diagnose(&original, &current, &seq(&solution), &two).unwrap();
    match &d.finding {
        Finding::Mistakes { mistakes, .. } => assert_eq!(mistakes.len(), 2, "{mistakes:?}"),
        other => panic!("{other:?}"),
    }
    assert_recovers(&d, &current, &solution);
    let one = Limits {
        two_mistakes: false,
        resume_depth: 1,
        ..limits()
    };
    let d = diagnose(&original, &current, &seq(&solution), &one).unwrap();
    assert!(matches!(d.finding, Finding::Unexplained), "{:?}", d.finding);
}

#[test]
fn a_turned_cube_is_recognised_after_the_normal_search_fails() {
    let (original, solution) = setup(3, 16, 11);
    let current = run(&original, &solution[..6]);
    // Turn the whole cube like R: Rw L'.
    let x = [
        Move::new(Face::R, 2, true, Turn::Clockwise),
        Move::new(Face::L, 1, false, Turn::CounterClockwise),
    ];
    let turned = current.apply(&seq(&x)).unwrap();
    let d = diagnose(&original, &turned, &seq(&solution), &limits()).unwrap();
    assert_eq!(d.finding, Finding::OnPath { done: 6 });
    assert_eq!(d.rotation.as_deref(), Some("x'"));
    let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
    assert_eq!(d.rotation, None);
}

#[test]
fn the_plan_never_recommends_more_moves_than_a_fresh_solution() {
    let (original, solution) = setup(3, 20, 3);
    let done = go_wrong(&solution, 2, 15, "skip");
    let current = run(&original, &done);
    let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
    let best = d.routes.iter().map(|r| r.total(20)).min().unwrap();
    let long = seq(&vec![
        Move::new(Face::U, 1, false, Turn::Clockwise);
        best + 5
    ]);
    let plan = Plan::new(d.clone(), seq(&solution), Some(long));
    assert!(matches!(plan.advice, Advice::Recover(_)));
    let short = seq(&vec![
        Move::new(Face::U, 1, false, Turn::Clockwise);
        best - 1
    ]);
    let plan = Plan::new(d.clone(), seq(&solution), Some(short.clone()));
    assert_eq!(plan.advice, Advice::Fresh);
    assert_eq!(plan.moves(), Some(&short));
    let text = plan.to_string();
    assert!(text.contains("Recommended: solve fresh"), "{text}");
}

#[test]
fn many_mistakes_fall_back_to_the_fresh_solution() {
    let solver = Min2PhaseSolver::shared();
    let (original, solution) = setup(3, 20, 17);
    let mut rng = Rng(5);
    let mut done = solution[..10].to_vec();
    for _ in 0..9 {
        done.push(random_move(&mut rng, PuzzleKind::Cube3));
    }
    let current = run(&original, &done);
    let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
    assert!(matches!(d.finding, Finding::Unexplained), "{:?}", d.finding);
    let fresh = solver
        .solve(&current, &SolveOptions::default().with_max_moves(21))
        .outcome
        .unwrap();
    let plan = Plan::new(d, seq(&solution), Some(fresh.clone()));
    assert_eq!(plan.advice, Advice::Fresh);
    assert!(current.apply(plan.moves().unwrap()).unwrap().is_solved());
}

#[test]
fn bad_input_is_an_error() {
    let (original, solution) = setup(3, 8, 1);
    let other = Facelets::solved(PuzzleKind::Cube4);
    assert!(diagnose(&original, &other, &seq(&solution), &limits()).is_err());
    let wrong = seq(&solution[..4]);
    assert!(diagnose(&original, &original, &wrong, &limits()).is_err());
}

#[test]
fn locate_finds_the_exact_position_only() {
    let (original, solution) = setup(4, 10, 21);
    let at = run(&original, &solution[..7]);
    assert_eq!(
        cubacle::recover::locate(&original, &at, &seq(&solution)),
        Ok(Some(7))
    );
    let off = run(&original, &go_wrong(&solution, 3, 8, "skip"));
    assert_eq!(
        cubacle::recover::locate(&original, &off, &seq(&solution)),
        Ok(None)
    );
}

fn staged(
    original: &Facelets,
    current: &Facelets,
    solution: &[Move],
    at_move: usize,
) -> cubacle::recover::Staged {
    let scope = Scope::at(at_move);
    diagnose_staged(original, current, &seq(solution), &limits(), Some(&scope)).unwrap()
}

#[test]
fn the_move_stopped_at_confines_the_search_to_the_moves_before_it() {
    let (original, solution) = setup(5, 30, 41);
    // Skip move 11 (index 10) and stop after printed move 16.
    let done = go_wrong(&solution, 10, 16, "skip");
    let current = run(&original, &done);
    let s = staged(&original, &current, &solution, 16);
    assert_eq!(s.stage, Widening::Prefix);
    assert_eq!(s.tried, [Widening::Prefix]);
    let Finding::Mistakes { mistakes, .. } = &s.diagnosis.finding else {
        panic!("{:?}", s.diagnosis.finding);
    };
    assert_eq!(mistakes[0].step, 11);
    assert_eq!(mistakes[0].kind, MistakeKind::Skipped);
    assert_recovers(&s.diagnosis, &current, &solution);
}

#[test]
fn a_stop_before_the_mistake_falls_back_to_the_whole_search() {
    let (original, solution) = setup(5, 30, 41);
    let done = go_wrong(&solution, 10, 16, "skip");
    let current = run(&original, &done);
    // Stopping "at 8" cannot explain a mistake at move 11: widened to the end.
    let s = staged(&original, &current, &solution, 8);
    assert_eq!(s.stage, Widening::Full);
    assert_eq!(s.tried, [Widening::Prefix, Widening::Full]);
    assert!(matches!(s.diagnosis.finding, Finding::Mistakes { .. }));
}

#[test]
fn a_stop_outside_the_solution_is_an_error() {
    let (original, solution) = setup(3, 12, 3);
    let current = run(&original, &solution[..4]);
    for at_move in [0, 13] {
        let scope = Scope::at(at_move);
        let err = diagnose_staged(
            &original,
            &current,
            &seq(&solution),
            &limits(),
            Some(&scope),
        );
        assert!(err.is_err(), "{at_move}");
    }
}

#[test]
fn without_a_scope_the_plain_search_runs() {
    let (original, solution) = setup(3, 12, 3);
    let current = run(&original, &solution[..4]);
    let s = diagnose_staged(&original, &current, &seq(&solution), &limits(), None).unwrap();
    assert_eq!(s.tried, [Widening::Full]);
    assert_eq!(s.diagnosis.finding, Finding::OnPath { done: 4 });
}

fn phased(at_move: usize, sure_until: Option<usize>) -> Scope {
    Scope {
        at_move,
        sure_until,
        phase_starts: vec![0, 8, 14, 24],
    }
}

fn skipped_at_11() -> (Facelets, Vec<Move>, Facelets) {
    let (original, solution) = setup(5, 30, 41);
    let done = go_wrong(&solution, 10, 16, "skip");
    let current = run(&original, &done);
    (original, solution, current)
}

#[test]
fn the_search_widens_phase_by_phase_until_the_mistake_is_inside() {
    let (original, solution, current) = skipped_at_11();
    let s = diagnose_staged(
        &original,
        &current,
        &seq(&solution),
        &limits(),
        Some(&phased(16, None)),
    )
    .unwrap();
    // Move 11 is in phase 2, the stop in phase 3.
    assert_eq!(s.tried, [Widening::Phase, Widening::PreviousPhase]);
    assert_eq!(s.stage, Widening::PreviousPhase);
    assert!(matches!(s.diagnosis.finding, Finding::Mistakes { .. }));
}

#[test]
fn a_right_sure_move_answers_in_the_first_stage() {
    let (original, solution, current) = skipped_at_11();
    let s = diagnose_staged(
        &original,
        &current,
        &seq(&solution),
        &limits(),
        Some(&phased(16, Some(9))),
    )
    .unwrap();
    assert_eq!(s.tried, [Widening::Sure]);
}

#[test]
fn a_wrong_sure_move_is_a_hint_only() {
    let (original, solution, current) = skipped_at_11();
    // Sure until 13, but the mistake is at move 11: widened past the hint.
    let s = diagnose_staged(
        &original,
        &current,
        &seq(&solution),
        &limits(),
        Some(&phased(16, Some(13))),
    )
    .unwrap();
    assert_eq!(s.tried, [Widening::Sure, Widening::PreviousPhase]);
    let Finding::Mistakes { mistakes, .. } = &s.diagnosis.finding else {
        panic!("{:?}", s.diagnosis.finding);
    };
    assert_eq!(mistakes[0].step, 11);
}

#[test]
fn a_sure_move_after_the_stop_is_an_error() {
    let (original, solution, current) = skipped_at_11();
    let err = diagnose_staged(
        &original,
        &current,
        &seq(&solution),
        &limits(),
        Some(&phased(16, Some(17))),
    );
    assert!(err.is_err());
}

#[test]
fn the_plan_names_the_move_and_the_phase_of_the_mistake() {
    use cubacle::recover::PhaseInfo;
    let (original, solution, current) = skipped_at_11();
    let s = staged(&original, &current, &solution, 16);
    let phases = ["a", "b", "c", "d"]
        .iter()
        .zip([0, 8, 14, 24])
        .enumerate()
        .map(|(i, (name, start))| PhaseInfo {
            number: i + 1,
            name: (*name).to_owned(),
            start,
        })
        .collect();
    let plan = Plan::new(s.diagnosis, seq(&solution), None).with_phases(phases);
    assert!(
        plan.to_string().contains("mistake at move 11, phase 2 (b)"),
        "{plan}"
    );
}
