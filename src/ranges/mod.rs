//! Post-solve compression: adjacent turns of one axis written as layer ranges.
//!
//! All turns about one axis commute, so a run of them is described by the net
//! quarter turns of each layer. Neighboring layers with the same net turn are
//! one [`LayerMove`] (`3Rw Rw'` is layer 3 alone; `2R 3R` is `2-3r` in SiGN,
//! `2-3Rw` in WCA). No search is involved, and a run is only replaced when the
//! replacement is shorter. The result is exactly the same state, never a
//! rotation of it. Only contiguous blocks are formed.

use std::collections::VecDeque;
use std::sync::OnceLock;

use crate::notation::Notation;
use crate::puzzle::{Face, Facelets, LayerMove, Move, MoveSequence, PuzzleKind, Turn};

fn axis_of(face: Face) -> usize {
    face.index() % 3
}

fn is_negative(face: Face) -> bool {
    face.index() >= 3
}

fn quarters(turn: Turn) -> u8 {
    match turn {
        Turn::Clockwise => 1,
        Turn::Half => 2,
        Turn::CounterClockwise => 3,
    }
}

fn turn_of(quarters: u8) -> Turn {
    match quarters % 4 {
        1 => Turn::Clockwise,
        2 => Turn::Half,
        _ => Turn::CounterClockwise,
    }
}

/// Adds a move to the net turns of an axis, layers counted from the positive face.
fn add(net: &mut [u8], mv: &Move) {
    let n = net.len();
    let d = usize::from(mv.depth);
    let negative = is_negative(mv.face);
    let q = quarters(mv.turn);
    let q = if negative { (4 - q) % 4 } else { q };
    let layers: Vec<usize> = match (mv.wide, negative) {
        (false, false) => vec![d - 1],
        (false, true) => vec![n - d],
        (true, false) => (0..d).collect(),
        (true, true) => (n - d..n).collect(),
    };
    for layer in layers {
        net[layer] = (net[layer] + q) % 4;
    }
}

/// The blocks of equal net turn in an axis, as `(first, last, quarters)`, 0-based.
fn blocks(net: &[u8]) -> Vec<(usize, usize, u8)> {
    let mut list: Vec<(usize, usize, u8)> = Vec::new();
    for (layer, &q) in net.iter().enumerate() {
        if q == 0 {
            continue;
        }
        match list.last_mut() {
            Some(last) if last.1 + 1 == layer && last.2 == q => last.1 = layer,
            _ => list.push((layer, layer, q)),
        }
    }
    list
}

/// One block as a move on the nearer face of its axis.
fn block_move(n: usize, axis: usize, (first, last, q): (usize, usize, u8)) -> LayerMove {
    if first <= n - 1 - last {
        LayerMove::new(
            Face::ALL[axis],
            (first + 1) as u8,
            (last + 1) as u8,
            turn_of(q),
        )
    } else {
        LayerMove::new(
            Face::ALL[axis + 3],
            (n - last) as u8,
            (n - first) as u8,
            turn_of((4 - q) % 4),
        )
    }
}

/// The run itself, or its net blocks when those are fewer.
fn compress_run(n: usize, base: usize, run: &[Move], out: &mut Vec<(LayerMove, usize)>) {
    let axis = axis_of(run[0].face);
    let mut net = vec![0u8; n];
    for mv in run {
        add(&mut net, mv);
    }
    let found = blocks(&net);
    // Every layer turned alike is a rotation of the cube: keep the run.
    let rotation = found.len() == 1 && found[0].1 - found[0].0 + 1 == n;
    if !rotation && found.len() < run.len() {
        out.extend(found.into_iter().map(|b| (block_move(n, axis, b), base)));
    } else {
        out.extend(
            run.iter()
                .enumerate()
                .map(|(i, mv)| (LayerMove::from(*mv), base + i)),
        );
    }
}

/// `moves` with each run of turns about one axis replaced by layer ranges
/// where that is shorter. Replaying the result gives the same state.
pub fn compress(kind: PuzzleKind, moves: &MoveSequence) -> Vec<LayerMove> {
    compress_indexed(kind, moves)
        .into_iter()
        .map(|(mv, _)| mv)
        .collect()
}

