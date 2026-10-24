use std::collections::hash_map::Entry;
use std::collections::HashMap;

use rand::RngExt;

use super::model::{tables, Cube, FACE_NAMES, MOVES, TURN_NAMES};
use super::*;

/// Answers with a fixed solution, as the injected default engine would.
struct Fixed(MoveSequence);

impl Solver for Fixed {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube3
    }

    fn solve(&self, _facelets: &Facelets, _options: &SolveOptions) -> SolveResult {
        SolveResult {
            outcome: Ok(self.0.clone()),
            message: None,
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}

/// The moves that undo `moves`.
fn undo(moves: &[usize]) -> MoveSequence {
    let text: Vec<String> = moves
        .iter()
        .rev()
        .map(|&m| format!("{}{}", FACE_NAMES[m / 3], TURN_NAMES[2 - m % 3]))
        .collect();
    text.join(" ").parse().unwrap()
}

fn solver(dir: &str, upper: &[usize]) -> OptimalSolver {
    let dir = std::env::temp_dir().join(format!("cubacle-optimal-{dir}-{}", std::process::id()));
    OptimalSolver::new(Arc::new(Fixed(undo(upper))), Tables::Tiny).with_cache_dir(Some(dir))
}

fn scramble(moves: &[usize]) -> Facelets {
    let text: Vec<String> = moves
        .iter()
        .map(|&m| format!("{}{}", FACE_NAMES[m / 3], TURN_NAMES[m % 3]))
        .collect();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&text.join(" ").parse().unwrap())
        .unwrap()
}

/// Exact distances up to depth 5 by a plain breadth first search.
fn distances() -> HashMap<Cube, u8> {
    let tables = tables();
    let mut seen = HashMap::from([(Cube::SOLVED, 0u8)]);
    let mut frontier = vec![Cube::SOLVED];
    for depth in 1..=5 {
        let mut next = Vec::new();
        for cube in &frontier {
            for m in 0..MOVES {
                let child = cube.turned_portable(tables, m);
                if let Entry::Vacant(slot) = seen.entry(child) {
                    slot.insert(depth);
                    next.push(child);
                }
            }
        }
        frontier = next;
    }
    seen
}

#[test]
fn short_scrambles_get_their_exact_length() {
    let exact = distances();
    let mut rng = rand::rng();
    for _ in 0..40 {
        let moves: Vec<usize> = (0..rng.random_range(1..=5))
            .map(|_| rng.random_range(0..MOVES))
            .collect();
        let solver = solver("exact", &moves);
        let cube = scramble(&moves);
        let wanted = exact[&Cube::from_facelets(&cube).unwrap()];
        let report = solver
            .solve_report(&cube, &SolveOptions::default(), &Limits::default())
            .unwrap();
        assert!(report.proven);
        assert_eq!(report.solution.len(), usize::from(wanted), "{moves:?}");
        assert!(cube.apply(&report.solution).unwrap().is_solved());
    }
}

#[test]
fn the_lower_bound_never_exceeds_the_distance() {
    let exact = distances();
    let solver = solver("bound", &[]);
    for (cube, &distance) in exact.iter().take(3000) {
        let bound = solver.heuristic_bound(cube);
        assert!(bound <= distance);
    }
}

#[test]
fn a_node_budget_falls_back_to_the_upper_bound() {
    let mut rng = rand::rng();
    let moves: Vec<usize> = (0..25).map(|_| rng.random_range(0..MOVES)).collect();
    let solver = solver("budget", &moves);
    let cube = scramble(&moves);
    let limits = Limits {
        nodes: Some(50_000),
        ..Limits::default()
    };
    let report = solver
        .solve_report(&cube, &SolveOptions::default(), &limits)
        .unwrap();
    assert!(cube.apply(&report.solution).unwrap().is_solved());
    assert!(report.solution.len() <= usize::from(report.upper_bound));
    assert!(report.lower_bound <= report.upper_bound);
}

#[test]
fn the_solved_cube_needs_no_move() {
    let report = solver("solved", &[])
        .solve_report(
            &Facelets::solved(PuzzleKind::Cube3),
            &SolveOptions::default(),
            &Limits::default(),
        )
        .unwrap();
    assert!(report.proven);
    assert!(report.solution.is_empty());
}
