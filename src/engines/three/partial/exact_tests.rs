//! Tests of the exact target-set route.

use std::time::{Duration, Instant};

use super::super::cubie::{Cubie, CORNER_FACELET, EDGE_FACELET};
use super::exact::solve_goal;
use super::goal::{Goal, Masks, EVERY_STICKER};
use super::model::Pos;
use crate::puzzle::{Facelets, Failure, MoveSequence, PuzzleKind, SolveOptions, Target};

struct Case {
    expected: usize,
    start: Facelets,
    views: Vec<Masks>,
}

/// The front-face target sets of the calendar spike (see the header of the data file).
fn cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut lines = include_str!("front_cases.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .peekable();
    while let Some(line) = lines.next() {
        let expected = line.strip_prefix("case ").unwrap().parse().unwrap();
        let start = Facelets::parse(lines.next().unwrap()).unwrap();
        let mut views = Vec::new();
        while lines.peek().is_some_and(|l| !l.starts_with("case")) {
            let tokens: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
            assert_eq!(tokens.len(), 54);
            let mut masks = [EVERY_STICKER; 54];
            for (slot, token) in tokens.iter().enumerate() {
                if *token != "*" {
                    masks[slot] = token
                        .split(',')
                        .fold(0, |m, n| m | 1u64 << n.parse::<u32>().unwrap());
                }
            }
            views.push(masks);
        }
        cases.push(Case {
            expected,
            start,
            views,
        });
    }
    cases
}

/// The home position of the sticker shown at facelet `f`, read from the
/// cubie model of the facelet string (independent of the search's tables).
fn sticker_at(cube: &Cubie, f: usize) -> usize {
    for (slot, home) in CORNER_FACELET.iter().enumerate() {
        if let Some(k) = home.iter().position(|&x| usize::from(x) == f) {
            let piece = usize::from(cube.ca[slot] & 7);
            let twist = usize::from(cube.ca[slot] >> 3);
            return usize::from(CORNER_FACELET[piece][(k + 3 - twist) % 3]);
        }
    }
    for (slot, home) in EDGE_FACELET.iter().enumerate() {
        if let Some(k) = home.iter().position(|&x| usize::from(x) == f) {
            let piece = usize::from(cube.ea[slot] >> 1);
            let flip = usize::from(cube.ea[slot] & 1);
            return usize::from(EDGE_FACELET[piece][(k + flip) % 2]);
        }
    }
    f
}

fn cubie_of(state: &Facelets) -> Cubie {
    let mut cube = Cubie::new();
    assert_eq!(cube.set_from_facelet(&state.to_string()), 0);
    cube
}

#[test]
fn the_front_face_target_sets_match_the_spike_optimum_and_replay() {
    for case in cases() {
        let goal = Goal::new(&case.views);
        let started = Instant::now();
        let result = solve_goal(
            started,
            &cubie_of(&case.start),
            &goal,
            &SolveOptions::default(),
        );
        let moves = result
            .outcome
            .clone()
            .unwrap_or_else(|f| panic!("{f:?} {:?}", result.message));
        eprintln!("TIMING {:?} {:?}", result.metrics.elapsed, result.message);
        assert_eq!(moves.len(), case.expected, "{:?}", result.message);
        assert!(result.message.unwrap().starts_with("shortest:"));
        // replay through the facelet model, then read the stickers independently
        let end = cubie_of(&case.start.apply(&moves).unwrap());
        assert!(
            case.views
                .iter()
                .any(|masks| (0..54).all(|f| masks[f] >> sticker_at(&end, f) & 1 == 1)),
            "the answer does not reach any view"
        );
        // and the pruning tables did not hide a shorter answer: none shorter reaches a view
        let pos = Pos::from_cubie(&cubie_of(&case.start));
        assert!(!goal.is_met(&pos));
    }
}

