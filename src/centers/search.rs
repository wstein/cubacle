//! A shortest move sequence that makes the centers of a cube uniform: every
//! face shows one color in each center orbit.
//!
//! The centers of one orbit (24 stickers, four on each face) never mix with
//! those of another, and a color has four stickers in each orbit. So a state
//! of one color in one orbit is a set of four of the 24 places, and a table
//! of its distance to "all four on one face" is tiny. An iterative deepening
//! search over layer moves uses the largest such distance as its bound.

use std::collections::VecDeque;

use crate::puzzle::{
    center_orbits, destinations, Face, Facelets, Move, MoveSequence, PuzzleKind, Turn,
};

/// Places in one orbit.
const CELLS: usize = 24;
/// Sets of four places out of 24.
const SETS: usize = 10_626;

type Binomials = [[usize; 5]; 25];

/// The moves the search uses, as permutations of every orbit's places.
pub(crate) struct Model {
    kind: PuzzleKind,
    orbits: Vec<Vec<usize>>,
    moves: Vec<Move>,
    axis: Vec<u8>,
    /// For every move and orbit: the place a sticker of the orbit goes to.
    perms: Vec<Vec<[u8; CELLS]>>,
    /// Per orbit, the distance of a set of four places to the four places of
    /// the face of color `i` (tables 0 to 5) or to those of any face (6).
    tables: Vec<Vec<Vec<u8>>>,
}

fn binomials() -> Binomials {
    let mut table = [[0usize; 5]; 25];
    for n in 0..25 {
        table[n][0] = 1;
        for k in 1..5 {
            if n > 0 {
                table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
            }
        }
    }
    table
}

fn rank(binom: &Binomials, mut mask: u32) -> usize {
    let mut at = 0;
    let mut total = 0;
    while mask != 0 {
        let place = mask.trailing_zeros() as usize;
        at += 1;
        total += binom[place][at];
        mask &= mask - 1;
    }
    total
}

fn turn_back(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    }
}

/// Every layer move that can help to sort centers, each layer set once. The
/// middle layer of an odd cube is left out: it would turn the face centers
/// (the frame of the cube) and the engines keep that frame.
fn alphabet(kind: PuzzleKind) -> Vec<Move> {
    let n = kind.size();
    let middle = (n % 2 == 1).then_some(n.div_ceil(2));
    let mut out = Vec::new();
    for face in Face::ALL {
        let positive = face.index() < 3;
        for depth in 1..n {
            let single = positive || depth == 1;
            let wide = depth >= 2 && depth <= n / 2;
            for (is_wide, allowed) in [(false, single), (true, wide)] {
                if !allowed {
                    continue;
                }
                let layer = if positive { depth } else { n + 1 - depth };
                if !is_wide && middle == Some(layer) {
                    continue;
                }
                for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
                    out.push(Move::new(face, depth as u8, is_wide, turn));
                }
            }
        }
    }
    out
}

impl Model {
    pub(crate) fn new(kind: PuzzleKind) -> Model {
        let orbits = center_orbits(kind);
        let mut slot = vec![None; kind.facelet_count()];
        for (o, orbit) in orbits.iter().enumerate() {
            for (i, &sticker) in orbit.iter().enumerate() {
                slot[sticker] = Some((o, i));
            }
        }
        let moves = alphabet(kind);
        let axis = moves.iter().map(|m| (m.face.index() % 3) as u8).collect();
        let perms: Vec<Vec<[u8; CELLS]>> = moves
            .iter()
            .map(|mv| {
                let to = destinations(kind, mv);
                orbits
                    .iter()
                    .map(|orbit| {
                        let mut perm = [0u8; CELLS];
                        for (i, &sticker) in orbit.iter().enumerate() {
                            let (_, place) = slot[to[sticker]]
                                .expect("a layer turn keeps centers in their orbit");
                            perm[i] = place as u8;
                        }
                        perm
                    })
                    .collect()
            })
            .collect();
        let binom = binomials();
        let tables = (0..orbits.len())
            .map(|o| {
                let face_mask = |face: usize| {
                    orbits[o]
                        .iter()
                        .enumerate()
                        .filter(|&(_, &s)| s / kind.face_len() == face)
                        .fold(0u32, |m, (i, _)| m | 1 << i)
                };
                (0..7)
                    .map(|t| {
                        let goals: Vec<usize> = if t < 6 { vec![t] } else { (0..6).collect() };
                        let mut table = vec![u8::MAX; SETS];
                        let mut queue = VecDeque::new();
                        for face in goals {
                            let mask = face_mask(face);
                            table[rank(&binom, mask)] = 0;
                            queue.push_back((mask, 0u8));
                        }
                        while let Some((mask, dist)) = queue.pop_front() {
                            for perm in perms.iter().map(|p| &p[o]) {
                                let next = permute(perm, mask);
                                let at = rank(&binom, next);
                                if table[at] == u8::MAX {
                                    table[at] = dist + 1;
                                    queue.push_back((next, dist + 1));
                                }
                            }
                        }
                        table
                    })
                    .collect()
            })
            .collect();
        Model {
            kind,
            orbits,
            moves,
            axis,
            perms,
            tables,
        }
    }

