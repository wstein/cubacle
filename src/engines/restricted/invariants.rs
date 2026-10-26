//! Telling that a cube cannot be solved with a set of moves, without search.
//!
//! The moves of the alphabet only ever carry a piece to places in its orbit,
//! and they change a few sums by fixed amounts: the parity of the pieces of
//! an orbit, the twist of the corners of an orbit (mod 3) and the flip of
//! its edges (mod 2). Those sums add up under composition, so every cube the
//! alphabet can build has a sum vector in the span of the vectors of its
//! letters. A cube whose pieces are in other orbits, or whose sums are not in
//! the span, is out of reach. The converse does not hold (a cube that passes
//! may still be out of reach), which is why this is the quick check and the
//! table and the search remain the proof.
//!
//! Pieces are read from the sticker colors: a cubie is the set of colors at
//! a place, its twist the place of its reference color (the U or D color,
//! else the F or B color) among the stickers read in turn around the cubie.

use std::collections::HashMap;

use super::letters::{apply, Letters, State, N};
use crate::puzzle::{locate, PuzzleKind};

type Vec3 = [i32; 3];

/// One cubie place: its stickers read around it, the first being the
/// reference sticker.
struct Place {
    stickers: Vec<usize>,
}

/// The places of a 3x3x3: 8 corners, 12 edges, 6 centers.
struct Places {
    corners: Vec<Place>,
    edges: Vec<Place>,
    centers: Vec<Place>,
    /// The solved colors of each place, in sticker order.
    colors: Vec<Vec<u8>>,
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: Vec3, b: Vec3) -> i32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Places {
    fn new() -> Places {
        let kind = PuzzleKind::Cube3;
        let mut by_position: HashMap<Vec3, Vec<(usize, Vec3)>> = HashMap::new();
        for index in 0..N {
            let (position, normal) = locate(kind, index);
            by_position
                .entry(position)
                .or_default()
                .push((index, normal));
        }
        let mut positions: Vec<Vec3> = by_position.keys().copied().collect();
        positions.sort_unstable();
        let (mut corners, mut edges, mut centers) = (Vec::new(), Vec::new(), Vec::new());
        for position in positions {
            let mut group = by_position[&position].clone();
            // The reference sticker first: on U or D, else on F or B.
            let rank = |normal: &Vec3| -> usize {
                if normal[1] != 0 {
                    0
                } else if normal[2] != 0 {
                    1
                } else {
                    2
                }
            };
            group.sort_by_key(|(_, normal)| rank(normal));
            if group.len() == 3 {
                // Right-handed turning order around the corner.
                let (a, b, c) = (group[0].1, group[1].1, group[2].1);
                if dot(a, cross(b, c)) < 0 {
                    group.swap(1, 2);
                }
            }
            let place = Place {
                stickers: group.iter().map(|(index, _)| *index).collect(),
            };
            match place.stickers.len() {
                3 => corners.push(place),
                2 => edges.push(place),
                _ => centers.push(place),
            }
        }
        let colors = corners
            .iter()
            .chain(edges.iter())
            .chain(centers.iter())
            .map(|place| place.stickers.iter().map(|&i| (i / 9) as u8).collect())
            .collect();
        Places {
            corners,
            edges,
            centers,
            colors,
        }
    }

    fn all(&self) -> impl Iterator<Item = &Place> {
        self.corners
            .iter()
            .chain(self.edges.iter())
            .chain(self.centers.iter())
    }
}

/// A cube read as pieces: at each place (corners, then edges, then centers)
/// the place the piece started at, and its turn about its reference sticker.
#[derive(Clone, PartialEq, Eq)]
struct Pieces {
    at: Vec<u8>,
    turn: Vec<u8>,
}

