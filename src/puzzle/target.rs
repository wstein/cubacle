//! A target that may leave stickers open: for every sticker position, the set
//! of colors allowed there.
//!
//! A complete target (one color everywhere) is a [`Facelets`] state and keeps
//! the relative-state method. A partial one is a goal set: a state reaches it
//! when every sticker shows an allowed color. The model carries sets of any
//! size; the text form only writes a full set (`X`) or a single color.

use std::fmt;

use super::relative::rotations;
use super::{Face, Facelets, MoveSequence, PuzzleKind};

/// All six colors allowed: the sticker does not matter.
const ANY: u8 = 0b11_1111;

/// Why a target could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetError {
    /// The length is not `6 * n * n` for any supported size.
    BadLength(usize),
    /// A sticker has no color allowed, so no state reaches it.
    EmptySet { position: usize },
    /// A color is required on more stickers than the cube has of it.
    TooMany {
        face: Face,
        wanted: usize,
        have: usize,
    },
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetError::BadLength(len) => {
                write!(f, "{len} stickers is not 6*n*n for a supported cube size")
            }
            TargetError::EmptySet { position } => write!(
                f,
                "sticker {position} allows no color, so the target is contradictory"
            ),
            TargetError::TooMany { face, wanted, have } => write!(
                f,
                "the target needs color {} on {wanted} stickers but a cube has only {have} of it; the target is contradictory",
                face.letter()
            ),
        }
    }
}

impl std::error::Error for TargetError {}

/// A map from sticker position to the set of colors allowed there.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    kind: PuzzleKind,
    allowed: Vec<u8>,
}

impl Target {
    /// A target from one color set per sticker, bit `i` standing for
    /// `Face::ALL[i]`.
    pub fn from_sets(allowed: Vec<u8>) -> Result<Target, TargetError> {
        let kind = PuzzleKind::of_facelet_count(allowed.len())
            .ok_or(TargetError::BadLength(allowed.len()))?;
        if let Some(position) = allowed.iter().position(|&set| set & ANY == 0) {
            return Err(TargetError::EmptySet { position });
        }
        let allowed: Vec<u8> = allowed.into_iter().map(|set| set & ANY).collect();
        let have = kind.face_len();
        for face in Face::ALL {
            let wanted = allowed
                .iter()
                .filter(|&&set| set == 1 << face.index())
                .count();
            if wanted > have {
                return Err(TargetError::TooMany { face, wanted, have });
            }
        }
        Ok(Target { kind, allowed })
    }

    /// The target that is exactly `state`.
    pub fn complete(state: &Facelets) -> Target {
        Target {
            kind: state.kind(),
            allowed: state.stickers().iter().map(|f| 1 << f.index()).collect(),
        }
    }

    /// `state` with the stickers where `keep` is false left open.
    pub fn masked(state: &Facelets, keep: impl Fn(usize) -> bool) -> Target {
        let mut target = Target::complete(state);
        for (position, set) in target.allowed.iter_mut().enumerate() {
            if !keep(position) {
                *set = ANY;
            }
        }
        target
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// The allowed colors of every sticker as bit sets (bit `i` is `Face::ALL[i]`).
    pub fn sets(&self) -> &[u8] {
        &self.allowed
    }

    /// Whether a sticker may show any color.
    pub fn is_open(&self, position: usize) -> bool {
        self.allowed[position] == ANY
    }

    /// How many stickers are constrained.
    pub fn constrained(&self) -> usize {
        self.allowed.iter().filter(|&&set| set != ANY).count()
    }

    /// The state this target is, when every sticker has exactly one allowed
    /// color and the colors form a cube: then the old code path applies.
    pub fn as_state(&self) -> Option<Facelets> {
        let stickers: Option<Vec<Face>> = self
            .allowed
            .iter()
            .map(|&set| (set.count_ones() == 1).then(|| Face::ALL[set.trailing_zeros() as usize]))
            .collect();
        Facelets::from_stickers(stickers?).ok()
    }

    /// Whether `state` shows an allowed color on every sticker.
    pub fn matches(&self, state: &Facelets) -> bool {
        state.kind() == self.kind
            && state
                .stickers()
                .iter()
                .zip(&self.allowed)
                .all(|(face, set)| set >> face.index() & 1 == 1)
    }

    /// Whether `moves` take `start` to a state of this target. A 2x2x2 has no
    /// centers to fix its frame, so (as for complete targets) a whole-cube
    /// rotation of the end state also counts.
    pub fn reached_by(&self, start: &Facelets, moves: &MoveSequence) -> bool {
        let Ok(end) = start.apply(moves) else {
            return false;
        };
        if self.kind != PuzzleKind::Cube2 {
            return self.matches(&end);
        }
        rotations().iter().any(|rotation| {
            end.apply(rotation)
                .is_ok_and(|turned| self.matches(&turned))
        })
    }

    /// The text form with `X` for an open sticker, or `None` when a sticker
    /// has a set of two to five colors (no syntax for it yet).
    pub fn to_text(&self) -> Option<String> {
        self.allowed
            .iter()
            .map(|&set| match set.count_ones() {
                1 => Some(Face::ALL[set.trailing_zeros() as usize].letter()),
                6 => Some('X'),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

    fn solved3() -> Facelets {
        Facelets::solved(PuzzleKind::Cube3)
    }

    #[test]
    fn a_complete_target_is_its_state() {
        let target = Target::complete(&solved3());
        assert_eq!(target.constrained(), 54);
        assert_eq!(target.as_state(), Some(solved3()));
        assert_eq!(target.to_text().as_deref(), Some(SOLVED));
    }

    #[test]
    fn open_stickers_make_it_partial() {
        let target = Target::masked(&solved3(), |position| position < 9);
        assert_eq!(target.constrained(), 9);
        assert!(target.as_state().is_none());
        assert!(target.is_open(9) && !target.is_open(0));
        assert!(target.to_text().unwrap().starts_with("UUUUUUUUUXXX"));
        let moved = solved3().apply(&"R".parse().unwrap()).unwrap();
        assert!(!target.matches(&moved));
        let down = solved3().apply(&"D".parse().unwrap()).unwrap();
        assert!(target.matches(&down));
    }

    #[test]
    fn contradictions_are_named() {
        assert_eq!(
            Target::from_sets(vec![1; 5]),
            Err(TargetError::BadLength(5))
        );
        let mut sets = vec![ANY; 54];
        sets[3] = 0;
        assert_eq!(
            Target::from_sets(sets),
            Err(TargetError::EmptySet { position: 3 })
        );
        let error = Target::from_sets(vec![1; 54]).unwrap_err();
        assert!(matches!(
            error,
            TargetError::TooMany {
                wanted: 54,
                have: 9,
                ..
            }
        ));
    }

    #[test]
    fn a_two_by_two_target_counts_after_a_rotation() {
        let start = Facelets::solved(PuzzleKind::Cube2);
        let target = Target::masked(&start, |position| position < 4);
        let rotate: MoveSequence = "R L'".parse().unwrap();
        assert!(target.reached_by(&start, &rotate));
    }
}
