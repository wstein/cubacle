//! The phases of a solution: the engine's own stages where it reports them,
//! else read from the cube.
//!
//! The 4x4x4 to 7x7x7 engines report their stages with the answer
//! ([`SolveResult::stages`](crate::puzzle::SolveResult::stages), the ends
//! already carried through the engines' normalizing); [`from_stages`] turns
//! them into phases. For an answer without stages (the 2x2x2 and 3x3x3, or a
//! solution from any other source) [`phases`] replays the solution on the
//! start cube and cuts it where a state predicate becomes true for good. The
//! table below is that fallback; the stages of each engine are in its module.
//!
//! | Cube | Phases |
//! | --- | --- |
//! | 2x2x2 | 1 `solve` |
//! | 3x3x3 | 1 `to the G1 subgroup`, 2 `solve` |
//! | 4x4x4, 5x5x5 | 1 `centers`, 2 `edges`, 3 `3x3 stage` |
//! | 6x6x6, 7x7x7 | 1 `inner centers`, 2 `centers`, 3 `edges`, 4 `3x3 stage` |
//!
//! Each predicate includes the ones before it. A phase ends at the first move
//! after which its predicate holds in every later state, including the final one;
//! a phase that is already true at its start is empty, and still counted. The
//! 4x4x4 engine, for one, makes the centers and the edges true at the same move;
//! then `edges` is an empty phase 2.
//!
//! * **G1** (3x3x3): the state is reachable with `U D R2 L2 F2 B2`, the
//!   Kociemba domino state: corners oriented, edges oriented and the four
//!   middle-layer edges in the middle layer. By colors: the U and D faces show
//!   only U or D colors (corners twisted right, up and down edges flipped
//!   right, no middle edge up or down), and the F or B sticker of each of the four
//!   middle-layer edges has an F or B color (those edges not flipped).
//! * **Centers** (4x4x4 and larger): on every face all center stickers show one
//!   color (the middle sticker of an odd cube included). The six colors are
//!   then distinct, because each color has exactly the center stickers of one face.
//! * **Inner centers** (6x6x6 and larger): the center stickers that are not in the
//!   outermost ring of a face show one color per face.
//! * **Edges**: on every edge, the stickers of all its pieces (wings and the
//!   middle piece of an odd cube) show one color on each of the two faces:
//!   the wings are paired.
//! * **3x3 stage** and **solve**: whatever is left, to the solved cube.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::puzzle::{locate, ApplyError, Face, Facelets, MoveSequence, PuzzleKind, SolveStage};

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

/// The phases of an answer of `total` moves from the stages its engine
/// reported, numbered from 1 in order; a stage with no moves is an empty
/// phase. `None` if the stages do not cover exactly `total` moves in order,
/// or if `total` is 0, so that the caller can fall back to [`phases`].
pub fn from_stages(stages: &[SolveStage], total: usize) -> Option<Vec<Phase>> {
    if total == 0 || stages.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(stages.len());
    let mut from = 0;
    for stage in stages {
        if stage.end < from {
            return None;
        }
        out.push(Phase {
            number: out.len() + 1,
            name: stage.name,
            moves: from..stage.end,
        });
        from = stage.end;
    }
    (from == total).then_some(out)
}

type Predicate = fn(&Layout, &[Face]) -> bool;

/// Which stickers the predicates look at, found from the geometry once.
struct Layout {
    face_len: usize,
    /// The stickers of the face interior, per face.
    centers: Vec<Vec<usize>>,
    /// The center stickers of each face that are not in its outermost ring
    /// (empty up to the 5x5x5).
    inner: Vec<Vec<usize>>,
    /// For each edge, its stickers on each of its two faces.
    edges: Vec<[Vec<usize>; 2]>,
    /// The stickers on the F or B face of the four middle-layer edges.
    slice: Vec<usize>,
}

impl Layout {
    fn new(kind: PuzzleKind) -> Layout {
        let n = kind.size() as i32;
        let face_len = kind.face_len();
        let mut centers = vec![Vec::new(); 6];
        let mut inner = vec![Vec::new(); 6];
        let mut edges: BTreeMap<[i32; 3], BTreeMap<usize, Vec<usize>>> = BTreeMap::new();
        for index in 0..kind.facelet_count() {
            let (position, _) = locate(kind, index);
            let outer = position.iter().filter(|c| c.abs() == n - 1).count();
            let face = index / face_len;
            if outer == 1 {
                centers[face].push(index);
                let (row, col) = (
                    (index % face_len) / n as usize,
                    (index % face_len) % n as usize,
                );
                if n >= 6
                    && row
                        .min(col)
                        .min(n as usize - 1 - row)
                        .min(n as usize - 1 - col)
                        >= 2
                {
                    inner[face].push(index);
                }
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
            centers,
            inner,
            edges: pairs,
            slice,
        }
    }
}

fn uniform(stickers: &[usize], state: &[Face]) -> bool {
    stickers.windows(2).all(|w| state[w[0]] == state[w[1]])
}

fn inner_centers_solved(layout: &Layout, state: &[Face]) -> bool {
    layout.inner.iter().all(|face| uniform(face, state))
}

fn centers_solved(layout: &Layout, state: &[Face]) -> bool {
    layout.centers.iter().all(|face| uniform(face, state))
}

fn edges_paired(layout: &Layout, state: &[Face]) -> bool {
    centers_solved(layout, state)
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
        PuzzleKind::Cube4 | PuzzleKind::Cube5 => vec![
            ("centers", Some(centers_solved)),
            ("edges", Some(edges_paired)),
            ("3x3 stage", None),
        ],
        _ => vec![
            ("inner centers", Some(inner_centers_solved)),
            ("centers", Some(centers_solved)),
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