fn decode(places: &Places, state: &State) -> Result<Pieces, String> {
    let mut at = Vec::new();
    let mut turn = Vec::new();
    for (place_index, place) in places.all().enumerate() {
        let here: Vec<u8> = place.stickers.iter().map(|&i| state[i]).collect();
        let mut sorted = here.clone();
        sorted.sort_unstable();
        let found = places.colors.iter().position(|home| {
            let mut other = home.clone();
            other.sort_unstable();
            other == sorted
        });
        let Some(home) = found else {
            return Err("a piece shows colors no piece has".to_owned());
        };
        // Pieces only change place within their kind.
        let kind_of = |i: usize| match i {
            0..=7 => 0,
            8..=19 => 1,
            _ => 2,
        };
        if kind_of(home) != kind_of(place_index) {
            return Err("a piece is of another kind than its place".to_owned());
        }
        let reference = places.colors[home][0];
        let shift = here
            .iter()
            .position(|&c| c == reference)
            .ok_or_else(|| "a piece lacks its reference color".to_owned())?;
        let k = here.len();
        for j in 0..k {
            if here[(shift + j) % k] != places.colors[home][j] {
                return Err("a corner is mirrored: its colors turn the wrong way".to_owned());
            }
        }
        at.push(home as u8);
        turn.push(shift as u8);
    }
    Ok(Pieces { at, turn })
}

/// Why a cube is out of reach.
pub(super) enum Reach {
    /// The text of the colors is no cube.
    NoCube(String),
    /// A cube, but not one these moves can solve.
    Outside(String),
}

/// An orbit of places under the letters.
struct Orbit {
    members: Vec<usize>,
    /// 0 corner, 1 edge, 2 center
    kind: u8,
}

pub(super) struct Invariants {
    places: Places,
    orbits: Vec<Orbit>,
    /// Basis of the span of the letters' sums over GF(2) and GF(3).
    basis2: Vec<u64>,
    basis3: Vec<Vec<u8>>,
    /// Which orbit each place belongs to.
    orbit_of: Vec<usize>,
}

fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

fn parity(members: &[usize], at: &[u8]) -> u8 {
    let mut seen = vec![false; members.len()];
    let index_of = |p: usize| members.iter().position(|&m| m == p).unwrap_or(0);
    let mut cycles = 0;
    for start in 0..members.len() {
        if seen[start] {
            continue;
        }
        cycles += 1;
        let mut here = start;
        while !seen[here] {
            seen[here] = true;
            here = index_of(usize::from(at[members[here]]));
        }
    }
    ((members.len() - cycles) % 2) as u8
}

impl Invariants {
    pub(super) fn new(letters: &Letters, solved: &State) -> Invariants {
        let places = Places::new();
        let count = places.colors.len();
        // Orbits: places joined by what a single letter does to the solved cube.
        let mut parent: Vec<usize> = (0..count).collect();
        let mut moved: Vec<Pieces> = Vec::new();
        let mut out = [0u8; N];
        for letter in &letters.letters {
            apply(&letter.src, solved, &mut out);
            let pieces = decode(&places, &out).expect("a move keeps the cube a cube");
            for (place, &home) in pieces.at.iter().enumerate() {
                let (a, b) = (
                    find(&mut parent, place),
                    find(&mut parent, usize::from(home)),
                );
                parent[a] = b;
            }
            moved.push(pieces);
        }
        let mut roots: Vec<usize> = Vec::new();
        let mut orbit_of = vec![0; count];
        let mut orbits: Vec<Orbit> = Vec::new();
        for (place, slot) in orbit_of.iter_mut().enumerate() {
            let root = find(&mut parent, place);
            let index = match roots.iter().position(|&r| r == root) {
                Some(i) => i,
                None => {
                    roots.push(root);
                    orbits.push(Orbit {
                        members: Vec::new(),
                        kind: match place {
                            0..=7 => 0,
                            8..=19 => 1,
                            _ => 2,
                        },
                    });
                    roots.len() - 1
                }
            };
            orbits[index].members.push(place);
            *slot = index;
        }
        let mut invariants = Invariants {
            places,
            orbits,
            basis2: Vec::new(),
            basis3: Vec::new(),
            orbit_of,
        };
        for pieces in &moved {
            let (two, three) = invariants
                .sums(pieces)
                .expect("a letter keeps every piece in its orbit");
            invariants.add_to_basis(two, three);
        }
        invariants
    }

