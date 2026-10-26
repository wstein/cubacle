//! Tests of partial targets on the 3x3x3.

use super::super::cubie::{CORNER_FACELET, EDGE_FACELET};
use super::super::solver::Min2PhaseSolver;
use super::ANY;
use crate::puzzle::{
    Face, Facelets, Failure, MoveSequence, PuzzleKind, SolveOptions, SolveResult, Solver, Target,
};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % below
    }
}

fn scramble(rng: &mut Lcg, length: usize) -> Facelets {
    let faces = ["U", "R", "F", "D", "L", "B"];
    let turns = ["", "2", "'"];
    let text: Vec<String> = (0..length)
        .map(|_| format!("{}{}", faces[rng.next(6)], turns[rng.next(3)]))
        .collect();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&text.join(" ").parse().unwrap())
        .unwrap()
}

/// The cross on the down face: its four edges and the centers.
fn cross() -> Target {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let mut keep = [false; 54];
    for (slot, pair) in EDGE_FACELET.iter().enumerate() {
        if (4..8).contains(&slot) {
            keep[usize::from(pair[0])] = true;
            keep[usize::from(pair[1])] = true;
        }
    }
    Target::masked(&solved, |position| keep[position])
}

fn solve(start: &Facelets, target: &Target) -> SolveResult {
    Min2PhaseSolver::new().solve_to(start, target, &SolveOptions::default())
}

#[test]
fn the_cross_is_reached_in_at_most_eight_moves_and_is_proved_shortest() {
    let mut rng = Lcg(3);
    for _ in 0..4 {
        let start = scramble(&mut rng, 25);
        let target = cross();
        let result = solve(&start, &target);
        let moves = result.outcome.clone().unwrap();
        assert!(moves.len() <= 8, "{moves}");
        assert!(target.reached_by(&start, &moves));
        assert!(result.message.unwrap().starts_with("shortest"));
    }
}

#[test]
fn the_cross_distance_equals_a_brute_force_search_on_short_scrambles() {
    let turns: Vec<MoveSequence> = ["U", "R", "F", "D", "L", "B"]
        .iter()
        .flat_map(|f| ["", "2", "'"].map(|s| format!("{f}{s}").parse().unwrap()))
        .collect();
    let target = cross();
    let mut rng = Lcg(17);
    for _ in 0..6 {
        let start = scramble(&mut rng, 3);
        let mut frontier = vec![start.clone()];
        let mut seen = std::collections::HashSet::from([start.clone()]);
        let mut least = 0;
        while !frontier.iter().any(|s| target.matches(s)) {
            least += 1;
            let mut next = Vec::new();
            for state in &frontier {
                for turn in &turns {
                    let to = state.apply(turn).unwrap();
                    if seen.insert(to.clone()) {
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        let moves = solve(&start, &target).outcome.unwrap();
        assert_eq!(moves.len(), least);
    }
}

#[test]
fn an_already_solved_cross_needs_no_move() {
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"U2".parse().unwrap())
        .unwrap();
    // the up layer moved, the down cross did not
    let moves = solve(&start, &cross()).outcome.unwrap();
    assert!(moves.is_empty());
}

#[test]
fn freeing_edges_never_lengthens_the_answer() {
    let mut rng = Lcg(11);
    let start = scramble(&mut rng, 25);
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let mut last = usize::MAX;
    for edges in [4usize, 3, 2, 1, 0] {
        let mut keep = [false; 54];
        for slot in 4..4 + edges {
            keep[usize::from(EDGE_FACELET[slot][0])] = true;
            keep[usize::from(EDGE_FACELET[slot][1])] = true;
        }
        let target = Target::masked(&solved, |position| keep[position]);
        let moves = solve(&start, &target).outcome.unwrap();
        assert!(target.reached_by(&start, &moves));
        assert!(moves.len() <= last, "{edges} edges");
        last = moves.len();
    }
    assert_eq!(last, 0);
}

#[test]
fn a_single_edge_is_reached_in_few_moves() {
    // one fixed edge: the distance is at most 7 (a single edge needs few moves)
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U F".parse().unwrap())
        .unwrap();
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::masked(&solved, |p| p == 5 || p == 10);
    let moves = solve(&start, &target).outcome.unwrap();
    assert!(target.reached_by(&start, &moves));
    assert!(moves.len() <= 7);
}

#[test]
fn a_wrong_center_is_unreachable() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let mut sets = vec![ANY; 54];
    sets[4] = 1 << Face::R.index();
    let target = Target::from_sets(sets).unwrap();
    let result = solve(&solved, &target);
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().starts_with("unreachable"));
}

#[test]
fn an_edge_that_does_not_exist_is_contradictory() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let mut sets = vec![ANY; 54];
    // the UR edge shows U and D: no edge has opposite colors
    sets[5] = 1 << Face::U.index();
    sets[10] = 1 << Face::D.index();
    let target = Target::from_sets(sets).unwrap();
    let result = solve(&solved, &target);
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().starts_with("contradictory"));
}

#[test]
fn open_corners_give_the_shortest_over_the_few_completions() {
    // all of the cube fixed except the three stickers of one corner: one completion
    let mut rng = Lcg(21);
    let start = scramble(&mut rng, 12);
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let corner = CORNER_FACELET[0];
    let target = Target::masked(&solved, |p| !corner.contains(&(p as u8)));
    let result = solve(&start, &target);
    let moves = result.outcome.clone().unwrap();
    assert!(target.reached_by(&start, &moves));
    assert!(result.message.unwrap().starts_with("best found"));
}

#[test]
fn too_many_completions_are_refused_with_the_supported_forms() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    // only the up face fixed: far more than 24 completions, and corners are constrained
    let target = Target::masked(&solved, |p| p < 9);
    let result = solve(&solved, &target);
    assert_eq!(result.outcome, Err(Failure::Exhausted));
    assert!(result.message.unwrap().starts_with("unsupported"));
}
