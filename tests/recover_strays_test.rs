//! The last widening stage of `recover`: one mistake followed by moves a
//! person made by hand. It runs only when nothing else explains the cube.

use cubacle::notation::orbit64;
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};
use cubacle::recover::{
    diagnose_staged, Finding, Limits, MistakeKind, Plan, Scope, Staged, Widening,
};

/// A scramble of non-commuting neighbors, and the solution that undoes it.
fn setup(size: usize, len: usize, seed: u64) -> (Facelets, Vec<Move>) {
    let kind = PuzzleKind::of_size(size).unwrap();
    let mut state = seed;
    let mut next = |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as usize % bound
    };
    let mut scramble: Vec<Move> = Vec::new();
    while scramble.len() < len {
        let face = Face::ALL[next(6)];
        let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][next(3)];
        let mv = if kind.size() <= 3 || next(3) == 0 {
            Move::new(face, 1, false, turn)
        } else {
            let depth = 2 + next(kind.size() - 2) as u8;
            Move::new(face, depth, next(2) == 0, turn)
        };
        if scramble.last().is_none_or(|last| last.face != mv.face) {
            scramble.push(mv);
        }
    }
    let original = Facelets::solved(kind)
        .apply(&MoveSequence::new(scramble.clone()))
        .unwrap();
    let solution = scramble
        .iter()
        .rev()
        .map(|m| {
            let turn = match m.turn {
                Turn::Clockwise => Turn::CounterClockwise,
                Turn::CounterClockwise => Turn::Clockwise,
                Turn::Half => Turn::Half,
            };
            Move::new(m.face, m.depth, m.wide, turn)
        })
        .collect();
    (original, solution)
}

fn seq(moves: &[Move]) -> MoveSequence {
    MoveSequence::new(moves.to_vec())
}

fn staged(original: &Facelets, current: &Facelets, sol: &[Move], limits: &Limits) -> Staged {
    diagnose_staged(original, current, &seq(sol), limits, None).unwrap()
}

/// Every route of the diagnosis, then the rest of the solution, solves the cube.
fn assert_recovers(s: &Staged, current: &Facelets, solution: &[Move]) {
    assert!(!s.diagnosis.routes.is_empty());
    for route in &s.diagnosis.routes {
        let after = current.apply(&route.moves).unwrap();
        let end = after
            .apply(&seq(&solution[route.continue_after..]))
            .unwrap();
        assert!(end.is_solved(), "route {} does not recover", route.moves);
    }
}

fn extra() -> [Move; 2] {
    [
        Move::new(Face::R, 1, false, Turn::Clockwise),
        Move::new(Face::D, 1, false, Turn::Half),
    ]
}

#[test]
fn stray_moves_after_the_last_step_are_found_last() {
    let (original, solution) = setup(4, 20, 99);
    let mut done = solution[..8].to_vec();
    done.push(Move::new(Face::R, 1, false, Turn::Clockwise));
    done.push(Move::new(Face::U, 2, true, Turn::Half));
    let current = original.apply(&seq(&done)).unwrap();
    let s = staged(&original, &current, &solution, &no_deadline());
    assert_eq!(s.stage, Widening::Strays, "{:?}", s.diagnosis.finding);
    assert_eq!(s.tried.last(), Some(&Widening::Strays));
    assert_recovers(&s, &current, &solution);
    let shortest = s.diagnosis.routes.iter().map(|r| r.moves.len()).min();
    assert!(shortest.unwrap() <= 2);
    let none = Limits {
        strays: 0,
        ..no_deadline()
    };
    let s = staged(&original, &current, &solution, &none);
    assert_ne!(s.stage, Widening::Strays);
    assert!(!s.tried.contains(&Widening::Strays));
}

#[test]
fn a_mistake_followed_by_stray_moves_is_found() {
    for size in [3, 5] {
        let (original, solution) = setup(size, 30, 77 + size as u64);
        let mut done = solution[..10].to_vec();
        done.extend_from_slice(&solution[11..16]);
        done.extend(extra());
        let current = original.apply(&seq(&done)).unwrap();
        let s = staged(&original, &current, &solution, &no_deadline());
        let Finding::Mistakes {
            mistakes, strays, ..
        } = &s.diagnosis.finding
        else {
            panic!("{size}x{size}: {:?}", s.diagnosis.finding);
        };
        assert_eq!(s.stage, Widening::Strays);
        assert_eq!(mistakes.len(), 1);
        assert!(mistakes[0].step <= 11, "{:?}", mistakes[0]);
        assert!(!strays.is_empty() && strays.len() <= 2);
        assert_recovers(&s, &current, &solution);
    }
}