    /// The sums of a cube: GF(2) coordinates as bits (the parity of every
    /// orbit, then the flip of every edge orbit) and GF(3) coordinates (the
    /// twist of every corner orbit). `None` when a piece is out of its orbit.
    fn sums(&self, pieces: &Pieces) -> Option<(u64, Vec<u8>)> {
        let mut two = 0u64;
        let mut three = Vec::new();
        for (index, orbit) in self.orbits.iter().enumerate() {
            if orbit
                .members
                .iter()
                .any(|&p| self.orbit_of[usize::from(pieces.at[p])] != index)
            {
                return None;
            }
            let bit = |n: usize| 1u64 << n;
            if parity(&orbit.members, &pieces.at) == 1 {
                two |= bit(index);
            }
            let total: u32 = orbit
                .members
                .iter()
                .map(|&p| u32::from(pieces.turn[p]))
                .sum();
            match orbit.kind {
                0 => three.push((total % 3) as u8),
                1 if total % 2 == 1 => two |= bit(self.orbits.len() + index),
                _ => {}
            }
        }
        Some((two, three))
    }

    fn add_to_basis(&mut self, mut two: u64, mut three: Vec<u8>) {
        for &b in &self.basis2 {
            // A basis vector clears its highest bit from the new vector.
            if two & (1 << (63 - b.leading_zeros())) != 0 {
                two ^= b;
            }
        }
        if two != 0 {
            self.basis2.push(two);
        }
        for b in &self.basis3 {
            if let Some(lead) = b.iter().position(|&x| x != 0) {
                let factor = (three[lead] * b[lead]) % 3; // b[lead] is 1 or 2, its own inverse mod 3
                for (t, &x) in three.iter_mut().zip(b.iter()) {
                    *t = (*t + 3 * 3 - factor * x) % 3;
                }
            }
        }
        if three.iter().any(|&x| x != 0) {
            self.basis3.push(three);
        }
    }

    fn in_span(&self, mut two: u64, mut three: Vec<u8>) -> bool {
        for &b in &self.basis2 {
            if two & (1 << (63 - b.leading_zeros())) != 0 {
                two ^= b;
            }
        }
        if two != 0 {
            return false;
        }
        for b in &self.basis3 {
            if let Some(lead) = b.iter().position(|&x| x != 0) {
                let factor = (three[lead] * b[lead]) % 3;
                for (t, &x) in three.iter_mut().zip(b.iter()) {
                    *t = (*t + 3 * 3 - factor * x) % 3;
                }
            }
        }
        three.iter().all(|&x| x == 0)
    }

