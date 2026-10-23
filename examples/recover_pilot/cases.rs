//! The simulated human: follows the printed solution, makes fixed mistakes at
//! fixed distances before the stop, and never repairs anything.

use cubacle::puzzle::{Face, Move, MoveSequence, Turn};

use crate::data::PilotCube;
use crate::rng::Rng;

/// What was done wrong. The classes are those of the real attempts: a quarter
/// instead of a half turn or the reverse (7 of 16), a skipped move (5), the
/// wrong layer (2) and an extra move (2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Amount,
    Skip,
    Layer,
    Extra,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Amount => "amount",
            Kind::Skip => "skip",
            Kind::Layer => "layer",
            Kind::Extra => "extra",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Injected {
    /// Printed step (from 1) the mistake belongs to.
    pub step: usize,
    pub kind: Kind,
    /// Distance back from the stop.
    pub distance: usize,
}

pub struct Case {
    pub cube: usize,
    pub number: usize,
    pub stop: usize,
    pub mistakes: Vec<Injected>,
    /// What the human really did, up to the stop.
    pub done: MoveSequence,
    /// Run the plain search too (no stop), to compare.
    pub also_plain: bool,
}

/// Distances in the last 20 moves (12 of 20) and the last 50 (8 of 20).
const DISTANCES: [usize; 20] = [
    2, 2, 5, 5, 5, 10, 10, 10, 15, 15, 20, 20, 25, 25, 25, 35, 35, 35, 50, 50,
];
/// One, two and three mistakes: 7, 2 and 1 of 10.
const COUNTS: [usize; 10] = [1, 1, 2, 1, 1, 3, 1, 1, 2, 1];
/// How far before the end of the solution the person notices.
const STOP_BEFORE_END: [usize; 6] = [0, 0, 5, 0, 12, 0];

fn kind_from(draw: usize) -> Kind {
    match draw {
        0..=6 => Kind::Amount,
        7..=11 => Kind::Skip,
        12..=13 => Kind::Layer,
        _ => Kind::Extra,
    }
}

fn amount(mv: Move) -> Move {
    let turn = if mv.turn == Turn::Half {
        Turn::Clockwise
    } else {
        Turn::Half
    };
    Move::new(mv.face, mv.depth, mv.wide, turn)
}

/// Other layers: a single layer becomes the wide turn of that depth, a wide
/// turn of two layers becomes the outer face, a wider one loses a layer.
fn layer(mv: Move) -> Move {
    match (mv.wide, mv.depth) {
        (false, 1) => Move::new(mv.face, 2, true, mv.turn),
        (false, d) => Move::new(mv.face, d, true, mv.turn),
        (true, 2) => Move::new(mv.face, 1, false, mv.turn),
        (true, d) => Move::new(mv.face, d - 1, true, mv.turn),
    }
}

/// `per_cube[i]` cases for cube `i`; the series number drives all the choices.
pub fn build(cubes: &[PilotCube], per_cube: &[usize]) -> Vec<Case> {
    let mut out = Vec::new();
    let mut global = 0;
    for (index, cube) in cubes.iter().enumerate() {
        let length = cube.solution.len();
        for number in 0..per_cube[index] {
            let mut rng = Rng::new(7000 + global as u64);
            let stop = length - STOP_BEFORE_END[global % 6].min(length / 4);
            let count = COUNTS[(global * 3) % 10];
            // The first (farthest) distance, then nearer ones.
            let mut distances = vec![DISTANCES[(global * 7) % 20]];
            while distances.len() < count {
                let last = *distances.last().unwrap();
                let nearer: Vec<usize> = DISTANCES.iter().copied().filter(|&d| d < last).collect();
                distances.push(if nearer.is_empty() {
                    last.saturating_sub(1).max(1)
                } else {
                    nearer[rng.below(nearer.len())]
                });
            }
            let mut mistakes: Vec<Injected> = distances
                .iter()
                .map(|&d| {
                    let distance = d.min(stop - 1);
                    Injected {
                        step: stop - distance,
                        kind: kind_from(rng.below(16)),
                        distance,
                    }
                })
                .collect();
            mistakes.dedup_by_key(|m| m.step);
            let mut done = Vec::new();
            for step in 1..=stop {
                let printed = cube.solution.moves()[step - 1];
                match mistakes.iter().find(|m| m.step == step) {
                    None => done.push(printed),
                    Some(m) => match m.kind {
                        Kind::Skip => {}
                        Kind::Amount => done.push(amount(printed)),
                        Kind::Layer => done.push(layer(printed)),
                        Kind::Extra => {
                            let face = Face::ALL[rng.below(6)];
                            done.push(Move::new(face, 1, false, Turn::Clockwise));
                            done.push(printed);
                        }
                    },
                }
            }
            out.push(Case {
                cube: index,
                number,
                stop,
                mistakes,
                done: MoveSequence::new(done),
                also_plain: global % 3 == 0,
            });
            global += 1;
        }
    }
    out
}