    /// The color sets of a state: per orbit and color, the places it fills.
    fn masks(&self, state: &Facelets) -> Vec<[u32; 6]> {
        self.orbits
            .iter()
            .map(|orbit| {
                let mut masks = [0u32; 6];
                for (i, &sticker) in orbit.iter().enumerate() {
                    masks[state.stickers()[sticker].index()] |= 1 << i;
                }
                masks
            })
            .collect()
    }

    /// Whether every color fills four places of every orbit, as on a cube.
    pub(crate) fn balanced(&self, state: &Facelets) -> bool {
        self.masks(state)
            .iter()
            .all(|colors| colors.iter().all(|mask| mask.count_ones() == 4))
    }

    fn bound(&self, binom: &Binomials, masks: &[[u32; 6]], own: bool) -> usize {
        let mut worst = 0;
        for (o, colors) in masks.iter().enumerate() {
            for (color, &mask) in colors.iter().enumerate() {
                let table = if own { color } else { 6 };
                worst = worst.max(self.tables[o][table][rank(binom, mask)]);
            }
        }
        usize::from(worst)
    }

    /// The fewest moves that make the centers of `state` uniform, if there
    /// are at most `deepest` of them and the search needs at most `nodes`
    /// steps; `None` otherwise.
    pub(crate) fn shortest(
        &self,
        state: &Facelets,
        own: bool,
        deepest: usize,
        nodes: u64,
    ) -> Option<MoveSequence> {
        debug_assert_eq!(state.kind(), self.kind);
        let binom = binomials();
        let start = self.masks(state);
        let mut budget = nodes;
        for limit in self.bound(&binom, &start, own)..=deepest {
            let mut walk = Walk {
                binom: &binom,
                own,
                budget,
                path: Vec::new(),
            };
            let outcome = self.descend(&mut walk, &start, limit, usize::MAX);
            budget = walk.budget;
            match outcome {
                Some(true) => {
                    return Some(MoveSequence::new(
                        walk.path.into_iter().map(|m| self.moves[m]).collect(),
                    ))
                }
                Some(false) => {}
                None => return None,
            }
        }
        None
    }

