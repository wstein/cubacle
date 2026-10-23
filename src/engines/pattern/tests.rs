use super::*;
use crate::puzzle::{Move, MoveSequence, Turn};

/// An engine that always gives the same answer.
struct Fixed {
    kind: PuzzleKind,
    outcome: Result<MoveSequence, Failure>,
}

impl Solver for Fixed {
    fn kind(&self) -> PuzzleKind {
        self.kind
    }

    fn solve(&self, _facelets: &Facelets, _options: &SolveOptions) -> SolveResult {
        SolveResult {
            outcome: self.outcome.clone(),
            message: None,
            metrics: SolveMetrics::default(),
            stages: Vec::new(),
        }
    }
}

fn seq(text: &str) -> MoveSequence {
    text.parse().unwrap()
}

/// The moves that undo `moves`.
fn undo(moves: &MoveSequence) -> MoveSequence {
    MoveSequence::new(
        moves
            .moves()
            .iter()
            .rev()
            .map(|mv| {
                let turn = match mv.turn {
                    Turn::Clockwise => Turn::CounterClockwise,
                    Turn::Half => Turn::Half,
                    Turn::CounterClockwise => Turn::Clockwise,
                };
                Move::new(mv.face, mv.depth, mv.wide, turn)
            })
            .collect(),
    )
}

/// The 15-move cube in a cube, every turn on the outer two layers.
const CUBE_IN_CUBE: &str = "2Fw 2Lw 2Fw 2Uw' 2Rw 2Uw 2Fw2 2Lw2 2Uw' 2Lw' 2Bw 2Dw' 2Bw' 2Lw2 2Uw";

fn shortcut(kind: PuzzleKind, outcome: Result<MoveSequence, Failure>) -> PatternShortcut {
    PatternShortcut::new(Box::new(Fixed { kind, outcome }))
}

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    Facelets::solved(kind).apply(&seq(moves)).unwrap()
}

#[test]
fn a_pattern_is_solved_when_the_engine_gives_up() {
    let pattern = cube(PuzzleKind::Cube5, CUBE_IN_CUBE);
    let solver = shortcut(PuzzleKind::Cube5, Err(Failure::BudgetExhausted));
    let result = solver.solve(&pattern, &SolveOptions::default());
    let solution = result.solution().expect("the pattern is found");
    assert!(solution.len() <= 15, "{} moves", solution.len());
    assert!(pattern.apply(solution).unwrap().is_solved());
    assert_eq!(result.stages.last().map(|s| s.end), Some(solution.len()));
}

#[test]
fn a_shorter_pattern_solution_replaces_a_long_one() {
    let pattern = cube(PuzzleKind::Cube7, CUBE_IN_CUBE);
    let long = undo(&seq(CUBE_IN_CUBE));
    let padded = MoveSequence::new(
        long.moves()
            .iter()
            .copied()
            .chain(seq("R R' 2R 2R'").moves().iter().copied())
            .collect(),
    );
    let solver = shortcut(PuzzleKind::Cube7, Ok(padded.clone()));
    let result = solver.solve(&pattern, &SolveOptions::default());
    let solution = result.solution().unwrap();
    assert!(solution.len() <= 15 && solution.len() < padded.len());
    assert!(pattern.apply(solution).unwrap().is_solved());
    assert!(result.message.unwrap().starts_with("known patterns"));
}

#[test]
fn the_engines_answer_stays_when_it_is_not_longer() {
    let pattern = cube(PuzzleKind::Cube5, CUBE_IN_CUBE);
    let own = undo(&seq(CUBE_IN_CUBE));
    let solver = shortcut(PuzzleKind::Cube5, Ok(own.clone()));
    let result = solver.solve(&pattern, &SolveOptions::default());
    assert_eq!(result.solution(), Some(&own));
    assert!(result.message.is_none());
}

#[test]
fn a_cube_that_is_no_pattern_keeps_the_engines_answer() {
    let scramble = seq("R U F' 2R 2U' 2F2 D L' B2 3Rw U2 F 2Dw' L2 B' R2 2L 2B U' 3Fw2 D2 R");
    let state = cube(PuzzleKind::Cube5, &scramble.to_string());
    let own = undo(&scramble);
    let solver = shortcut(PuzzleKind::Cube5, Ok(own.clone()));
    let result = solver.solve(&state, &SolveOptions::default());
    assert_eq!(result.solution(), Some(&own));
}

#[test]
fn a_move_limit_below_the_pattern_keeps_the_engines_answer() {
    let pattern = cube(PuzzleKind::Cube5, CUBE_IN_CUBE);
    let solver = shortcut(PuzzleKind::Cube5, Err(Failure::Exhausted));
    let options = SolveOptions::default().with_max_moves(3);
    assert_eq!(
        solver.solve(&pattern, &options).outcome,
        Err(Failure::Exhausted)
    );
}

#[test]
fn small_cubes_pass_through() {
    let state = cube(PuzzleKind::Cube3, "R U R' U'");
    let own = undo(&seq("R U R' U'"));
    let solver = shortcut(PuzzleKind::Cube3, Ok(own.clone()));
    assert_eq!(
        solver.solve(&state, &SolveOptions::default()).solution(),
        Some(&own)
    );
}

/// A 7x7x7 built by hand from a checkerboard and three six spots (Orbit64
/// token `C9bbhQdPbX8H1aJN4wwcaQyWAQ5PNcpbR93ezUq8FJnjmFSHFpM7Bwo2FnlNxFrsrEeYcbIIeW3BbYvgU49rEd8w34`);
/// the reduction engine needs 172 moves for it.
const HAND_BUILT: &str = "BFBFBFBFDFDFDFBFLRLFBFDRURDFBFLRLFBFDFDFDFBFBFBFBUDUDUDUDLDLDLDUDFBFDUDLBRBLDUDFBFDUDLDLDLDUDUDUDULRLRLRLRFRFRFRLRDUDRLRFUFUFRLRDUDRLRFRFRFRLRLRLRLFBFBFBFBUBUBUBFBRLRBFBULDLUBFBRLRBFBUBUBUBFBFBFBFDUDUDUDURURURUDUBFBUDURFLFRUDUBFBUDURURURUDUDUDUDRLRLRLRLBLBLBLRLUDULRLBDBDBLRLUDULRLBLBLBLRLRLRLR";

#[test]
fn a_hand_built_7x7x7_is_solved_in_38_moves() {
    let state = Facelets::parse(HAND_BUILT).unwrap();
    let solver = shortcut(PuzzleKind::Cube7, Err(Failure::BudgetExhausted));
    let result = solver.solve(&state, &SolveOptions::default());
    let solution = result.solution().expect("a product of four macros");
    assert_eq!(solution.len(), 38);
    assert!(state.apply(solution).unwrap().is_solved());
}
