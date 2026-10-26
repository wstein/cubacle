//! Axis normalization: the shortest way to write what a run of moves does.
//!
//! All turns about one axis commute (`R`, `Rw`, `2R`, `L` and `Lw` turn
//! layers of the same stack), so a run of them is described by the net
//! quarter turns of each layer, a number 0 to 3 per layer. Every move counts
//! one, whatever its angle (the metric of
//! [Performance](../../docs/performance.md)), so the shortest run is the
//! fewest moves whose layers add up to that description. It is found by a
//! breadth-first search over the descriptions (4^n of them, at most 4^7) from
//! the empty run; where the run is already as short, it is kept as it is.
//!
//! The result is exactly equivalent to the input: the same state, never a
//! rotation of it. Runs that become empty let their neighbors on the same
//! axis merge, so the pass repeats until nothing changes.

use super::{Face, Move, Turn};

/// A move that can be used for a run, relative to an axis: which layers (from
/// the positive face U, R or F) it turns, and on which face of the axis.
struct Generator {
    layers: Vec<usize>,
    negative: bool,
    depth: u8,
    wide: bool,
}

fn axis_of(face: Face) -> usize {
    face.index() % 3
}

fn is_negative(face: Face) -> bool {
    face.index() >= 3
}

fn quarter(turn: Turn) -> u8 {
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

/// The layers a move turns, counted from the positive face, and by how many
/// quarter turns as seen from that face; `None` if it does not fit the cube.
fn layers_of(m: &Move, n: usize) -> Option<(Vec<usize>, u8)> {
    let d = usize::from(m.depth);
    if d == 0 || d > n {
        return None;
    }
    let negative = is_negative(m.face);
    let layers: Vec<usize> = match (m.wide, negative) {
        (false, false) => vec![d - 1],
        (false, true) => vec![n - d],
        (true, false) => (0..d).collect(),
        (true, true) => (n - d..n).collect(),
    };
    let q = quarter(m.turn);
    Some((layers, if negative { (4 - q) % 4 } else { q }))
}

/// The moves a run may use. `slices` allows every single layer; otherwise the
/// outer layers, on a 5x5x5 or larger the second layer, and on a 6x6x6 or
/// larger also the third, as the engines do.
fn generators(n: usize, slices: bool) -> Vec<Generator> {
    let mut list = Vec::new();
    let single = |layer: usize| {
        let negative = layer > n - 1 - layer;
        Generator {
            layers: vec![layer],
            negative,
            depth: (if negative { n - layer } else { layer + 1 }) as u8,
            wide: false,
        }
    };
    let layer_set: Vec<usize> = if slices {
        (0..n).collect()
    } else {
        let mut v = vec![0, n - 1];
        if n >= 5 {
            v.extend([1, n - 2]);
        }
        if n >= 6 {
            v.extend([2, n - 3]);
        }
        v.sort_unstable();
        v
    };
    for layer in layer_set {
        list.push(single(layer));
    }
    // Two-layer turns; on a 6x6x6 and larger also the wider blocks (up to
    // half the cube, or three layers without `slices`).
    let widest = if n < 6 {
        2
    } else if slices {
        n / 2
    } else {
        3
    };
    for depth in 2..=widest.min(n - 1) {
        list.push(Generator {
            layers: (0..depth).collect(),
            negative: false,
            depth: depth as u8,
            wide: true,
        });
        list.push(Generator {
            layers: (n - depth..n).collect(),
            negative: true,
            depth: depth as u8,
            wide: true,
        });
    }
    list
}

fn add(state: usize, n: usize, layers: &[usize], quarters: u8) -> usize {
    let mut next = state;
    for &l in layers {
        let digit = (state >> (2 * l)) & 3;
        let sum = (digit + usize::from(quarters)) & 3;
        next = (next & !(3 << (2 * l))) | (sum << (2 * l));
    }
    debug_assert!(next < 1 << (2 * n));
    next
}

/// For every description, the fewest moves and how the search reached it.
struct Table {
    dist: Vec<u8>,
    /// `(previous state, generator, quarters)`
    from: Vec<(u32, u8, u8)>,
}

fn search(n: usize, gens: &[Generator]) -> Table {
    let states = 1usize << (2 * n);
    let mut dist = vec![u8::MAX; states];
    let mut from = vec![(0u32, 0u8, 0u8); states];
    dist[0] = 0;
    let mut queue = std::collections::VecDeque::from([0usize]);
    while let Some(s) = queue.pop_front() {
        for (g, gen) in gens.iter().enumerate() {
            for q in 1..4u8 {
                let t = add(s, n, &gen.layers, q);
                if dist[t] == u8::MAX {
                    dist[t] = dist[s] + 1;
                    from[t] = (s as u32, g as u8, q);
                    queue.push_back(t);
                }
            }
        }
    }
    Table { dist, from }
}

/// The run `moves` (all on `axis`) written with the fewest moves, or `None`
/// if it is already as short or cannot be written with the generators.
fn shortest(
    n: usize,
    axis: usize,
    moves: &[Move],
    gens: &[Generator],
    table: &Table,
) -> Option<Vec<Move>> {
    let mut state = 0;
    for m in moves {
        let (layers, q) = layers_of(m, n)?;
        state = add(state, n, &layers, q);
    }
    let d = table.dist[state];
    if usize::from(d) >= moves.len() {
        return None;
    }
    let mut parts = Vec::new();
    let mut s = state;
    while s != 0 {
        let (prev, g, q) = table.from[s];
        parts.push((g, q));
        s = prev as usize;
    }
    parts.sort_unstable();
    Some(
        parts
            .into_iter()
            .map(|(g, q)| {
                let gen = &gens[usize::from(g)];
                let face = Face::ALL[axis + if gen.negative { 3 } else { 0 }];
                let quarters = if gen.negative { (4 - q) % 4 } else { q };
                Move::new(face, gen.depth, gen.wide, turn_of(quarters))
            })
            .collect(),
    )
}

/// `moves` rewritten with the fewest moves per axis run, equivalent to them
/// exactly. With `slices` false the result uses only the moves of the engines
/// (outer layers, second layers on 5x5x5 and larger, two-layer turns); with
/// `slices` true it may also turn any single layer (`2R`). Returned unchanged
/// if some move does not fit a cube of edge `n`.
pub(crate) fn normalize(n: usize, moves: &[Move], slices: bool) -> Vec<Move> {
    if n < 2 || moves.iter().any(|m| layers_of(m, n).is_none()) {
        return moves.to_vec();
    }
    let gens = generators(n, slices);
    let mut table: Option<Table> = None;
    let mut current = moves.to_vec();
    loop {
        let mut out: Vec<Move> = Vec::with_capacity(current.len());
        let mut changed = false;
        let mut i = 0;
        while i < current.len() {
            let axis = axis_of(current[i].face);
            let mut j = i;
            while j < current.len() && axis_of(current[j].face) == axis {
                j += 1;
            }
            let run = &current[i..j];
            let table = table.get_or_insert_with(|| search(n, &gens));
            match shortest(n, axis, run, &gens, table) {
                Some(better) => {
                    out.extend(better);
                    changed = true;
                }
                None => out.extend_from_slice(run),
            }
            i = j;
        }
        current = out;
        if !changed {
            return current;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

    fn seq(text: &str) -> Vec<Move> {
        text.parse::<MoveSequence>().unwrap().moves().to_vec()
    }

    fn show(moves: &[Move]) -> String {
        MoveSequence::new(moves.to_vec()).to_string()
    }

    #[test]
    fn opposite_faces_cancel_and_merge() {
        assert_eq!(show(&normalize(4, &seq("R L R' L'"), false)), "");
        assert_eq!(show(&normalize(4, &seq("R U Rw Rw' U'"), false)), "R");
        // Lw turns two layers, L' takes one back: two moves are the least.
        assert_eq!(normalize(4, &seq("R Lw L' R'"), false).len(), 2);
        assert_eq!(show(&normalize(4, &seq("R Lw L' R'"), true)), "2L");
    }

    #[test]
    fn a_slice_needs_the_slices_option() {
        // R Rw' turns the second layer back: no outer or wide pair is shorter.
        assert_eq!(normalize(4, &seq("R Rw'"), false), seq("R Rw'"));
        assert_eq!(show(&normalize(4, &seq("R Rw'"), true)), "2R'");
    }

    #[test]
    fn nothing_is_changed_when_nothing_is_shorter() {
        let s = seq("R U R' U' Rw Lw' F2");
        assert_eq!(normalize(4, &s, false), s);
    }

    #[test]
    fn the_result_is_exactly_the_same_state() {
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = |bound: u64| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) % bound
        };
        for (kind, n) in [
            (PuzzleKind::Cube4, 4),
            (PuzzleKind::Cube5, 5),
            (PuzzleKind::Cube6, 6),
            (PuzzleKind::Cube7, 7),
        ] {
            let scramble: Vec<Move> = (0..40).map(|_| move_at(&mut next, n, true)).collect();
            let base = Facelets::solved(kind)
                .apply(&MoveSequence::new(scramble))
                .unwrap();
            for slices in [false, true] {
                let mut shortened = 0;
                for _ in 0..300 {
                    let len = 2 + next(12) as usize;
                    let moves: Vec<Move> = (0..len).map(|_| move_at(&mut next, n, false)).collect();
                    let result = normalize(n, &moves, slices);
                    assert!(result.len() <= moves.len());
                    shortened += usize::from(result.len() < moves.len());
                    let want = base.apply(&MoveSequence::new(moves.clone())).unwrap();
                    let got = base.apply(&MoveSequence::new(result.clone())).unwrap();
                    assert_eq!(want, got, "{} -> {}", show(&moves), show(&result));
                    if !slices {
                        let allowed = |m: &Move| {
                            (m.depth == 1 && !m.wide)
                                || (m.depth == 2 && (m.wide || n >= 5))
                                || (n >= 6 && m.depth == 3)
                        };
                        assert!(result.iter().all(allowed), "{}", show(&result));
                    }
                }
                assert!(shortened > 0);
            }
        }
    }

    /// A random move of the engines' set (`all`: with the middle layers).
    fn move_at(next: &mut impl FnMut(u64) -> u64, n: usize, all: bool) -> Move {
        let face = Face::ALL[next(6) as usize];
        let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][next(3) as usize];
        match next(4) {
            0 => Move::new(face, 1, false, turn),
            1 => Move::new(face, 2, true, turn),
            3 if n >= 6 => Move::new(face, 3, n.is_multiple_of(2), turn),
            3 => Move::new(face, 1, false, turn),
            _ if n >= 5 || all => Move::new(face, 2, false, turn),
            _ => Move::new(face, 1, false, turn),
        }
    }
}
