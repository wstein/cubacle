//! `repair` on four real 7x7x7 attempts (tests/fixtures/repair_7x7.txt): a
//! person followed the solver's solution and stopped at another cube.

use std::collections::HashMap;
use std::time::Instant;

use cubacle::notation::orbit64;
use cubacle::puzzle::{Facelets, MoveSequence};
use cubacle::repair::{diagnose, Advice, Finding, Limits, MistakeKind, Plan};

struct Attempt {
    original: Facelets,
    solution: MoveSequence,
    current: Facelets,
    fresh: MoveSequence,
}

fn attempts() -> HashMap<String, Attempt> {
    let text = include_str!("fixtures/repair_7x7.txt");
    let mut out = HashMap::new();
    let mut name = String::new();
    let mut fields: HashMap<&str, &str> = HashMap::new();
    for line in text.lines().chain(std::iter::once("")) {
        if line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(' ') {
            if key == "PAIR" {
                name = value.to_owned();
            } else {
                fields.insert(key, value);
            }
        } else if !fields.is_empty() {
            let cube = |key: &str| orbit64::decode(fields[key]).expect("token");
            let moves = |key: &str| fields[key].parse::<MoveSequence>().expect("moves");
            out.insert(
                std::mem::take(&mut name),
                Attempt {
                    original: cube("ORIGINAL"),
                    solution: moves("SOLUTION"),
                    current: cube("CURRENT"),
                    fresh: moves("FRESH"),
                },
            );
            fields.clear();
        }
    }
    out
}

/// Diagnoses a pair, checks every route by replaying it, and returns the plan.
fn plan_of(name: &str) -> Plan {
    let a = &attempts()[name];
    let started = Instant::now();
    let d = diagnose(&a.original, &a.current, &a.solution, &Limits::default()).unwrap();
    assert!(started.elapsed().as_secs() < 10, "{name} took too long");
    for route in &d.routes {
        let after = a.current.apply(&route.moves).unwrap();
        let rest = MoveSequence::new(a.solution.moves()[route.continue_after..].to_vec());
        assert!(after.apply(&rest).unwrap().is_solved(), "{name}: {route:?}");
    }
    let plan = Plan::new(d, a.solution.clone(), Some(a.fresh.clone()));
    // A repair is never longer than solving fresh.
    if let Advice::Repair(route) = &plan.advice {
        assert!(route.total(a.solution.len()) <= a.fresh.len());
    }
    plan
}

fn mistake_steps(plan: &Plan) -> Vec<usize> {
    match &plan.diagnosis.finding {
        Finding::Mistakes { mistakes, .. } => mistakes.iter().map(|m| m.step).collect(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_skipped_move_costs_more_to_repair_than_to_solve_fresh() {
    let plan = plan_of("4-5");
    let Finding::Mistakes { mistakes, done, .. } = &plan.diagnosis.finding else {
        panic!("{:?}", plan.diagnosis.finding);
    };
    assert_eq!((mistakes[0].step, *done), (15, 28));
    assert_eq!(mistakes[0].kind, MistakeKind::Skipped);
    assert_eq!(mistakes[0].printed.unwrap().to_string(), "Bw2");
    assert_eq!(plan.advice, Advice::Fresh);
}

#[test]
fn two_mistakes_on_the_way_are_both_named() {
    let plan = plan_of("1-2");
    assert_eq!(mistake_steps(&plan), [7, 42]);
    assert_eq!(plan.advice, Advice::Fresh);
}

#[test]
fn a_wrong_amount_and_two_stray_moves_are_repaired_in_three_moves() {
    let plan = plan_of("8-9");
    let Finding::Mistakes {
        mistakes, strays, ..
    } = &plan.diagnosis.finding
    else {
        panic!("{:?}", plan.diagnosis.finding);
    };
    assert_eq!(mistakes[0].step, 25);
    assert!(matches!(mistakes[0].kind, MistakeKind::WrongAmount { .. }));
    assert_eq!(strays.len(), 2);
    let Advice::Repair(route) = &plan.advice else {
        panic!("{:?}", plan.advice);
    };
    assert_eq!(route.moves.len(), 3);
    assert!(plan.to_string().contains("Recommended: repair"));
}

#[test]
fn a_skipped_move_with_a_stray_move_is_repaired_when_that_is_shorter() {
    let plan = plan_of("12-13");
    assert_eq!(mistake_steps(&plan), [54]);
    assert!(matches!(plan.advice, Advice::Repair(_)), "{plan}");
}
