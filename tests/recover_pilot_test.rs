//! A small permanent subset of the recover pilot (examples/recover_pilot):
//! fixed cubes from tests/fixtures/recover_pilot.txt, mistakes injected the way
//! the pilot does, `recover` run with the stop. The expected values (mistake
//! step and kind, stage, advice, route length) are those the pilot printed;
//! see the comment of each case.

use std::sync::Arc;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn};
use cubacle::recover::{
    diagnose_staged, Advice, Finding, Limits, Mistake, MistakeKind, Plan, Scope,
};

struct Cube {
    original: Facelets,
    solution: MoveSequence,
    phase_starts: Vec<usize>,
    size: usize,
}

fn cubes() -> Vec<Cube> {
    let text = include_str!("fixtures/recover_pilot.txt");
    let (mut size, mut scramble, mut solution) = (0, None, None);
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("CUBE ") {
            size = rest.split_whitespace().next().unwrap().parse().unwrap();
        } else if let Some(rest) = line.strip_prefix("SCRAMBLE ") {
            scramble = Some(rest.parse::<MoveSequence>().unwrap());
        } else if let Some(rest) = line.strip_prefix("SOLUTION ") {
            solution = Some(rest.parse::<MoveSequence>().unwrap());
        } else if let Some(rest) = line.strip_prefix("PHASES ") {
            let kind = match size {
                4 => PuzzleKind::Cube4,
                5 => PuzzleKind::Cube5,
                _ => PuzzleKind::Cube7,
            };
            out.push(Cube {
                original: Facelets::solved(kind)
                    .apply(&scramble.take().unwrap())
                    .unwrap(),
                solution: solution.take().unwrap(),
                phase_starts: rest
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect(),
                size,
            });
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Amount,
    Skip,
    Layer,
}

/// The pilot's wrong moves (examples/recover_pilot/cases.rs).
fn wrong(mv: Move, kind: Kind) -> Option<Move> {
    match kind {
        Kind::Skip => None,
        Kind::Amount => {
            let turn = if mv.turn == Turn::Half {
                Turn::Clockwise
            } else {
                Turn::Half
            };
            Some(Move::new(mv.face, mv.depth, mv.wide, turn))
        }
        Kind::Layer => Some(match (mv.wide, mv.depth) {
            (false, 1) => Move::new(mv.face, 2, true, mv.turn),
            (false, d) => Move::new(mv.face, d, true, mv.turn),
            (true, 2) => Move::new(mv.face, 1, false, mv.turn),
            (true, d) => Move::new(mv.face, d - 1, true, mv.turn),
        }),
    }
}

fn kind_matches(m: &Mistake, kind: Kind) -> bool {
    matches!(
        (&m.kind, kind),
        (MistakeKind::Skipped, Kind::Skip)
            | (MistakeKind::WrongAmount { .. }, Kind::Amount)
            | (MistakeKind::WrongLayers { .. }, Kind::Layer)
    )
}

fn solver_for(size: usize) -> Box<dyn Solver> {
    let three = Arc::new(Min2PhaseSolver::new());
    let four = FourSolver::new(three.clone());
    four.prepare();
    match size {
        4 => Box::new(four),
        5 => {
            let five = FiveSolver::new(three);
            five.prepare();
            Box::new(five)
        }
        _ => {
            let five = FiveSolver::new(three.clone());
            five.prepare();
            Box::new(DwaltonSolver::new(
                PuzzleKind::Cube7,
                three,
                Arc::new(four),
                Arc::new(five),
            ))
        }
    }
}

struct Case {
    cube: usize,
    stop: usize,
    mistakes: &'static [(usize, Kind)],
    stage: &'static str,
    /// Solve the cube afresh too (cheap except on a 7x7x7).
    fresh: bool,
    /// Expected advice.
    recover: bool,
    /// The pilot's route length (moves to do plus printed moves left): the
    /// test allows no more.
    route_max: usize,
}

fn check(cubes: &[Cube], c: &Case, solver: Option<&dyn Solver>) {
    let cube = &cubes[c.cube];
    let printed = cube.solution.moves();
    let mut done = Vec::new();
    for step in 1..=c.stop {
        let mv = printed[step - 1];
        match c.mistakes.iter().find(|m| m.0 == step) {
            None => done.push(mv),
            Some(&(_, kind)) => done.extend(wrong(mv, kind)),
        }
    }
    let now = cube.original.apply(&MoveSequence::new(done)).unwrap();
    // no deadline: the 8 s default reports "Unexplained" on a loaded machine
    let limits = Limits {
        max_time: None,
        ..Limits::default()
    };
    let scope = Scope {
        at_move: c.stop,
        sure_until: None,
        phase_starts: cube.phase_starts.clone(),
    };
    let staged = diagnose_staged(&cube.original, &now, &cube.solution, &limits, Some(&scope))
        .expect("diagnose");
    let Finding::Mistakes { mistakes, .. } = &staged.diagnosis.finding else {
        panic!(
            "case {c_cube}: {:?}",
            staged.diagnosis.finding,
            c_cube = c.cube
        );
    };
    let steps: Vec<usize> = mistakes.iter().map(|m| m.step).collect();
    let wanted: Vec<usize> = c.mistakes.iter().map(|m| m.0).collect();
    assert_eq!(steps, wanted, "mistake steps");
    for (m, &(_, kind)) in mistakes.iter().zip(c.mistakes) {
        assert!(kind_matches(m, kind), "{m:?} is not {kind:?}");
    }
    assert_eq!(staged.stage.describe(), c.stage, "stage");
    let n = cube.solution.len();
    let route = staged
        .diagnosis
        .routes
        .iter()
        .map(|r| r.total(n))
        .min()
        .expect("a route");
    assert!(route <= c.route_max, "route {route} > {}", c.route_max);
    let fresh = if c.fresh {
        let s = solver.expect("solver");
        Some(
            s.solve(&now, &SolveOptions::default())
                .solution()
                .cloned()
                .unwrap(),
        )
    } else {
        None
    };
    let plan = Plan::new(staged.diagnosis.clone(), cube.solution.clone(), fresh);
    match (&plan.advice, c.recover) {
        (Advice::Recover(r), true) => assert_eq!(r.total(n), route),
        (Advice::Fresh, false) => {}
        (other, _) => panic!("advice {other:?}, recover expected: {}", c.recover),
    }
}

const PHASE: &str = "the phase of the stop";
const PREVIOUS: &str = "the previous phase as well";
const PREFIX: &str = "all moves up to the stop";

// The pilot (cargo run --release --example recover_pilot) printed, per case:
// stage, route, fresh length and advice. Thresholds below are those values.

#[test]
fn seven_near_mistakes_are_recovered() {
    let cubes = cubes();
    assert_eq!(cubes[0].size, 7);
    // Pilot 1.4, skip 2 back of the stop: route 3, fresh 10, recover.
    let skip = Case {
        cube: 0,
        stop: 184,
        mistakes: &[(182, Kind::Skip)],
        stage: PHASE,
        fresh: false,
        recover: true,
        route_max: 3,
    };
    check(&cubes, &skip, None);
    // Pilot 3.1, a quarter for a half turn 5 back: route 12, fresh 19, recover.
    let amount = Case {
        cube: 2,
        stop: 187,
        mistakes: &[(182, Kind::Amount)],
        stage: PHASE,
        fresh: false,
        recover: true,
        route_max: 12,
    };
    check(&cubes, &amount, None);
    // Pilot 2.5, 20 back: needs the previous phase, route 54, fresh 69, recover.
    let far = Case {
        cube: 1,
        stop: 183,
        mistakes: &[(163, Kind::Amount)],
        stage: PREVIOUS,
        fresh: false,
        recover: true,
        route_max: 54,
    };
    check(&cubes, &far, None);
    // Pilot 1.5, two mistakes (15 and 5 back): route 44, fresh 47, recover.
    let two = Case {
        cube: 0,
        stop: 172,
        mistakes: &[(157, Kind::Amount), (167, Kind::Amount)],
        stage: PREVIOUS,
        fresh: false,
        recover: true,
        route_max: 44,
    };
    check(&cubes, &two, None);
}

#[test]
fn seven_far_mistake_is_solved_afresh() {
    let cubes = cubes();
    // Pilot 2.3, wrong layers 35 back: route 77 against fresh 45, so fresh.
    // Beyond about 25 back the route stops beating a fresh solve (pilot).
    let case = Case {
        cube: 1,
        stop: 190,
        mistakes: &[(155, Kind::Layer)],
        stage: PREVIOUS,
        fresh: true,
        recover: false,
        route_max: 77,
    };
    check(&cubes, &case, Some(solver_for(7).as_ref()));
}

#[test]
fn five_two_mistakes_are_recovered() {
    let cubes = cubes();
    assert_eq!(cubes[7].size, 5);
    // Pilot 8.3, two quarter-for-half turns 15 and 5 back: route 37 against
    // fresh 39, recover.
    let case = Case {
        cube: 7,
        stop: 64,
        mistakes: &[(49, Kind::Amount), (59, Kind::Amount)],
        stage: PREVIOUS,
        fresh: true,
        recover: true,
        route_max: 37,
    };
    check(&cubes, &case, Some(solver_for(5).as_ref()));
}

#[test]
fn four_mistakes_near_and_far() {
    let cubes = cubes();
    assert_eq!(cubes[9].size, 4);
    let solver = solver_for(4);
    // Pilot 10.5, 10 back: route 33 against fresh 35, recover.
    let near = Case {
        cube: 9,
        stop: 35,
        mistakes: &[(25, Kind::Amount)],
        stage: PREVIOUS,
        fresh: true,
        recover: true,
        route_max: 33,
    };
    check(&cubes, &near, Some(solver.as_ref()));
    // Pilot 9.1, 35 back (the start of the solution): route 72 against fresh
    // 34, so fresh.
    let far = Case {
        cube: 8,
        stop: 44,
        mistakes: &[(9, Kind::Amount)],
        stage: PREFIX,
        fresh: true,
        recover: false,
        route_max: 72,
    };
    check(&cubes, &far, Some(solver.as_ref()));
}