    /// A short, not proven shortest, sequence: a beam search that keeps the
    /// `width` best states of every length, ranked by the summed table
    /// distances of the colors, and stops at `deepest` moves.
    pub(crate) fn beam(
        &self,
        state: &Facelets,
        own: bool,
        width: usize,
        deepest: usize,
    ) -> Option<MoveSequence> {
        let binom = binomials();
        let score = |masks: &[[u32; 6]]| -> (usize, usize) {
            let mut sum = 0;
            let mut worst = 0;
            for (o, colors) in masks.iter().enumerate() {
                for (color, &mask) in colors.iter().enumerate() {
                    let table = if own { color } else { 6 };
                    let d = usize::from(self.tables[o][table][rank(&binom, mask)]);
                    sum += d;
                    worst = worst.max(d);
                }
            }
            (sum, worst)
        };
        // (score, state, parent index in the previous level, move)
        type Node = ((usize, usize), Vec<[u32; 6]>, usize, usize);
        let first = self.masks(state);
        let mut levels: Vec<Vec<Node>> = vec![vec![(score(&first), first, usize::MAX, usize::MAX)]];
        let mut seen = std::collections::HashSet::new();
        for _ in 0..deepest {
            let last = levels.last().expect("a level");
            let mut next: Vec<Node> = Vec::new();
            for (parent, node) in last.iter().enumerate() {
                for m in 0..self.moves.len() {
                    let masks: Vec<[u32; 6]> = node
                        .1
                        .iter()
                        .enumerate()
                        .map(|(o, colors)| colors.map(|mask| permute(&self.perms[m][o], mask)))
                        .collect();
                    next.push((score(&masks), masks, parent, m));
                }
            }
            next.sort_by_key(|node| node.0);
            let mut kept: Vec<Node> = Vec::with_capacity(width);
            for node in next {
                if kept.len() == width {
                    break;
                }
                if seen.insert(node.1.clone()) {
                    kept.push(node);
                }
            }
            let next = kept;
            if let Some(done) = next.iter().position(|node| node.0 .0 == 0) {
                let mut path = vec![next[done].3];
                let mut at = next[done].2;
                for level in levels.iter().skip(1).rev() {
                    path.push(level[at].3);
                    at = level[at].2;
                }
                path.reverse();
                return Some(MoveSequence::new(
                    path.into_iter().map(|m| self.moves[m]).collect(),
                ));
            }
            if next.is_empty() {
                return None;
            }
            levels.push(next);
        }
        None
    }

    /// `Some(true)` when a solution of at most `left` more moves was put on
    /// `path`, `Some(false)` when there is none, `None` when the budget ended.
    fn descend(
        &self,
        walk: &mut Walk,
        masks: &[[u32; 6]],
        left: usize,
        last: usize,
    ) -> Option<bool> {
        let bound = self.bound(walk.binom, masks, walk.own);
        if bound == 0 {
            return Some(true);
        }
        if bound > left {
            return Some(false);
        }
        for m in 0..self.moves.len() {
            // moves about one axis commute: try them in index order only
            if last != usize::MAX && self.axis[last] == self.axis[m] && m <= last {
                continue;
            }
            if walk.budget == 0 {
                return None;
            }
            walk.budget -= 1;
            let next: Vec<[u32; 6]> = masks
                .iter()
                .enumerate()
                .map(|(o, colors)| colors.map(|mask| permute(&self.perms[m][o], mask)))
                .collect();
            walk.path.push(m);
            match self.descend(walk, &next, left - 1, m) {
                Some(false) => {
                    walk.path.pop();
                }
                other => return other,
            }
        }
        Some(false)
    }
}

/// The state of one deepening pass of the exact search.
struct Walk<'a> {
    binom: &'a Binomials,
    own: bool,
    budget: u64,
    path: Vec<usize>,
}

fn permute(perm: &[u8; CELLS], mut mask: u32) -> u32 {
    let mut out = 0;
    while mask != 0 {
        out |= 1 << perm[mask.trailing_zeros() as usize];
        mask &= mask - 1;
    }
    out
}

/// The moves that undo `moves`.
pub(crate) fn inverse(moves: &MoveSequence) -> MoveSequence {
    moves
        .moves()
        .iter()
        .rev()
        .map(|m| Move::new(m.face, m.depth, m.wide, turn_back(m.turn)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
        Facelets::solved(kind)
            .apply(&moves.parse().unwrap())
            .unwrap()
    }

    #[test]
    fn a_table_covers_every_set_of_four_places() {
        let model = Model::new(PuzzleKind::Cube4);
        assert!(model.tables[0][6].iter().all(|&d| d != u8::MAX));
    }

    #[test]
    fn it_undoes_a_few_slice_turns_in_as_many_moves() {
        for kind in [PuzzleKind::Cube4, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let model = Model::new(kind);
            let state = cube(kind, "2R 2U'");
            let found = model.shortest(&state, false, 4, 5_000_000).expect("found");
            assert!(found.len() <= 2, "{kind:?} {found}");
            let sorted = state.apply(&found).unwrap();
            assert!(super::super::uniform(&sorted), "{kind:?}");
        }
    }

    #[test]
    fn the_inverse_undoes_the_sequence() {
        let seq: MoveSequence = "2R U' 3Rw2 F".parse().unwrap();
        let solved = Facelets::solved(PuzzleKind::Cube5);
        let there = solved.apply(&seq).unwrap();
        assert!(there.apply(&inverse(&seq)).unwrap().is_solved());
    }
}
