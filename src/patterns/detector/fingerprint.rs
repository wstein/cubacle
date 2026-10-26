//! The fingerprint of a state: for every piece orbit how many pieces are out
//! of place, how many are twisted or flipped, and (where the pieces can be
//! told apart) the cycle type of their permutation.
//!
//! Orientation depends on which axis is looked at first, and "home" on the
//! colour frame, so the numbers are read in every frame and axis order that
//! fits and the least result is kept. That makes the fingerprint the same for
//! a cube, its turned or mirrored image and any renaming of its colours.

use super::center_names::{centre_classes, pair_up};
use super::pieces::{frames, Cube, Frame, ORDERS};
use crate::puzzle::PuzzleKind;

/// The counts of one piece orbit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrbitStat {
    /// `corners`, `edges`, `wings` (with their depth) or `centres`.
    pub name: String,
    /// How many pieces (centre stickers for `centres`).
    pub pieces: usize,
    /// How many are not in a place with their own faces.
    pub displaced: usize,
    /// How many are twisted (corners) or flipped (edges); 0 for centres.
    pub oriented_wrong: usize,
    /// The lengths of the cycles longer than one, longest first, when the
    /// pieces of the orbit are all different and all found; `None` otherwise.
    pub cycles: Option<Vec<usize>>,
    /// The token prefix of a centre orbit (`X`, `+`, `O1-2:`); `None` for
    /// corners, edges and wings.
    pub tag: Option<String>,
}

impl OrbitStat {
    /// Whether the orbit is solved: nothing displaced, twisted or flipped.
    pub fn is_solved(&self) -> bool {
        self.displaced == 0 && self.oriented_wrong == 0
    }

    /// The compact token: `C0t0`, `E0f12`, `W1:0f12`, `X12`.
    fn token(&self) -> String {
        if let Some(tag) = &self.tag {
            return format!("{tag}{}", self.displaced);
        }
        let (tag, depth) = match self.name.split_once(' ') {
            Some((word, depth)) => (word, format!("{depth}:")),
            None => (self.name.as_str(), String::new()),
        };
        let letter = match tag {
            "corners" => "C",
            "edges" => "E",
            "wings" => "W",
            _ => "?",
        };
        let wrong = match tag {
            "corners" => format!("t{}", self.oriented_wrong),
            "edges" | "wings" => format!("f{}", self.oriented_wrong),
            _ => String::new(),
        };
        let cycles = match &self.cycles {
            Some(c) if !c.is_empty() => format!(
                "({})",
                c.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            ),
            _ => String::new(),
        };
        format!("{letter}{depth}{}{wrong}{cycles}", self.displaced)
    }
}

/// The per-orbit counts of a state, and the string that stands for them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    pub orbits: Vec<OrbitStat>,
}

impl Fingerprint {
    /// A compact string, equal for equivalent states: one token per orbit,
    /// such as `C0t0 E0f12`.
    pub fn code(&self) -> String {
        let tokens: Vec<String> = self.orbits.iter().map(OrbitStat::token).collect();
        tokens.join(" ")
    }
}

/// The cycle lengths longer than one of a permutation given as targets.
fn cycle_type(target: &[usize]) -> Option<Vec<usize>> {
    let mut seen = vec![false; target.len()];
    let mut cycles = Vec::new();
    for start in 0..target.len() {
        let mut length = 0;
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            at = target[at];
            length += 1;
            if at >= target.len() {
                return None;
            }
        }
        if length > 1 {
            cycles.push(length);
        }
    }
    cycles.sort_unstable_by(|a, b| b.cmp(a));
    Some(cycles)
}