/// [`compress`], each step with the index of the move where it starts: the
/// first move of its run for a range, its own index for a move left alone.
/// The indices never decrease.
pub fn compress_indexed(kind: PuzzleKind, moves: &MoveSequence) -> Vec<(LayerMove, usize)> {
    let n = kind.size();
    let mut out = Vec::new();
    let all = moves.moves();
    let mut start = 0;
    while start < all.len() {
        let axis = axis_of(all[start].face);
        let mut end = start + 1;
        while end < all.len() && axis_of(all[end].face) == axis {
            end += 1;
        }
        compress_run(n, start, &all[start..end], &mut out);
        start = end;
    }
    out
}

/// The plain moves a compressed sequence stands for.
pub fn expand(moves: &[LayerMove]) -> MoveSequence {
    MoveSequence::new(moves.iter().flat_map(LayerMove::to_moves).collect())
}

/// One compressed move in a notation: `2-3Rw` in WCA, `2-3r` in SiGN, `V2R`
/// or `N2-4R` in SSE; every move that is not a range is written as the
/// notation writes it.
pub fn write_move(notation: Notation, kind: PuzzleKind, mv: &LayerMove) -> String {
    if notation == Notation::Sse {
        return crate::notation::write_sse_layer_move(kind, mv);
    }
    match mv.to_move() {
        Some(plain) => notation.write_move(kind, &plain),
        None => {
            let letter = match notation {
                Notation::Sign => mv.face.letter().to_ascii_lowercase().to_string(),
                _ => format!("{}w", mv.face.letter()),
            };
            format!("{}-{}{letter}{}", mv.from, mv.to, mv.turn_suffix())
        }
    }
}

/// A compressed sequence, separated by single spaces.
pub fn write(notation: Notation, kind: PuzzleKind, moves: &[LayerMove]) -> String {
    let words: Vec<String> = moves
        .iter()
        .map(|mv| write_move(notation, kind, mv))
        .collect();
    words.join(" ")
}

/// The opt-in post-solve optimizations (`--optimize`, the `optimize` request field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Optimize {
    /// Write adjacent same-axis turns as layer ranges.
    Ranges,
    /// Minimize the engine move count within each axis run, preserving the frame.
    Moves,
    /// Minimize block turns within each axis run, allowing a final cube rotation.
    Easy,
}

impl std::str::FromStr for Optimize {
    type Err = String;

    fn from_str(text: &str) -> Result<Optimize, String> {
        match text {
            "ranges" => Ok(Optimize::Ranges),
            "moves" => Ok(Optimize::Moves),
            "easy" => Ok(Optimize::Easy),
            _ => Err(format!(
                "optimize must be ranges, moves, or easy, not {text:?}"
            )),
        }
    }
}

/// Rewrite a sequence for the chosen objective. `Moves` counts each plain
/// layer or outer-block turn once, independently of angle. It uses the
/// axis normalizer with single layers enabled. `Easy` counts each contiguous
/// block once (BTM); it may leave the final cube in a different orientation.
/// Both are local axis-run optimizations, not globally shortest solutions.
pub fn optimize(kind: PuzzleKind, solution: &MoveSequence, objective: Optimize) -> Vec<LayerMove> {
    if !solution.fits(kind) {
        return solution
            .moves()
            .iter()
            .copied()
            .map(LayerMove::from)
            .collect();
    }
    match objective {
        Optimize::Ranges => compress(kind, solution),
        Optimize::Moves => crate::puzzle::normalize(kind.size(), solution.moves(), true)
            .into_iter()
            .map(LayerMove::from)
            .collect(),
        Optimize::Easy => easy(kind, solution),
    }
}

/// Inverse face permutation of a whole-cube rotation about `face`.
fn inverse_rotation(kind: PuzzleKind, face: Face, q: u8) -> [Face; 6] {
    let turns = (1..=kind.size())
        .map(|depth| {
            if depth <= kind.size() / 2 {
                Move::new(face, depth as u8, false, turn_of(q))
            } else {
                Move::new(
                    Face::ALL[face.index() + 3],
                    (kind.size() + 1 - depth) as u8,
                    false,
                    turn_of(4 - q),
                )
            }
        })
        .collect();
    let rotated = Facelets::solved(kind)
        .apply(&MoveSequence::new(turns))
        .expect("whole-cube rotation fits");
    std::array::from_fn(|i| rotated.stickers()[i * kind.face_len()])
}

