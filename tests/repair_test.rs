//! `repair` on synthetic mistakes: a printed solution is the inverse of a
//! scramble, a mistake is introduced at a chosen step, and the diagnosis and
//! the route back are checked by replaying them.

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn};
use cubacle::repair::{diagnose, Advice, Diagnosis, Finding, Limits, MistakeKind, Plan};

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
fn assert_repairs(diagnosis: &Diagnosis, current: &Facelets, solution: &[Move]) {
    assert!(!diagnosis.routes.is_empty(), "no route in {diagnosis:?}");
    for route in &diagnosis.routes {
        let after = current.apply(&route.moves).unwrap();
        let end = after
            .apply(&seq(&solution[route.continue_after..]))
            .unwrap();
        assert!(end.is_solved(), "route {} does not repair", route.moves);
    }
}

fn limits() -> Limits {
    Limits::default()
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
fn every_kind_of_mistake_is_named_and_repaired() {
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
            assert_repairs(&d, &current, &solution);
        }
    }
}

#[test]
fn stray_moves_after_the_last_step_are_found() {
    let (original, solution) = setup(4, 20, 99);
    let mut done = solution[..8].to_vec();
    done.push(Move::new(Face::R, 1, false, Turn::Clockwise));
    done.push(Move::new(Face::U, 2, true, Turn::Half));
    let current = run(&original, &done);
    let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
    assert!(
        !matches!(d.finding, Finding::Unexplained),
        "{:?}",
        d.finding
    );
    assert_repairs(&d, &current, &solution);
    let shortest = d.routes.iter().map(|r| r.moves.len()).min().unwrap();
    assert!(shortest <= 2);
}

#[test]
fn two_mistakes_are_found_when_asked_for() {
    let (original, solution) = setup(3, 22, 5);
    let mut done = go_wrong(&solution, 4, 9, "amount");
    // Second mistake: skip the move at 12, then stop at 15.
    done.extend_from_slice(&solution[9..12]);
    done.extend_from_slice(&solution[13..15]);
    let current = run(&original, &done);
    let d = diagnose(&original, &current, &seq(&solution), &limits()).unwrap();
    match &d.finding {
        Finding::Mistakes { mistakes, .. } => assert_eq!(mistakes.len(), 2, "{mistakes:?}"),
        other => panic!("{other:?}"),
    }
    assert_repairs(&d, &current, &solution);
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
    assert!(matches!(plan.advice, Advice::Repair(_)));
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
        cubacle::repair::locate(&original, &at, &seq(&solution)),
        Ok(Some(7))
    );
    let off = run(&original, &go_wrong(&solution, 3, 8, "skip"));
    assert_eq!(
        cubacle::repair::locate(&original, &off, &seq(&solution)),
        Ok(None)
    );
}
