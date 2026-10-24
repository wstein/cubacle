//! A cube read as pieces: the corners, the edge pieces (wings) and the
//! centre stickers of any size, with the colour frame that makes "home"
//! mean something.
//!
//! The pieces are read from sticker colours alone, as the restricted engine
//! does: a cubie is the set of colours at a place and its orientation is the
//! place of its reference colour among its stickers.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::puzzle::{centre_orbits, locate, PuzzleKind};

/// An edge piece place: two stickers.
pub(crate) struct Wing {
    pub(crate) stickers: [usize; 2],
    /// Distance from the nearest corner along the edge, 1 for the wings next
    /// to a corner.
    pub(crate) depth: usize,
    /// Place along the edge, 1 to `n - 2`, counted towards +x, +y or +z.
    pub(crate) along: usize,
    /// The only wing of its edge, on a cube of odd size.
    pub(crate) middle: bool,
}

/// Where the pieces of one size sit.
pub(crate) struct Layout {
    pub(crate) face_len: usize,
    pub(crate) corners: Vec<[usize; 3]>,
    pub(crate) wings: Vec<Wing>,
    /// The orbits of the movable centre stickers.
    pub(crate) centres: Vec<Vec<usize>>,
}

pub(crate) fn layout(kind: PuzzleKind) -> &'static Layout {
    static CELLS: [OnceLock<Layout>; 6] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    CELLS[kind.size() - 2].get_or_init(|| build(kind))
}

fn build(kind: PuzzleKind) -> Layout {
    let n = kind.size() as i32;
    let m = n - 1;
    let mut cubies: BTreeMap<[i32; 3], Vec<usize>> = BTreeMap::new();
    for i in 0..kind.facelet_count() {
        cubies.entry(locate(kind, i).0).or_default().push(i);
    }
    let mut corners = Vec::new();
    let mut wings = Vec::new();
    for (position, stickers) in &cubies {
        let extreme = position.iter().filter(|c| c.abs() == m).count();
        match (extreme, stickers.as_slice()) {
            (3, &[a, b, c]) => corners.push([a, b, c]),
            (2, &[a, b]) => {
                let along = position.iter().find(|c| c.abs() != m).copied().unwrap_or(0);
                let k = ((m + along) / 2) as usize;
                wings.push(Wing {
                    stickers: [a, b],
                    depth: k.min(kind.size() - 1 - k),
                    along: k,
                    middle: n % 2 == 1 && k == kind.size() / 2,
                });
            }
            _ => {}
        }
    }
    Layout {
        face_len: kind.face_len(),
        corners,
        wings,
        centres: centre_orbits(kind),
    }
}

/// The face colours a frame gives to the six colours of a state:
/// `frame[c]` is the face whose home colour is `c`.
pub(crate) type Frame = [u8; 6];

/// The frames in which to read a state: the one the centres fix on an odd
/// cube; otherwise every assignment of colours to faces that matches the
/// most stickers (a tie gives several).
pub(crate) fn frames(kind: PuzzleKind, state: &[u8]) -> Vec<Frame> {
    let n = kind.size();
    let face_len = kind.face_len();
    if n % 2 == 1 {
        let mut frame = [u8::MAX; 6];
        for f in 0..6 {
            frame[state[f * face_len + (n / 2) * n + n / 2] as usize] = f as u8;
        }
        if frame.iter().all(|&f| f != u8::MAX) {
            return vec![frame];
        }
    }
    let mut counts = [[0usize; 6]; 6];
    for (i, &c) in state.iter().enumerate() {
        counts[i / face_len][c as usize] += 1;
    }
    let mut best = 0;
    let mut found: Vec<Frame> = Vec::new();
    let mut frame = [0u8; 6];
    let mut used = [false; 6];
    assign(&counts, 0, 0, &mut used, &mut frame, &mut best, &mut found);
    found
}

