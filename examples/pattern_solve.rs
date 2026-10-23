//! Finds a generator by solving the pattern with our own solvers and
//! inverting the solution (3x3x3 to 5x5x5). The result is a valid but not
//! necessarily shortest generator, so it is printed with the status
//! `best-known`, in the outer-block metric.
//!
//! ```text
//! cargo run --release --example pattern_solve -- ID SIZE [MAX_MOVES] [BUDGET_MS]
//! ```

use std::sync::Arc;
use std::time::Duration;

use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::Notation;
use cubacle::patterns;
use cubacle::puzzle::{Facelets, Move, MoveSequence, PuzzleKind, SolveOptions, Solver, Turn};
use cubacle::ranges::{compress, write};

fn inverse(mv: &Move) -> Move {
    let turn = match mv.turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    };
    Move::new(mv.face, mv.depth, mv.wide, turn)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [id, size, rest @ ..] = &args[..] else {
        eprintln!("usage: pattern_solve ID SIZE [MAX_MOVES] [BUDGET_MS]");
        std::process::exit(2);
    };
    let kind = PuzzleKind::of_size(size.parse().expect("size")).expect("cube size");
    let max_moves: u8 = rest.first().map_or(25, |s| s.parse().expect("moves"));
    let budget: u64 = rest.get(1).map_or(20_000, |s| s.parse().expect("ms"));
    let target = patterns::find(id)
        .expect("pattern id")
        .state(kind)
        .expect("defined for this size");
    let three = Arc::new(Min2PhaseSolver::new());
    let solver: Box<dyn Solver> = match kind {
        PuzzleKind::Cube3 => Box::new(Min2PhaseSolver::new()),
        PuzzleKind::Cube4 => Box::new(FourSolver::new(three)),
        PuzzleKind::Cube5 => Box::new(FiveSolver::new(three)),
        _ => {
            eprintln!("no solver for this size");
            return;
        }
    };
    let options = SolveOptions::default()
        .with_max_moves(max_moves)
        .with_time_budget(Duration::from_millis(budget));
    let result = solver.solve(&target, &options);
    let Some(solution) = result.solution() else {
        eprintln!("{id} {}: no solution ({result:?})", kind.size());
        return;
    };
    let reversed: Vec<Move> = solution.moves().iter().rev().map(inverse).collect();
    let generator = MoveSequence::new(reversed);
    let built = Facelets::solved(kind).apply(&generator).expect("fits");
    assert_eq!(built, target, "inverse of the solution builds the pattern");
    let text: Vec<String> = generator.moves().iter().map(ToString::to_string).collect();
    println!(
        "{id} | {} | outer | best-known | {} | {}",
        kind.size(),
        text.len(),
        text.join(" ")
    );
    // The same moves with adjacent turns of one axis merged into ranges.
    let packed = compress(kind, &generator);
    if packed.len() < text.len() {
        println!(
            "{id} | {} | range | best-known | {} | {}",
            kind.size(),
            packed.len(),
            write(Notation::Wca, kind, &packed)
        );
    }
}
