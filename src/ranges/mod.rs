//! Post-solve compression: adjacent turns of one axis written as layer ranges.
//!
//! All turns about one axis commute, so a run of them is described by the net
//! quarter turns of each layer. Neighbouring layers with the same net turn are
//! one [`LayerMove`] (`3Rw Rw'` is layer 3 alone; `2R 3R` is `2-3r` in SiGN,
//! `2-3Rw` in WCA). No search is involved, and a run is only replaced when the
//! replacement is shorter. The result is exactly the same state, never a
//! rotation of it. Only contiguous blocks are formed.

use crate::notation::Notation;
use crate::puzzle::{Face, LayerMove, Move, MoveSequence, PuzzleKind, Turn};

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
fn compress_run(n: usize, run: &[Move], out: &mut Vec<LayerMove>) {
    let axis = axis_of(run[0].face);
    let mut net = vec![0u8; n];
    for mv in run {
        add(&mut net, mv);
    }
    let found = blocks(&net);
    // Every layer turned alike is a rotation of the cube: keep the run.
    let rotation = found.len() == 1 && found[0].1 - found[0].0 + 1 == n;
    if !rotation && found.len() < run.len() {
        out.extend(found.into_iter().map(|b| block_move(n, axis, b)));
    } else {
        out.extend(run.iter().copied().map(LayerMove::from));
    }
}

/// `moves` with each run of turns about one axis replaced by layer ranges
/// where that is shorter. Replaying the result gives the same state.
pub fn compress(kind: PuzzleKind, moves: &MoveSequence) -> Vec<LayerMove> {
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
        compress_run(n, &all[start..end], &mut out);
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

/// The opt-in post-solve optimisations (`--optimise`, the `optimise` request field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Optimise {
    /// Write adjacent same-axis turns as layer ranges.
    Ranges,
}

impl std::str::FromStr for Optimise {
    type Err = String;

    fn from_str(text: &str) -> Result<Optimise, String> {
        match text {
            "ranges" => Ok(Optimise::Ranges),
            _ => Err(format!("optimise must be ranges, not {text:?}")),
        }
    }
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
