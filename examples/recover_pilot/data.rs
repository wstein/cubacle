//! The fixed cubes of tests/fixtures/recover_pilot.txt.

use cubacle::puzzle::{Facelets, MoveSequence};

use crate::solvers::kind_of;

pub struct PilotCube {
    pub size: usize,
    /// The cube the solution was made for.
    pub original: Facelets,
    pub solution: MoveSequence,
    /// The 0-based first move of each phase of the solution.
    pub phase_starts: Vec<usize>,
}

pub fn load() -> Vec<PilotCube> {
    let text = include_str!("../../tests/fixtures/recover_pilot.txt");
    let mut out = Vec::new();
    let (mut size, mut scramble, mut solution) = (0, None, None);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("CUBE ") {
            size = rest.split_whitespace().next().unwrap().parse().unwrap();
        } else if let Some(rest) = line.strip_prefix("SCRAMBLE ") {
            scramble = Some(rest.parse::<MoveSequence>().expect("scramble"));
        } else if let Some(rest) = line.strip_prefix("SOLUTION ") {
            solution = Some(rest.parse::<MoveSequence>().expect("solution"));
        } else if let Some(rest) = line.strip_prefix("PHASES ") {
            let kind = kind_of(size);
            out.push(PilotCube {
                size,
                original: Facelets::solved(kind)
                    .apply(&scramble.take().expect("scramble"))
                    .expect("apply"),
                solution: solution.take().expect("solution"),
                phase_starts: rest
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect(),
            });
        }
    }
    out
}
