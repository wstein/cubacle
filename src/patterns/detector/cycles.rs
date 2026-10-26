//! A state as part cycles: for each piece orbit the cycles in which the
//! pieces have moved, with an orientation mark for each piece, and the order
//! of the whole permutation.
//!
//! The spelling is our own. A slot is named by the faces that meet there, in
//! the order U/D, F/B, R/L: `UFR` for a corner, `UF` for an edge, `UF2` for
//! the second wing of that edge on a bigger cube (counted towards +x, +y or
//! +z, that is from L to R, from D to U and from B to F), `Ur2c3` for the
//! centre sticker in row 2, column 3 of the U face in the usual net. The
//! cycle `(UFR UBR DBR)` means: the corner that belongs at UFR sits at UBR,
//! the one that belongs at UBR sits at DBR, and the one that belongs at DBR
//! sits at UFR. The marks after a cycle say how each of those pieces is turned,
//! in the same order. A corner mark is the number of clockwise thirds (seen
//! from outside the cube) from the slot's U/D face to the face that shows the
//! piece's own U/D colour: `0`, `+1`, `+2`. An edge or wing mark is `0` if the
//! piece shows its first-named colour (U/D before F/B before R/L) on the
//! slot's first-named face, `+1` if it is flipped.

use std::fmt;

use super::center_names::{centre_classes, pair_up};
use super::matching::{centre_groups, Permutation};
use super::pieces::{axis, layout};
use crate::puzzle::{locate, Face, Facelets, PuzzleKind};

/// The order of a permutation: how many repetitions bring the cube back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    /// Every piece back in place and orientation.
    pub whole: u128,
    /// The same with the centres left out.
    pub without_centres: u128,
}

/// One cycle of pieces, listed by slot, with the mark of each piece.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    pub slots: Vec<String>,
    /// Empty when the orbit has no orientation (centres).
    pub marks: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Corner,
    Wing,
    Centre,
}

/// A piece orbit: its cycles, leaving out the pieces in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrbitCycles {
    pub name: String,
    pub pieces: usize,
    /// Pieces that sit at home, unturned (not listed in `cycles`).
    pub in_place: usize,
    pub cycles: Vec<Cycle>,
    class: Class,
    /// (home slot, current slot, mark) of every piece.
    moves: Vec<(usize, usize, u8)>,
}

/// A state as cycles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycles {
    pub orbits: Vec<OrbitCycles>,
    /// `None` when the pieces were read from colours and equal pieces had to
    /// be matched arbitrarily, so no order is defined.
    pub order: Option<Order>,
    kind: PuzzleKind,
}

fn rank(face: usize) -> usize {
    match axis(face) {
        0 => 0,
        2 => 1,
        _ => 2,
    }
}

/// The face letters of a slot's stickers, U/D then F/B then R/L.
fn letters(stickers: &[usize], face_len: usize) -> String {
    let mut faces: Vec<usize> = stickers.iter().map(|s| s / face_len).collect();
    faces.sort_by_key(|&f| rank(f));
    faces.into_iter().map(|f| Face::ALL[f].letter()).collect()
}

/// The stickers of a corner: the U/D one first, then clockwise from outside.
fn ring(kind: PuzzleKind, corner: [usize; 3]) -> [usize; 3] {
    let face_len = kind.face_len();
    let mut s = corner;
    s.sort_by_key(|&x| rank(x / face_len));
    let n: Vec<[i32; 3]> = s.iter().map(|&x| locate(kind, x).1).collect();
    let cross = [
        n[1][1] * n[2][2] - n[1][2] * n[2][1],
        n[1][2] * n[2][0] - n[1][0] * n[2][2],
        n[1][0] * n[2][1] - n[1][1] * n[2][0],
    ];
    let det: i32 = (0..3).map(|i| n[0][i] * cross[i]).sum();
    if det < 0 {
        s
    } else {
        [s[0], s[2], s[1]]
    }
}