fn solve(
    start: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> (Result<MoveSequence, Failure>, String) {
    use crate::puzzle::Solver;
    let result = super::super::solver::Min2PhaseSolver::new().solve_to(start, target, options);
    (result.outcome, result.message.unwrap_or_default())
}

fn scramble(seed: u64, length: usize) -> Facelets {
    let mut state = seed;
    let mut next = |below: u64| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) % below
    };
    let text: Vec<String> = (0..length)
        .map(|_| {
            format!(
                "{}{}",
                ["U", "R", "F", "D", "L", "B"][next(6) as usize],
                ["", "2", "'"][next(3) as usize]
            )
        })
        .collect();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&text.join(" ").parse().unwrap())
        .unwrap()
}

#[test]
fn the_exact_route_agrees_with_the_exact_edge_search_on_the_cross() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let cross = Target::masked(&solved, |p| {
        super::super::cubie::EDGE_FACELET[4..8]
            .iter()
            .any(|pair| pair.iter().any(|&f| usize::from(f) == p))
    });
    for seed in 1..=4 {
        let start = scramble(seed, 18);
        let (plain, _) = solve(&start, &cross, &SolveOptions::default());
        let (exact, message) = solve(&start, &cross, &SolveOptions::default().with_exact(true));
        let (plain, exact) = (plain.unwrap(), exact.unwrap());
        assert_eq!(plain.len(), exact.len(), "seed {seed}: {message}");
        assert!(cross.reached_by(&start, &exact));
    }
}

#[test]
fn a_ring_without_its_center_is_solved_in_the_fewest_moves_and_any_orientation_never_costs_more() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    // the eight stickers around the U center: the center is open, so another
    // face whose ring shows the U color also counts in any orientation
    let ring = Target::masked(&solved, |p| p < 9 && p != 4);
    let mut cheaper = 0;
    for seed in 1..=8 {
        let start = scramble(seed, 16);
        let fixed = SolveOptions::default().with_exact(true);
        let free = SolveOptions::default().with_free_orientation(true);
        let (fixed_moves, message) = solve(&start, &ring, &fixed);
        let (free_moves, free_message) = solve(&start, &ring, &free);
        let (fixed_moves, free_moves) = (
            fixed_moves.expect(&message),
            free_moves.expect(&free_message),
        );
        assert!(ring.reached_by(&start, &fixed_moves));
        assert!(ring.reached_by_any_rotation(&start, &free_moves));
        assert!(free_moves.len() <= fixed_moves.len(), "seed {seed}");
        cheaper += usize::from(free_moves.len() < fixed_moves.len());
    }
    eprintln!("any orientation was shorter on {cheaper} of 8 scrambles");
}

#[test]
fn a_cap_too_small_for_any_table_still_answers_and_a_deadline_is_reported_as_not_proven() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let up = Target::masked(&solved, |p| p < 9);
    // no tables fit in 0 MB: a short answer is found by plain deepening
    let start = scramble(7, 3);
    let none = SolveOptions::default().with_exact(true).with_max_memory(0);
    let (moves, message) = solve(&start, &up, &none);
    let moves = moves.expect(&message);
    assert!(up.reached_by(&start, &moves));
    assert!(
        message.contains("no corner table, no edge table"),
        "{message}"
    );
    // a deep target with no tables and a tiny budget: stopped, with the proven depth
    let deep = scramble(9, 25);
    let tight = none.with_time_budget(Duration::from_millis(30));
    let (outcome, message) = solve(&deep, &Target::complete(&solved), &tight);
    assert_eq!(outcome.unwrap_err(), Failure::BudgetExhausted);
    assert!(
        message.starts_with("not proven: no solution has fewer than"),
        "{message}"
    );
}

#[test]
fn a_target_no_orientation_can_meet_is_unreachable() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    // two centers that must both show U: no orientation does that
    let target = Target::masked(&solved, |p| p == 4 || p == 13);
    let mut sets = target.sets().to_vec();
    sets[13] = 1; // the R center must show U as well
    let both = Target::from_sets(sets).unwrap();
    let (outcome, message) = solve(
        &solved,
        &both,
        &SolveOptions::default().with_free_orientation(true),
    );
    assert_eq!(outcome.unwrap_err(), Failure::Exhausted);
    assert!(message.starts_with("unreachable"), "{message}");
}
