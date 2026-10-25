//! Measures of regularity beyond patternness, the input of the pattern kinds.
//!
//! Patternness asks how symmetric and uniform a state is as a whole. These
//! measures look at it from other sides, each cheap and each answering one
//! question:
//!
//! - **defect mask**: turn the whole cube so that as many stickers as possible
//!   have the colour of the solved cube; the stickers that do not are the
//!   *defect*. `fixed` is the share outside it, `mask_symmetry` how many of the
//!   48 symmetries map the defect onto itself (as `ln count / ln 48`), and
//!   `moved` the number of pieces it touches.
//! - **faces**: the share of the six faces that show the same arrangement as
//!   the best-represented one (up to turning, mirroring and renaming colours),
//!   and the share of *simple* faces (one colour, stripes, a chequer, a cross,
//!   rings, ...).
//! - **symmetry** and **orbit** uniformity, the two parts of patternness.
//!
//! They are measures of regularity, not proofs; `kind.rs` turns them into a
//! class.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::colours;
use super::fingerprint::{fingerprint, OrbitStat};
use super::geometry::{symmetries, tables};
use super::patternness::{symmetry_score, uniformity_score};
use super::symmetry::{symmetry_of, Symmetry};
use crate::puzzle::{locate, Facelets, PuzzleKind};

/// The measures of one state.
#[derive(Debug, Clone, PartialEq)]
pub struct Measures {
    /// The cube size.
    pub size: usize,
    /// Patternness, 0 to 100.
    pub patternness: u8,
    /// Symmetry part of patternness, 0 to 1.
    pub symmetry: f64,
    /// How many of the 48 symmetries fix the state (up to renaming colours).
    pub symmetry_order: usize,
    /// Orbit uniformity part of patternness, 0 to 1.
    pub orbit: f64,
    /// Share of stickers outside the defect of the best whole-cube turn.
    pub fixed: f64,
    /// Symmetry of the defect: `ln(symmetries that keep it) / ln 48`, 0 to 1.
    pub mask_symmetry: f64,
    /// Pieces the defect touches.
    pub moved: usize,
    /// All pieces of the cube (corners, edge pieces and centre stickers).
    pub pieces: usize,
    /// Share of faces with the most common arrangement, 0 to 1.
    pub faces: f64,
    /// Share of simple faces, 0 to 1.
    pub simple_faces: f64,
}

impl Measures {
    /// Share of pieces the defect touches.
    pub fn moved_share(&self) -> f64 {
        self.moved as f64 / self.pieces as f64
    }
}

/// Everything about a size that the measures need.
struct Geometry {
    /// The 24 rotations: indices into the 48 symmetries.
    rotations: Vec<usize>,
    /// The piece (cubie) of each sticker, numbered from 0.
    piece: Vec<usize>,
    pieces: usize,
}

fn geometry(kind: PuzzleKind) -> &'static Geometry {
    static CELLS: [OnceLock<Geometry>; 6] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    CELLS[kind.size() - 2].get_or_init(|| {
        let rotations = (0..48).filter(|&g| !symmetries()[g].is_mirror()).collect();
        let mut ids = HashMap::new();
        let piece = (0..kind.facelet_count())
            .map(|i| {
                let next = ids.len();
                *ids.entry(locate(kind, i).0).or_insert(next)
            })
            .collect();
        Geometry {
            rotations,
            piece,
            pieces: ids.len(),
        }
    })
}

/// The measures of a cube.
pub fn measures(cube: &Facelets) -> Measures {
    let kind = cube.kind();
    let state = colours(cube);
    let symmetry = symmetry_of(kind, &state);
    let orbits = fingerprint(kind, &state).orbits;
    measures_of(kind, &state, &symmetry, &orbits)
}

pub(crate) fn measures_of(
    kind: PuzzleKind,
    state: &[u8],
    symmetry: &Symmetry,
    orbits: &[OrbitStat],
) -> Measures {
    let geo = geometry(kind);
    let mask = defect(kind, geo, state);
    let total = state.len();
    let defective = mask.iter().filter(|&&d| d).count();
    let kept = tables(kind)
        .perm
        .iter()
        .filter(|perm| (0..total).all(|p| mask[perm[p] as usize] == mask[p]))
        .count();
    let mut touched = vec![false; geo.pieces];
    for (p, _) in mask.iter().enumerate().filter(|(_, &d)| d) {
        touched[geo.piece[p]] = true;
    }
    let (faces, simple_faces) = face_measures(kind, state);
    Measures {
        size: kind.size(),
        patternness: super::patternness::patternness(symmetry, orbits),
        symmetry: symmetry_score(symmetry),
        symmetry_order: symmetry.order(),
        orbit: uniformity_score(orbits),
        fixed: (total - defective) as f64 / total as f64,
        mask_symmetry: (kept as f64).ln() / 48f64.ln(),
        moved: touched.iter().filter(|&&t| t).count(),
        pieces: geo.pieces,
        faces,
        simple_faces,
    }
}