/// The reference sticker of a wing and the other one.
fn wing_pair(face_len: usize, stickers: [usize; 2]) -> [usize; 2] {
    if rank(stickers[0] / face_len) <= rank(stickers[1] / face_len) {
        stickers
    } else {
        [stickers[1], stickers[0]]
    }
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn lcm(a: u128, b: u128) -> u128 {
    a / gcd(a, b) * b
}

/// The cycle form of a permutation.
pub(crate) fn analyse(kind: PuzzleKind, pi: &Permutation, exact: bool) -> Cycles {
    let layout = layout(kind);
    let face_len = layout.face_len;
    let n = kind.size();
    let mut corner_of = vec![usize::MAX; kind.facelet_count()];
    for (i, c) in layout.corners.iter().enumerate() {
        for &s in c {
            corner_of[s] = i;
        }
    }
    let mut wing_of = vec![usize::MAX; kind.facelet_count()];
    for (i, w) in layout.wings.iter().enumerate() {
        for s in w.stickers {
            wing_of[s] = i;
        }
    }
    let mut orbits = Vec::new();

    let corner_name = |i: usize| letters(&layout.corners[i], face_len);
    let rings: Vec<[usize; 3]> = layout.corners.iter().map(|&c| ring(kind, c)).collect();
    let pieces: Vec<(usize, usize, u8)> = (0..layout.corners.len())
        .map(|h| {
            let image = pi[rings[h][0]];
            let slot = corner_of[image];
            let mark = rings[slot].iter().position(|&s| s == image).unwrap_or(0);
            (h, slot, mark as u8)
        })
        .collect();
    orbits.push(build(
        "corners".to_owned(),
        Class::Corner,
        pieces,
        &corner_name,
    ));

    let mut depths: Vec<usize> = layout.wings.iter().map(|w| w.depth).collect();
    depths.sort_unstable();
    depths.dedup();
    let wing_name = |i: usize| {
        let w = &layout.wings[i];
        let edge = letters(&w.stickers, face_len);
        if n > 3 {
            format!("{edge}{}", w.along)
        } else {
            edge
        }
    };
    for depth in depths {
        let members: Vec<usize> = (0..layout.wings.len())
            .filter(|&i| layout.wings[i].depth == depth)
            .collect();
        let pieces: Vec<(usize, usize, u8)> = members
            .iter()
            .map(|&h| {
                let image = pi[wing_pair(face_len, layout.wings[h].stickers)[0]];
                let slot = wing_of[image];
                let flipped = image != wing_pair(face_len, layout.wings[slot].stickers)[0];
                (h, slot, u8::from(flipped))
            })
            .collect();
        let name = if layout.wings[members[0]].middle {
            "edges".to_owned()
        } else {
            format!("wings {depth}")
        };
        orbits.push(build(name, Class::Wing, pieces, &wing_name));
    }

    let centre_name = |s: usize| {
        let row = (s % face_len) / n;
        format!(
            "{}r{}c{}",
            Face::ALL[s / face_len].letter(),
            row + 1,
            s % n + 1
        )
    };
    let groups = centre_groups(kind);
    let mut names: Vec<String> = centre_classes(kind).into_iter().map(|c| c.name).collect();
    pair_up(names.iter_mut());
    for (g, group) in groups.iter().enumerate() {
        let name = match names.get(g) {
            Some(name) => name.clone(),
            None if n == 3 => "centres".to_owned(),
            None => "centres (face middles)".to_owned(),
        };
        let pieces = group.iter().map(|&s| (s, pi[s], 0)).collect();
        orbits.push(build(name, Class::Centre, pieces, &centre_name));
    }

    let order = exact.then(|| order_of(&orbits));
    Cycles {
        orbits,
        order,
        kind,
    }
}

fn build(
    name: String,
    class: Class,
    moves: Vec<(usize, usize, u8)>,
    slot_name: &dyn Fn(usize) -> String,
) -> OrbitCycles {
    let mut target = std::collections::HashMap::new();
    let mut mark = std::collections::HashMap::new();
    for &(home, slot, m) in &moves {
        target.insert(home, slot);
        mark.insert(home, m);
    }
    let mut homes: Vec<usize> = moves.iter().map(|m| m.0).collect();
    homes.sort_unstable();
    let mut seen = std::collections::HashSet::new();
    let (mut cycles, mut in_place) = (Vec::new(), 0);
    for &start in &homes {
        if seen.contains(&start) {
            continue;
        }
        let mut slots = vec![start];
        seen.insert(start);
        let mut at = target[&start];
        while at != start {
            slots.push(at);
            seen.insert(at);
            at = target[&at];
        }
        let marks: Vec<u8> = slots.iter().map(|s| mark[s]).collect();
        if slots.len() == 1 && marks[0] == 0 {
            in_place += 1;
            continue;
        }
        cycles.push(Cycle {
            slots: slots.iter().map(|&s| slot_name(s)).collect(),
            marks: if class == Class::Centre {
                Vec::new()
            } else {
                marks
            },
        });
    }
    OrbitCycles {
        name,
        pieces: moves.len(),
        in_place,
        cycles,
        class,
        moves,
    }
}

fn modulus(class: Class) -> u32 {
    match class {
        Class::Corner => 3,
        Class::Wing => 2,
        Class::Centre => 0,
    }
}

/// The order from the cycle type: each cycle of length `L` whose marks do not
/// sum to a multiple of the modulus takes `L` times the modulus.
fn order_of(orbits: &[OrbitCycles]) -> Order {
    let (mut whole, mut without) = (1u128, 1u128);
    for orbit in orbits {
        let m = modulus(orbit.class);
        let mut result = 1u128;
        for cycle in &orbit.cycles {
            let sum: u32 = cycle.marks.iter().map(|&x| u32::from(x)).sum();
            let turns = if m != 0 && !sum.is_multiple_of(m) {
                m
            } else {
                1
            };
            result = lcm(result, cycle.slots.len() as u128 * u128::from(turns));
        }
        whole = lcm(whole, result);
        if orbit.class != Class::Centre {
            without = lcm(without, result);
        }
    }
    Order {
        whole,
        without_centres: without,
    }
}

impl Cycles {
    /// The cube these cycles make from the solved cube, built from the cycles
    /// alone.
    pub fn state(&self) -> Facelets {
        let kind = self.kind;
        let layout = layout(kind);
        let face_len = layout.face_len;
        let mut pi = vec![0usize; kind.facelet_count()];
        for orbit in &self.orbits {
            for &(home, slot, mark) in &orbit.moves {
                match orbit.class {
                    Class::Corner => {
                        let (from, to) = (
                            ring(kind, layout.corners[home]),
                            ring(kind, layout.corners[slot]),
                        );
                        for j in 0..3 {
                            pi[from[j]] = to[(j + mark as usize) % 3];
                        }
                    }
                    Class::Wing => {
                        let from = wing_pair(face_len, layout.wings[home].stickers);
                        let to = wing_pair(face_len, layout.wings[slot].stickers);
                        let flip = usize::from(mark);
                        pi[from[0]] = to[flip];
                        pi[from[1]] = to[1 - flip];
                    }
                    Class::Centre => pi[home] = slot,
                }
            }
        }
        let mut stickers = vec![Face::U; kind.facelet_count()];
        for (i, &to) in pi.iter().enumerate() {
            stickers[to] = Face::ALL[i / face_len];
        }
        Facelets::from_stickers(stickers).expect("a permutation keeps every colour count")
    }
}

fn mark_text(mark: u8) -> String {
    if mark == 0 {
        "0".to_owned()
    } else {
        format!("+{mark}")
    }
}

impl fmt::Display for Cycles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for orbit in &self.orbits {
            if orbit.cycles.is_empty() {
                writeln!(f, "{}: solved", orbit.name)?;
                continue;
            }
            let parts: Vec<String> = orbit
                .cycles
                .iter()
                .map(|c| {
                    let slots = format!("({})", c.slots.join(" "));
                    if c.marks.iter().all(|&m| m == 0) {
                        slots
                    } else {
                        let marks: Vec<String> = c.marks.iter().map(|&m| mark_text(m)).collect();
                        format!("{slots} {}", marks.join(" "))
                    }
                })
                .collect();
            let rest = if orbit.in_place > 0 {
                format!("; {} in place", orbit.in_place)
            } else {
                String::new()
            };
            writeln!(f, "{}: {}{rest}", orbit.name, parts.join("; "))?;
        }
        match self.order {
            Some(order) => {
                writeln!(f, "order: {}", order.whole)?;
                if order.without_centres != order.whole {
                    writeln!(f, "order without the centres: {}", order.without_centres)?;
                }
            }
            None => writeln!(
                f,
                "order: not defined from colours alone (equal pieces are matched in index order); give the moves"
            )?,
        }
        Ok(())
    }
}
