//! Makes tests/fixtures/recover_pilot.txt: ten fixed scrambles with the
//! solver's solution and the starts of its phases. Run once, commit the file.

use std::time::Instant;

use cubacle::phases::{from_stages, phases};
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, SolveOptions, Turn};

use crate::rng::Rng;
use crate::solvers::{kind_of, solver_for};

/// (size, scramble length) of the ten cubes, in file order.
pub const CUBES: [(usize, usize); 10] = [
    (7, 60),
    (7, 60),
    (7, 60),
    (7, 60),
    (7, 60),
    (7, 60),
    (5, 40),
    (5, 40),
    (4, 40),
    (4, 40),
];

/// A fixed scramble: outer turns, and from the 4x4x4 on wide turns of two
/// layers (and three on the 7x7x7).
fn scramble(size: usize, length: usize, rng: &mut Rng) -> MoveSequence {
    let widest = if size >= 7 {
        3
    } else if size >= 4 {
        2
    } else {
        1
    };
    let mut moves: Vec<Move> = Vec::new();
    while moves.len() < length {
        let face = Face::ALL[rng.below(6)];
        if moves.last().is_some_and(|m| m.face == face) {
            continue;
        }
        let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][rng.below(3)];
        let depth = 1 + rng.below(widest) as u8;
        moves.push(Move::new(face, depth, depth > 1, turn));
    }
    MoveSequence::new(moves)
}

pub fn generate() {
    println!("# Fixed cubes for the recover pilot (examples/recover_pilot).");
    println!("# CUBE <size> <seed>; SCRAMBLE from the solved cube; SOLUTION made by");
    println!("# the solver with its defaults; PHASES the 0-based first move of each phase.");
    for (index, (size, length)) in CUBES.into_iter().enumerate() {
        let seed = 1000 + index as u64;
        let kind = kind_of(size);
        let scramble = scramble(size, length, &mut Rng::new(seed));
        let cube = Facelets::solved(kind).apply(&scramble).expect("scramble");
        let started = Instant::now();
        let result = solver_for(kind).solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("the solver solves it").clone();
        assert!(cube.apply(&solution).expect("apply").is_solved());
        let found = from_stages(&result.stages, solution.len())
            .or_else(|| phases(&cube, &solution).ok())
            .unwrap_or_default();
        let starts: Vec<String> = found.iter().map(|p| p.moves.start.to_string()).collect();
        eprintln!(
            "cube {} ({size}x{size}x{size}): {} moves in {:?}",
            index + 1,
            solution.len(),
            started.elapsed()
        );
        println!();
        println!("CUBE {size} {seed}");
        println!("SCRAMBLE {scramble}");
        println!("SOLUTION {solution}");
        println!("PHASES {}", starts.join(" "));
    }
}
