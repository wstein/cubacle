//! The order of a move sequence on the sticker level.
//!
//! A sequence moves every sticker to some other place. Repeating it returns
//! the cube exactly when every sticker is back, so the order is the least
//! common multiple of the cycle lengths of that permutation. No piece model
//! is needed, which makes the definition the same on every size.

use super::apply::{destinations, locate};
use super::{ApplyError, MoveSequence, PuzzleKind};

/// The kind of place a sticker sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickerOrbit {
    Corners,
    /// The edge stickers of the 3x3x3 and the middle edges of odd cubes.
    Edges,
    /// The edge stickers that are not in the middle of an edge (4x4x4 and up).
    Wings,
    Centers,
}

impl StickerOrbit {
    const ALL: [StickerOrbit; 4] = [Self::Corners, Self::Edges, Self::Wings, Self::Centers];

    pub fn name(self) -> &'static str {
        match self {
            Self::Corners => "corners",
            Self::Edges => "edges",
            Self::Wings => "wings",
            Self::Centers => "centers",
        }
    }
}

/// The cycle lengths of the stickers of one orbit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrbitCycles {
    pub orbit: StickerOrbit,
    /// Number of stickers in the orbit.
    pub stickers: usize,
    /// Lengths of the cycles longer than one, ascending.
    pub lengths: Vec<usize>,
}

impl OrbitCycles {
    /// The least common multiple of this orbit's cycle lengths.
    pub fn order(&self) -> u128 {
        self.lengths
            .iter()
            .fold(1, |acc, &len| lcm(acc, len as u128))
    }
}

/// The cycle structure of a sequence and its order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StickerCycles {
    /// Only the orbits the cube has.
    pub orbits: Vec<OrbitCycles>,
}

impl StickerCycles {
    /// How many repetitions bring the cube back (1 for the identity).
    pub fn order(&self) -> u128 {
        self.orbits
            .iter()
            .fold(1, |acc, orbit| lcm(acc, orbit.order()))
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

fn orbit_of(kind: PuzzleKind, index: usize) -> usize {
    let m = kind.size() as i32 - 1;
    let (position, _) = locate(kind, index);
    let extreme = position.iter().filter(|c| c.abs() == m).count();
    let orbit = match extreme {
        3 => StickerOrbit::Corners,
        2 if kind.size() <= 3 || position.contains(&0) => StickerOrbit::Edges,
        2 => StickerOrbit::Wings,
        _ => StickerOrbit::Centers,
    };
    StickerOrbit::ALL
        .iter()
        .position(|&o| o == orbit)
        .expect("every orbit is listed")
}

/// The sticker cycles of `moves` on a cube of `kind`.
///
/// Fails if a move turns a layer the cube does not have.
pub fn sticker_cycles(kind: PuzzleKind, moves: &MoveSequence) -> Result<StickerCycles, ApplyError> {
    if let Some(&mv) = moves.moves().iter().find(|mv| !mv.fits(kind)) {
        return Err(ApplyError { mv, kind });
    }
    let count = kind.facelet_count();
    // at[i] is the original sticker now sitting at place i
    let mut at: Vec<usize> = (0..count).collect();
    for mv in moves.moves() {
        let mut next = at.clone();
        for (from, &to) in destinations(kind, mv).iter().enumerate() {
            next[to] = at[from];
        }
        at = next;
    }
    let mut seen = vec![false; count];
    let mut orbits: Vec<OrbitCycles> = StickerOrbit::ALL
        .iter()
        .map(|&orbit| OrbitCycles {
            orbit,
            stickers: 0,
            lengths: Vec::new(),
        })
        .collect();
    for start in 0..count {
        let slot = orbit_of(kind, start);
        orbits[slot].stickers += 1;
        if seen[start] {
            continue;
        }
        let (mut length, mut place) = (0, start);
        while !seen[place] {
            seen[place] = true;
            place = at[place];
            length += 1;
        }
        if length > 1 {
            orbits[slot].lengths.push(length);
        }
    }
    orbits.retain(|orbit| orbit.stickers > 0);
    for orbit in &mut orbits {
        orbit.lengths.sort_unstable();
    }
    Ok(StickerCycles { orbits })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(kind: PuzzleKind, text: &str) -> u128 {
        let moves: MoveSequence = text.parse().unwrap();
        sticker_cycles(kind, &moves).unwrap().order()
    }

    #[test]
    fn known_orders() {
        assert_eq!(order(PuzzleKind::Cube3, ""), 1);
        assert_eq!(order(PuzzleKind::Cube3, "R"), 4);
        assert_eq!(order(PuzzleKind::Cube3, "R2"), 2);
        assert_eq!(order(PuzzleKind::Cube3, "R U"), 105);
    }

    #[test]
    fn a_missing_layer_is_an_error() {
        let moves: MoveSequence = "3R".parse().unwrap();
        assert!(sticker_cycles(PuzzleKind::Cube3, &moves).is_err());
    }
}