/// The stickers whose colour differs from the solved cube under the whole-cube
/// turn that matches the most stickers.
fn defect(kind: PuzzleKind, geo: &Geometry, state: &[u8]) -> Vec<bool> {
    let face_len = kind.face_len();
    let perm = &tables(kind).perm;
    let colour_of_face = |g: usize| -> [u8; 6] {
        std::array::from_fn(|f| (perm[g][f * face_len] as usize / face_len) as u8)
    };
    let matching = |g: usize| {
        let to = colour_of_face(g);
        (0..state.len())
            .filter(|&p| state[p] == to[p / face_len])
            .count()
    };
    let best = geo
        .rotations
        .iter()
        .copied()
        .max_by_key(|&g| (matching(g), std::cmp::Reverse(g)))
        .unwrap_or(0);
    let to = colour_of_face(best);
    (0..state.len())
        .map(|p| state[p] != to[p / face_len])
        .collect()
}

/// A face rotated or mirrored by one of the 8 symmetries of the square.
fn square_image(n: usize, t: usize, r: usize, c: usize) -> (usize, usize) {
    let (r, c) = if t & 4 != 0 { (c, r) } else { (r, c) };
    let r = if t & 1 != 0 { n - 1 - r } else { r };
    let c = if t & 2 != 0 { n - 1 - c } else { c };
    (r, c)
}

/// Colours renamed 0, 1, 2, ... in order of first appearance.
fn normalised(face: &[u8]) -> Vec<u8> {
    let mut seen = [u8::MAX; 6];
    let mut next = 0;
    face.iter()
        .map(|&c| {
            if seen[c as usize] == u8::MAX {
                seen[c as usize] = next;
                next += 1;
            }
            seen[c as usize]
        })
        .collect()
}

/// Cells of equal class in the partitions of a simple face.
type Partition = fn(usize, usize, usize) -> i32;

fn distance(n: usize, i: usize) -> i32 {
    (2 * i as i32 - (n as i32 - 1)).abs()
}

const PARTITIONS: [Partition; 8] = [
    |_, _, _| 0,
    |_, _, c| c as i32,
    |_, r, _| r as i32,
    |_, r, c| ((r + c) % 2) as i32,
    |n, r, c| distance(n, r).max(distance(n, c)),
    |n, r, c| (distance(n, r).min(distance(n, c)) == 0) as i32,
    |n, r, c| (r == c || r + c == n - 1) as i32,
    |n, r, c| {
        let (x, y) = (distance(n, r), distance(n, c));
        x.max(y) * 1000 + x.min(y)
    },
];

/// Whether a face is one colour per class of a partition with at most half as
/// many classes as cells.
fn is_simple(n: usize, face: &[u8]) -> bool {
    PARTITIONS.iter().any(|part| {
        let mut colour_of: HashMap<i32, u8> = HashMap::new();
        let consistent = (0..n * n)
            .all(|i| *colour_of.entry(part(n, i / n, i % n)).or_insert(face[i]) == face[i]);
        consistent && (colour_of.len() <= n * n / 2 || colour_of.len() == 1)
    })
}

/// The share of faces in the largest class of equal arrangements, and the
/// share of simple faces.
fn face_measures(kind: PuzzleKind, state: &[u8]) -> (f64, f64) {
    let n = kind.size();
    let len = kind.face_len();
    let faces: Vec<&[u8]> = state.chunks(len).collect();
    let forms: Vec<Vec<Vec<u8>>> = faces
        .iter()
        .map(|face| {
            (0..8)
                .map(|t| {
                    let mut turned = vec![0u8; len];
                    for r in 0..n {
                        for c in 0..n {
                            let (r2, c2) = square_image(n, t, r, c);
                            turned[r2 * n + c2] = face[r * n + c];
                        }
                    }
                    normalised(&turned)
                })
                .collect()
        })
        .collect();
    let same = (0..6)
        .map(|i| (0..6).filter(|&j| forms[i].contains(&forms[j][0])).count())
        .max()
        .unwrap_or(0);
    let simple = faces.iter().filter(|f| is_simple(n, f)).count();
    (same as f64 / 6.0, simple as f64 / 6.0)
}
