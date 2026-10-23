//! `recover` with `--at-move`/`--sure-until` on all fourteen real 7x7x7
//! attempts (tests/fixtures/recover_7x7_runs.txt): thirteen consecutive pairs,
//! each the cube a person held after following the solver's solution of the
//! previous one. The stop is taken as the number of printed moves the plain
//! search says were used up.

use cubacle::notation::orbit64;
use cubacle::phases::phases;
use cubacle::puzzle::{Facelets, MoveSequence};
use cubacle::recover::{diagnose, diagnose_staged, Finding, Limits, Scope, Widening};

struct Run {
    cube: Facelets,
    solution: MoveSequence,
}

fn runs() -> Vec<Run> {
    let text = include_str!("fixtures/recover_7x7_runs.txt");
    let mut out = Vec::new();
    let mut cube = None;
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with("MOVES") || line.trim().is_empty() {
            continue;
        }
        if let Some(token) = line.strip_prefix("STATE ") {
            cube = Some(orbit64::decode(token).expect("token"));
        } else {
            out.push(Run {
                cube: cube.take().expect("a state before the moves"),
                solution: line.parse().expect("moves"),
            });
        }
    }
    out
}

/// The first mistake (0-based position) and the printed moves used up.
fn plain(a: &Run, b: &Run) -> Option<(usize, usize)> {
    let d = diagnose(&a.cube, &b.cube, &a.solution, &Limits::default()).unwrap();
    match d.finding {
        Finding::Mistakes { mistakes, done, .. } => Some((mistakes[0].step - 1, done)),
        _ => None,
    }
}

fn scope(a: &Run, at_move: usize, sure_until: Option<usize>) -> Scope {
    let found = phases(&a.cube, &a.solution).unwrap();
    Scope {
        at_move,
        sure_until,
        phase_starts: found.iter().map(|p| p.moves.start).collect(),
    }
}

/// (pair index from 0, widening needed with only the stop known)
const EXPLAINED: [(usize, Widening); 6] = [
    (1, Widening::Phase),
    (4, Widening::Phase),
    (7, Widening::PreviousPhase),
    (9, Widening::Phase),
    (10, Widening::Phase),
    (11, Widening::PreviousPhase),
];

#[test]
fn the_stop_alone_finds_each_explainable_mistake_in_the_phase_stages() {
    let runs = runs();
    assert_eq!(runs.len(), 14);
    for (i, expected) in EXPLAINED {
        let (a, b) = (&runs[i], &runs[i + 1]);
        let (_, done) = plain(a, b).expect("explained without a stop");
        let s = diagnose_staged(
            &a.cube,
            &b.cube,
            &a.solution,
            &Limits::default(),
            Some(&scope(a, done, None)),
        )
        .unwrap();
        assert_eq!(s.stage, expected, "pair {}-{}", i + 1, i + 2);
        assert!(matches!(s.diagnosis.finding, Finding::Mistakes { .. }));
    }
}

#[test]
fn a_right_sure_move_is_found_at_once_and_a_wrong_one_only_costs_a_stage() {
    let runs = runs();
    for (i, _) in EXPLAINED {
        let (a, b) = (&runs[i], &runs[i + 1]);
        let (first, done) = plain(a, b).unwrap();
        let right = diagnose_staged(
            &a.cube,
            &b.cube,
            &a.solution,
            &Limits::default(),
            Some(&scope(a, done, Some(first))),
        )
        .unwrap();
        assert_eq!(right.stage, Widening::Sure, "pair {}-{}", i + 1, i + 2);
        // "Sure" of a move after the mistake: still found, one stage later.
        let wrong = diagnose_staged(
            &a.cube,
            &b.cube,
            &a.solution,
            &Limits::default(),
            Some(&scope(a, done, Some((first + 4).min(done - 1)))),
        )
        .unwrap();
        assert!(
            matches!(wrong.diagnosis.finding, Finding::Mistakes { .. }),
            "pair {}-{}",
            i + 1,
            i + 2
        );
    }
}