/// Breadth-first table for every axis description using contiguous blocks.
struct BlockTable {
    distance: Vec<u8>,
    previous: Vec<(usize, usize, usize, u8)>,
}

fn block_table(n: usize) -> &'static BlockTable {
    static TABLES: [OnceLock<BlockTable>; 6] = [const { OnceLock::new() }; 6];
    TABLES[n - 2].get_or_init(|| {
        let count = 1usize << (2 * n);
        let mut distance = vec![u8::MAX; count];
        let mut previous = vec![(0, 0, 0, 0); count];
        distance[0] = 0;
        let mut queue = VecDeque::from([0]);
        while let Some(state) = queue.pop_front() {
            for first in 0..n {
                for last in first..n {
                    if first == 0 && last + 1 == n {
                        continue;
                    }
                    for q in 1..=3 {
                        let mut next = state;
                        for layer in first..=last {
                            let shift = 2 * layer;
                            let digit = ((state >> shift) + q) & 3;
                            next = (next & !(3 << shift)) | (digit << shift);
                        }
                        if distance[next] == u8::MAX {
                            distance[next] = distance[state] + 1;
                            previous[next] = (state, first, last, q as u8);
                            queue.push_back(next);
                        }
                    }
                }
            }
        }
        BlockTable { distance, previous }
    })
}

fn encode(net: &[u8], rotation: u8) -> usize {
    net.iter().enumerate().fold(0, |state, (layer, &q)| {
        state | (usize::from((q + 4 - rotation) % 4) << (2 * layer))
    })
}

/// Keep an omitted rotation as a face permutation and conjugate all later
/// moves through it. The table finds the fewest contiguous blocks after
/// subtracting each possible uniform rotation from the axis.
fn easy(kind: PuzzleKind, solution: &MoveSequence) -> Vec<LayerMove> {
    let n = kind.size();
    let mut inverse = Face::ALL;
    let mut output = Vec::new();
    let moves = solution.moves();
    let mut start = 0;
    while start < moves.len() {
        let axis = axis_of(moves[start].face);
        let mut end = start + 1;
        while end < moves.len() && axis_of(moves[end].face) == axis {
            end += 1;
        }
        let run: Vec<Move> = moves[start..end]
            .iter()
            .map(|mv| {
                let mut transformed = *mv;
                transformed.face = inverse[mv.face.index()];
                transformed
            })
            .collect();
        let transformed_axis = axis_of(run[0].face);
        let mut net = vec![0; n];
        for mv in &run {
            add(&mut net, mv);
        }
        let table = block_table(n);
        let mut best: Option<(usize, u8)> = None;
        let mut best_len = run.len();
        for q in 0..4 {
            let state = encode(&net, q);
            let length = usize::from(table.distance[state]);
            if length < best_len {
                best_len = length;
                best = Some((state, q));
            }
        }
        if let Some((mut state, q)) = best {
            let mut found = Vec::new();
            while state != 0 {
                let (previous, first, last, turns) = table.previous[state];
                found.push(block_move(n, transformed_axis, (first, last, turns)));
                state = previous;
            }
            found.reverse();
            output.extend(found);
            if q != 0 {
                let rotation = inverse_rotation(kind, Face::ALL[transformed_axis], q);
                inverse = inverse.map(|face| rotation[face.index()]);
            }
        } else {
            output.extend(run.into_iter().map(LayerMove::from));
        }
        start = end;
    }
    output
}

/// A solution compressed and written: the text and its move count.
pub fn compressed_text(
    notation: Notation,
    kind: PuzzleKind,
    solution: &MoveSequence,
) -> (String, usize) {
    let packed = compress(kind, solution);
    (write(notation, kind, &packed), packed.len())
}

#[cfg(test)]
mod tests;
