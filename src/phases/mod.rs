//! The phases of a solution, read from the cube and not from the engine.
//!
//! An engine's own phase boundaries are not trusted: the answers are merged and
//! normalised after the search, and the engines differ. Instead the solution is
//! replayed on the start cube and cut where a state predicate becomes true for
//! good, so the same rule works for every engine and for a solution from any
//! source.
//!
//! | Cube | Phases |
//! | --- | --- |
//! | 2x2x2 | 1 `solve` |
//! | 3x3x3 | 1 `to the G1 subgroup`, 2 `solve` |
//! | 4x4x4 to 7x7x7 | 1 `centres`, 2 `edges`, 3 `3x3 stage` |
//!
//! Each predicate includes the ones before it. A phase ends at the first move
//! after which its predicate holds in every later state, including the final one;
//! a phase that is already true at its start is empty, and still counted. The
//! 4x4x4 engine, for one, makes the centres and the edges true at the same move;
//! then `edges` is an empty phase 2.
//!
//! * **G1** (3x3x3): the state is reachable with `U D R2 L2 F2 B2`, the
//!   Kociemba domino state: corners oriented, edges oriented and the four
//!   middle-layer edges in the middle layer. By colours: the U and D faces show
//!   only U or D colours (corners twisted right, up and down edges flipped
//!   right, no middle edge up or down), and the F or B sticker of each of the four
//!   middle-layer edges has an F or B colour (those edges not flipped).
//! * **Centres** (4x4x4 and larger): on every face all centre stickers show one
//!   colour (the middle sticker of an odd cube included). The six colours are
//!   then distinct, because each colour has exactly the centre stickers of one face.
//! * **Edges**: on every edge, the stickers of all its pieces (wings and the
//!   middle piece of an odd cube) show one colour on each of the two faces:
//!   the wings are paired.
//! * **3x3 stage** and **solve**: whatever is left, to the solved cube.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::puzzle::{locate, ApplyError, Face, Facelets, MoveSequence, PuzzleKind};

/// One phase of a solution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase {
    /// The number of the phase among all of its cube, from 1; empty ones count.
    pub number: usize,
    /// A short name such as `edges`.
    pub name: &'static str,
    /// The moves of the phase, as indices into the solution.
    pub moves: Range<usize>,
}

type Predicate = fn(&Layout, &[Face]) -> bool;

/// Which stickers the predicates look at, found from the geometry once.
struct Layout {
    face_len: usize,
    /// The stickers of the face interior, per face.
    centres: Vec<Vec<usize>>,
    /// For each edge, its stickers on each of its two faces.
    edges: Vec<[Vec<usize>; 2]>,
    /// The stickers on the F or B face of the four middle-layer edges.
    slice: Vec<usize>,
}

impl Layout {
    fn new(kind: PuzzleKind) -> Layout {
        let n = kind.size() as i32;
        let face_len = kind.face_len();
        let mut centres = vec![Vec::new(); 6];
        let mut edges: BTreeMap<[i32; 3], BTreeMap<usize, Vec<usize>>> = BTreeMap::new();
        for index in 0..kind.facelet_count() {
            let (position, _) = locate(kind, index);
            let outer = position.iter().filter(|c| c.abs() == n - 1).count();
            let face = index / face_len;
            if outer == 1 {
                centres[face].push(index);
            } else if outer == 2 {
                // the same key for every piece along one edge
                let key = position.map(|c| if c.abs() == n - 1 { c } else { 0 });
                edges
                    .entry(key)
                    .or_default()
                    .entry(face)
                    .or_default()
                    .push(index);
            }
        }
        let side = |face: usize| [1, 2, 4, 5].contains(&face);
        let mut slice = Vec::new();
        let mut pairs = Vec::new();
        for faces in edges.into_values() {
            if faces.keys().all(|&face| side(face)) {
                slice.extend(
                    faces
                        .iter()
                        .filter(|(&face, _)| face == 2 || face == 5)
                        .flat_map(|(_, stickers)| stickers.iter().copied()),
                );
            }
            let mut sides = faces.into_values();
            if let (Some(a), Some(b)) = (sides.next(), sides.next()) {
                pairs.push([a, b]);
            }
        }
        Layout {
            face_len,
            centres,
            edges: pairs,
            slice,
        }
    }
}

fn uniform(stickers: &[usize], state: &[Face]) -> bool {
    stickers.windows(2).all(|w| state[w[0]] == state[w[1]])
}

fn centres_solved(layout: &Layout, state: &[Face]) -> bool {
    layout.centres.iter().all(|face| uniform(face, state))
}

fn edges_paired(layout: &Layout, state: &[Face]) -> bool {
    centres_solved(layout, state)
        && layout
            .edges
            .iter()
            .all(|[a, b]| uniform(a, state) && uniform(b, state))
}

fn in_g1(layout: &Layout, state: &[Face]) -> bool {
    let vertical = |face: Face| matches!(face, Face::U | Face::D);
    let ends = |range: Range<usize>| state[range].iter().all(|&f| vertical(f));
    let (up, down) = (0..layout.face_len, 3 * layout.face_len..4 * layout.face_len);
    let front_back = |face: Face| matches!(face, Face::F | Face::B);
    ends(up) && ends(down) && layout.slice.iter().all(|&i| front_back(state[i]))
}

/// The stages of a kind: number, name and predicate; the last one is "solved".
fn stages(kind: PuzzleKind) -> Vec<(&'static str, Option<Predicate>)> {
    match kind {
        PuzzleKind::Cube2 => vec![("solve", None)],
        PuzzleKind::Cube3 => vec![("to the G1 subgroup", Some(in_g1)), ("solve", None)],
        _ => vec![
            ("centres", Some(centres_solved)),
            ("edges", Some(edges_paired)),
            ("3x3 stage", None),
        ],
    }
}

/// Cuts `solution` into phases by replaying it on `start`.
///
/// The phases are in order and cover the solution without gaps; a phase that
/// is already true at its start is empty. A solution of no moves has no phases.
/// A solution that does not end in a state that satisfies a predicate puts
/// that phase's end at the end of the solution.
pub fn phases(start: &Facelets, solution: &MoveSequence) -> Result<Vec<Phase>, ApplyError> {
    let kind = start.kind();
    let layout = Layout::new(kind);
    let stages = stages(kind);
    let total = solution.len();
    let mut states = Vec::with_capacity(total + 1);
    let mut state = start.clone();
    states.push(state.clone());
    for mv in solution.moves() {
        state = state.apply(&MoveSequence::new(vec![*mv]))?;
        states.push(state.clone());
    }
    if total == 0 {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut from = 0;
    for (name, predicate) in &stages {
        // the end: the first index from which the predicate holds to the last state
        let end = match predicate {
            None => total,
            Some(holds) => {
                let mut end = total;
                while end > from && holds(&layout, states[end - 1].stickers()) {
                    end -= 1;
                }
                if holds(&layout, states[total].stickers()) {
                    end
                } else {
                    total
                }
            }
        };
        out.push(Phase {
            number: out.len() + 1,
            name,
            moves: from..end,
        });
        from = end;
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