#[test]
fn a_cube_explained_without_stray_moves_never_reaches_the_stage() {
    let (original, solution) = setup(5, 30, 41);
    let mut done = solution[..10].to_vec();
    done.extend_from_slice(&solution[11..16]);
    let current = original.apply(&seq(&done)).unwrap();
    let s = staged(&original, &current, &solution, &no_deadline());
    assert!(!s.tried.contains(&Widening::Strays));
    let Finding::Mistakes { strays, .. } = &s.diagnosis.finding else {
        panic!("{:?}", s.diagnosis.finding);
    };
    assert!(strays.is_empty());
}

#[test]
fn with_a_stop_the_stray_moves_come_after_it() {
    let (original, solution) = setup(5, 30, 41);
    let mut done = solution[..10].to_vec();
    done.extend_from_slice(&solution[11..16]);
    done.extend(extra());
    let current = original.apply(&seq(&done)).unwrap();
    let scope = Scope::at(16);
    let limits = no_deadline();
    let s = diagnose_staged(&original, &current, &seq(&solution), &limits, Some(&scope)).unwrap();
    assert_eq!(s.stage, Widening::Strays);
    assert_eq!(s.tried.last(), Some(&Widening::Strays));
    assert!(matches!(
        s.diagnosis.finding,
        Finding::Mistakes { ref strays, .. } if !strays.is_empty()
    ));
}

/// A real 7x7x7 attempt of tests/fixtures/repair_7x7.txt.
fn real(name: &str) -> (Facelets, Facelets, MoveSequence) {
    let text = include_str!("fixtures/repair_7x7.txt");
    let mut found = false;
    let mut fields = std::collections::HashMap::new();
    for line in text.lines() {
        if let Some(n) = line.strip_prefix("PAIR ") {
            found = n == name;
        } else if found {
            if let Some((k, v)) = line.split_once(' ') {
                fields.insert(k, v);
            }
        }
    }
    let cube = |k: &str| orbit64::decode(fields[k]).expect("token");
    (
        cube("ORIGINAL"),
        cube("CURRENT"),
        fields["SOLUTION"].parse().expect("moves"),
    )
}

#[test]
fn a_wrong_amount_and_two_stray_moves_are_found_last_on_a_real_cube() {
    let (original, current, solution) = real("8-9");
    let s = diagnose_staged(&original, &current, &solution, &no_deadline(), None).unwrap();
    assert_eq!(s.stage, Widening::Strays);
    let Finding::Mistakes {
        mistakes, strays, ..
    } = &s.diagnosis.finding
    else {
        panic!("{:?}", s.diagnosis.finding);
    };
    assert_eq!(mistakes[0].step, 25);
    assert!(matches!(mistakes[0].kind, MistakeKind::WrongAmount { .. }));
    assert_eq!(strays.len(), 2);
}

#[test]
fn a_skipped_move_with_a_stray_move_is_found_last_on_a_real_cube() {
    let (original, current, solution) = real("12-13");
    let s = diagnose_staged(&original, &current, &solution, &no_deadline(), None).unwrap();
    assert_eq!(s.stage, Widening::Strays);
    let Finding::Mistakes { mistakes, .. } = &s.diagnosis.finding else {
        panic!("{:?}", s.diagnosis.finding);
    };
    assert_eq!(mistakes[0].step, 54);
}

#[test]
fn an_explanation_that_needs_stray_moves_says_it_is_less_certain() {
    let (original, current, solution) = real("8-9");
    let text = |limits: &Limits| {
        let s = diagnose_staged(&original, &current, &solution, limits, None).unwrap();
        Plan::new(s.diagnosis, solution.clone(), None).to_string()
    };
    let with = text(&no_deadline());
    assert!(
        with.contains("Less certain: this explanation assumes 2 extra moves after the mistake"),
        "{with}"
    );
    let without = text(&Limits {
        strays: 0,
        ..no_deadline()
    });
    assert!(!without.contains("Less certain"), "{without}");
}

/// Limits without the wall-clock deadline, so a loaded machine searches longer instead of
/// answering wrongly.
fn no_deadline() -> Limits {
    Limits {
        max_time: None,
        ..Limits::default()
    }
}