    /// Whether the cube may be solvable with the alphabet.
    pub(super) fn check(&self, state: &State) -> Result<(), Reach> {
        let pieces = decode(&self.places, state).map_err(Reach::NoCube)?;
        let Some((two, three)) = self.sums(&pieces) else {
            return Err(Reach::Outside(
                "a piece sits in a place these moves never bring it to".to_owned(),
            ));
        };
        if self.in_span(two, three) {
            Ok(())
        } else {
            Err(Reach::Outside(
                "its twist, flip or parity is not one these moves can make".to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_alphabet;
    use crate::puzzle::{Facelets, MoveSequence};

    fn state_of(moves: &str) -> State {
        let sequence: MoveSequence = moves.parse().unwrap();
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&sequence)
            .unwrap();
        let mut state = [0u8; N];
        for (slot, face) in state.iter_mut().zip(cube.stickers()) {
            *slot = *face as u8;
        }
        state
    }

    fn invariants(set: &str) -> Invariants {
        let alphabet = parse_alphabet(set, PuzzleKind::Cube3).unwrap();
        let letters = Letters::new(&alphabet).unwrap();
        Invariants::new(&letters, &state_of(""))
    }

    #[test]
    fn the_sums_of_the_whole_cube_group_are_what_the_rules_of_the_cube_say() {
        // With all face turns, every reachable cube has twist 0, flip 0 and
        // equal parities; check it on random walks.
        let all = invariants("<U,D,R,L,F,B>");
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut moves = Vec::new();
        for _ in 0..200 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let face = ["U", "R", "F", "D", "L", "B"][((seed >> 33) % 6) as usize];
            let turn = ["", "2", "'"][((seed >> 40) % 3) as usize];
            moves.push(format!("{face}{turn}"));
            assert!(all.check(&state_of(&moves.join(" "))).is_ok());
        }
    }

    #[test]
    fn no_face_turn_changes_a_twist_or_flip_sum() {
        let all = invariants("<U,D,R,L,F,B>");
        assert!(all.basis3.is_empty(), "twist sums are always 0 mod 3");
        let parities = (1u64 << all.orbits.len()) - 1;
        assert!(
            all.basis2.iter().all(|b| b & !parities == 0),
            "flip sums are always even"
        );
        assert_eq!(all.orbits.len(), 8, "corners, edges and six fixed centers");
    }

    #[test]
    fn a_flipped_edge_is_out_of_reach_of_every_face_turn() {
        let all = invariants("<U,D,R,L,F,B>");
        let places = Places::new();
        let edge = &places.edges[0];
        let mut state = state_of("");
        state.swap(edge.stickers[0], edge.stickers[1]);
        assert!(matches!(all.check(&state), Err(Reach::Outside(_))));
    }

    #[test]
    fn a_twisted_corner_is_out_of_reach_of_every_face_turn() {
        let all = invariants("<U,D,R,L,F,B>");
        // One corner twisted in place: swap two stickers of a corner by hand.
        let mut state = state_of("");
        let places = Places::new();
        let corner = &places.corners[0];
        let (a, b, c) = (corner.stickers[0], corner.stickers[1], corner.stickers[2]);
        let saved = state[a];
        state[a] = state[b];
        state[b] = state[c];
        state[c] = saved;
        assert!(matches!(all.check(&state), Err(Reach::Outside(_))));
    }

    #[test]
    fn a_quarter_turn_is_outside_the_half_turn_group() {
        let half = invariants("half-turns");
        assert!(half.check(&state_of("R2 U2 F2 D2")).is_ok());
        assert!(matches!(half.check(&state_of("R")), Err(Reach::Outside(_))));
        assert!(matches!(
            half.check(&state_of("R U2 R'")),
            Err(Reach::Outside(_)) | Ok(())
        ));
    }

    #[test]
    fn a_piece_outside_its_orbit_is_found() {
        let ru = invariants("<R,U>");
        assert!(ru.check(&state_of("R U R' U' R U2")).is_ok());
        // F moves the piece on the DLF corner... which <R,U> never moves.
        assert!(matches!(ru.check(&state_of("L")), Err(Reach::Outside(_))));
        assert!(matches!(ru.check(&state_of("F")), Err(Reach::Outside(_))));
    }

    #[test]
    fn the_slice_group_moves_centers_and_edges_only() {
        let slices = invariants("quarter-slices");
        assert!(slices.check(&state_of("2R 2U 2F'")).is_ok());
        assert!(matches!(
            slices.check(&state_of("R")),
            Err(Reach::Outside(_))
        ));
    }

    #[test]
    fn colors_that_are_no_cube_are_named() {
        let all = invariants("<R,U>");
        let mut state = state_of("");
        state.swap(0, 9);
        assert!(matches!(all.check(&state), Err(Reach::NoCube(_))));
    }
}