fn assign(
    counts: &[[usize; 6]; 6],
    face: usize,
    score: usize,
    used: &mut [bool; 6],
    frame: &mut Frame,
    best: &mut usize,
    found: &mut Vec<Frame>,
) {
    if face == 6 {
        match score.cmp(best) {
            std::cmp::Ordering::Greater => {
                *best = score;
                found.clear();
                found.push(*frame);
            }
            std::cmp::Ordering::Equal => found.push(*frame),
            std::cmp::Ordering::Less => {}
        }
        return;
    }
    for colour in 0..6 {
        if !used[colour] {
            used[colour] = true;
            frame[colour] = face as u8;
            assign(
                counts,
                face + 1,
                score + counts[face][colour],
                used,
                frame,
                best,
                found,
            );
            used[colour] = false;
        }
    }
}

/// The axis (U/D 0, R/L 1, F/B 2) of a face index.
pub(crate) fn axis(face: usize) -> usize {
    face % 3
}

/// The axis priorities: which axis is looked at first, second and third when
/// an orientation is read. Every order is a permutation of `0..3`.
pub(crate) const ORDERS: [[usize; 3]; 6] = [
    [0, 2, 1],
    [0, 1, 2],
    [2, 0, 1],
    [2, 1, 0],
    [1, 0, 2],
    [1, 2, 0],
];

fn rank(order: [usize; 3], axis: usize) -> usize {
    order.iter().position(|&a| a == axis).unwrap_or(3)
}

/// What a place holds, for one piece.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Reading {
    /// The piece's colours are those of a cubie of the cube.
    pub(crate) valid: bool,
    /// The set of faces the piece belongs to, as bits (the place whose
    /// faces they are is its home place).
    pub(crate) home: u8,
    /// Whether the piece sits in a place with the same faces.
    pub(crate) placed: bool,
    /// Whether its reference colour is on the reference axis of the place.
    pub(crate) oriented: bool,
}

/// A state in a colour frame, ready to be read piece by piece.
pub(crate) struct Cube {
    pub(crate) layout: &'static Layout,
    /// The face each sticker's colour belongs to.
    pub(crate) lab: Vec<u8>,
}

impl Cube {
    pub(crate) fn new(kind: PuzzleKind, state: &[u8], frame: &Frame) -> Cube {
        Cube {
            layout: layout(kind),
            lab: state.iter().map(|&c| frame[c as usize]).collect(),
        }
    }

    fn face_of(&self, sticker: usize) -> usize {
        sticker / self.layout.face_len
    }

    /// Reads the piece at the place with these stickers.
    pub(crate) fn read(&self, stickers: &[usize], order: [usize; 3]) -> Reading {
        let colours: Vec<usize> = stickers.iter().map(|&s| self.lab[s] as usize).collect();
        let faces: Vec<usize> = stickers.iter().map(|&s| self.face_of(s)).collect();
        let bits = |list: &[usize]| list.iter().fold(0u8, |acc, &f| acc | 1 << f);
        let (home, here) = (bits(&colours), bits(&faces));
        let mut axes: Vec<usize> = colours.iter().map(|&c| axis(c)).collect();
        axes.sort_unstable();
        axes.dedup();
        let valid = axes.len() == stickers.len();
        let by_rank = |list: &[usize]| {
            list.iter()
                .copied()
                .enumerate()
                .min_by_key(|&(_, f)| rank(order, axis(f)))
                .map(|(k, _)| k)
                .unwrap_or(0)
        };
        let oriented = valid && {
            // The sticker holding the reference colour is on the place's
            // reference axis.
            let holder = by_rank(&colours);
            let reference_axis = axis(faces[by_rank(&faces)]);
            axis(faces[holder]) == reference_axis
        };
        Reading {
            valid,
            home,
            placed: valid && home == here,
            oriented,
        }
    }

    /// The faces of a place as bits.
    pub(crate) fn faces_of(&self, stickers: &[usize]) -> u8 {
        stickers
            .iter()
            .fold(0u8, |acc, &s| acc | 1 << self.face_of(s))
    }
}