/// The slot-to-slot permutation of a set of distinct pieces, when every piece
/// is valid and every place is the home of exactly one.
fn permutation(cube: &Cube, places: &[Vec<usize>], order: [usize; 3]) -> Option<Vec<usize>> {
    let homes: Vec<u8> = places.iter().map(|p| cube.faces_of(p)).collect();
    let mut target = Vec::with_capacity(places.len());
    for place in places {
        let reading = cube.read(place, order);
        if !reading.valid {
            return None;
        }
        target.push(homes.iter().position(|&h| h == reading.home)?);
    }
    let mut sorted = target.clone();
    sorted.sort_unstable();
    sorted.dedup();
    (sorted.len() == places.len()).then_some(target)
}

fn read(kind: PuzzleKind, cube: &Cube, order: [usize; 3], odd: bool) -> Vec<OrbitStat> {
    let layout = cube.layout;
    let mut out = Vec::new();

    let corners: Vec<Vec<usize>> = layout.corners.iter().map(|c| c.to_vec()).collect();
    let readings: Vec<_> = corners.iter().map(|c| cube.read(c, order)).collect();
    out.push(OrbitStat {
        name: "corners".to_owned(),
        pieces: corners.len(),
        displaced: readings.iter().filter(|r| !r.placed).count(),
        oriented_wrong: readings.iter().filter(|r| !r.oriented).count(),
        cycles: permutation(cube, &corners, order).and_then(|t| cycle_type(&t)),
        tag: None,
    });

    let depths: Vec<usize> = {
        let mut d: Vec<usize> = layout.wings.iter().map(|w| w.depth).collect();
        d.sort_unstable();
        d.dedup();
        d
    };
    for depth in depths {
        let wings: Vec<&super::pieces::Wing> =
            layout.wings.iter().filter(|w| w.depth == depth).collect();
        let places: Vec<Vec<usize>> = wings.iter().map(|w| w.stickers.to_vec()).collect();
        let readings: Vec<_> = places.iter().map(|p| cube.read(p, order)).collect();
        let single = odd && wings.iter().all(|w| w.middle);
        out.push(OrbitStat {
            name: if single {
                "edges".to_owned()
            } else {
                format!("wings {depth}")
            },
            pieces: places.len(),
            displaced: readings.iter().filter(|r| !r.placed).count(),
            oriented_wrong: readings.iter().filter(|r| !r.oriented).count(),
            cycles: single
                .then(|| permutation(cube, &places, order))
                .flatten()
                .and_then(|t| cycle_type(&t)),
            tag: None,
        });
    }

    let classes = centre_classes(kind);
    let mut centres: Vec<(usize, OrbitStat)> = layout
        .centres
        .iter()
        .zip(&classes)
        .map(|(orbit, class)| {
            let rank = classes.iter().position(|c| c.name == class.name);
            let stat = OrbitStat {
                name: class.name.clone(),
                pieces: orbit.len(),
                displaced: orbit
                    .iter()
                    .filter(|&&s| cube.lab[s] as usize != s / layout.face_len)
                    .count(),
                oriented_wrong: 0,
                cycles: None,
                tag: Some(class.tag.clone()),
            };
            (rank.unwrap_or(0), stat)
        })
        .collect();
    // Mirrors swap the two chiral orbits of a class, so only the multiset counts.
    centres.sort();
    let mut centres: Vec<OrbitStat> = centres.into_iter().map(|(_, s)| s).collect();
    pair_up(centres.iter_mut().map(|s| &mut s.name));
    out.extend(centres);
    out
}

/// The fingerprint of a state (colours 0 to 5 in `URFDLB` order).
pub(crate) fn fingerprint(kind: PuzzleKind, state: &[u8]) -> Fingerprint {
    let odd = kind.size() % 2 == 1;
    let frames: Vec<Frame> = frames(kind, state);
    let mut best: Option<Vec<OrbitStat>> = None;
    for frame in &frames {
        let cube = Cube::new(kind, state, frame);
        for order in ORDERS {
            let stats = read(kind, &cube, order, odd);
            if best.as_ref().is_none_or(|b| stats < *b) {
                best = Some(stats);
            }
        }
    }
    Fingerprint {
        orbits: best.unwrap_or_default(),
    }
}
