//! A target other than the solved cube: the state that, solved, takes one
//! cube to another.
//!
//! Moves act on positions, so a state is a permutation of sticker positions
//! once every sticker is given an identity (the position it has on the solved
//! cube). If `S` is the start and `T` the target, the moves `y` that take `S`
//! to `T` are those that solve the relative state `R = T^-1 S`; sticker `p` of
//! `R` shows the face of the place where the start's sticker `p` has to go.

use std::collections::HashMap;
use std::fmt;

use super::cubies::{cubies, Cubie};
use super::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// Why a relative state could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeError {
    /// Start and target are cubes of different sizes.
    SizeMismatch {
        start: PuzzleKind,
        target: PuzzleKind,
    },
    /// A piece of the named state shows colours no piece of the cube has.
    ImpossiblePiece { which: &'static str },
    /// The target of a cube with movable centres has centres out of place,
    /// which the colour-based search cannot aim for.
    CentresMoved,
}

impl fmt::Display for RelativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelativeError::SizeMismatch { start, target } => write!(
                f,
                "the start is a {0}x{0}x{0} but the target is a {1}x{1}x{1}",
                start.size(),
                target.size()
            ),
            RelativeError::CentresMoved => write!(
                f,
                "the target has centres out of place; the solvers can only reach a target whose \
                 centres are on their own faces (turns of the outer layers keep them there)"
            ),
            RelativeError::ImpossiblePiece { which } => {
                write!(
                    f,
                    "the {which} state is impossible (a piece has colours no piece has)"
                )
            }
        }
    }
}

impl std::error::Error for RelativeError {}

type Vec3 = [i32; 3];

fn quarter(axis: usize, v: Vec3) -> Vec3 {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// The smallest of the 24 rotated images of a cell: the same for every cell
/// of one orbit, so a piece is only ever placed within its orbit.
fn orbit_key(position: Vec3) -> Vec3 {
    let mut seen = vec![position];
    let mut at = 0;
    while at < seen.len() {
        for axis in 0..3 {
            let next = quarter(axis, seen[at]);
            if !seen.contains(&next) {
                seen.push(next);
            }
        }
        at += 1;
    }
    seen.into_iter().min().expect("at least the cell itself")
}

/// What tells a piece in a cell from the others of its orbit: the colours of
/// its stickers in the cubie's order for a wing (whose two wings on an edge
/// differ in which colour sits where), else the set of its colours.
fn piece_key(cubie: &Cubie, kind: PuzzleKind, colours: &[Face]) -> (Vec3, Vec<u8>) {
    let wing = colours.len() == 2
        && cubie
            .position
            .iter()
            .any(|&c| c.abs() < kind.size() as i32 - 1 && c != 0);
    let mut ids: Vec<u8> = colours.iter().map(|c| c.index() as u8).collect();
    if !wing {
        ids.sort_unstable();
    }
    (orbit_key(cubie.position), ids)
}

fn face_of(kind: PuzzleKind, index: usize) -> Face {
    Face::ALL[index / kind.face_len()]
}

/// For every sticker of `state`, the place on the solved cube it comes from:
/// pieces are told apart by orbit and colours; identical pieces are paired in
/// index order, which depends on the state alone.
fn identities(state: &Facelets, which: &'static str) -> Result<Vec<usize>, RelativeError> {
    let kind = state.kind();
    let cells = cubies(kind);
    let mut free: HashMap<(Vec3, Vec<u8>), Vec<&Cubie>> = HashMap::new();
    for cubie in cells.iter().rev() {
        let colours: Vec<Face> = cubie.stickers.iter().map(|&s| face_of(kind, s)).collect();
        free.entry(piece_key(cubie, kind, &colours))
            .or_default()
            .push(cubie);
    }
    let mut identity = vec![0; kind.facelet_count()];
    for cubie in &cells {
        let colours: Vec<Face> = cubie
            .stickers
            .iter()
            .map(|&s| state.stickers()[s])
            .collect();
        let impossible = RelativeError::ImpossiblePiece { which };
        let distinct = colours.iter().fold(0u8, |m, c| m | 1 << c.index());
        if distinct.count_ones() as usize != colours.len() {
            return Err(impossible);
        }
        let home = free
            .get_mut(&piece_key(cubie, kind, &colours))
            .and_then(Vec::pop)
            .ok_or(impossible)?;
        for (&sticker, colour) in cubie.stickers.iter().zip(&colours) {
            identity[sticker] = *home
                .stickers
                .iter()
                .find(|&&h| face_of(kind, h) == *colour)
                .expect("the home cell shows these colours");
        }
    }
    Ok(identity)
}

/// The state whose solution takes `start` to `target`.
///
/// Solving the result with moves `y` means `start` followed by `y` shows
/// `target`, as far as the pieces are told apart by their colours. Identical
/// pieces (the centre stickers of a face, the two wings of an edge) are
/// interchangeable to the solvers, so callers replay the answer; it matches
/// when the target does not depend on which identical piece is which.
pub fn relative_state(start: &Facelets, target: &Facelets) -> Result<Facelets, RelativeError> {
    if start.kind() != target.kind() {
        return Err(RelativeError::SizeMismatch {
            start: start.kind(),
            target: target.kind(),
        });
    }
    let kind = start.kind();
    // Centres of one colour are interchangeable to the solvers, so a target
    // is only exact when every centre sticker sits on its own face.
    let centres_home = cubies(kind)
        .iter()
        .filter(|cubie| cubie.stickers.len() == 1)
        .all(|cubie| target.stickers()[cubie.stickers[0]] == face_of(kind, cubie.stickers[0]));
    if kind.size() >= 4 && !centres_home {
        return Err(RelativeError::CentresMoved);
    }
    let from = identities(start, "start")?;
    let to = identities(target, "target")?;
    let mut place = vec![0; kind.facelet_count()];
    for (position, &identity) in to.iter().enumerate() {
        place[identity] = position;
    }
    let stickers = from
        .iter()
        .map(|&identity| face_of(kind, place[identity]))
        .collect();
    Ok(Facelets::from_stickers(stickers).expect("every place is used once"))
}

/// Whether `moves` take `start` to `target`.
///
/// A 2x2x2 has no centres to fix its frame, and its solver ends in whichever
/// of the 24 whole-cube orientations suits it, so there the target counts as
/// reached when the cube shows it after a rotation of the whole cube.
pub fn reaches(start: &Facelets, moves: &MoveSequence, target: &Facelets) -> bool {
    let Ok(end) = start.apply(moves) else {
        return false;
    };
    if start.kind() != PuzzleKind::Cube2 {
        return end == *target;
    }
    rotations()
        .iter()
        .any(|rotation| end.apply(rotation).is_ok_and(|turned| turned == *target))
}

/// The 24 whole-cube rotations of a 2x2x2, as the turns of opposite layers
/// that make them.
fn rotations() -> Vec<MoveSequence> {
    let turn = |face, turn| Move::new(face, 1, false, turn);
    let axes = [
        [
            turn(Face::R, Turn::Clockwise),
            turn(Face::L, Turn::CounterClockwise),
        ],
        [
            turn(Face::U, Turn::Clockwise),
            turn(Face::D, Turn::CounterClockwise),
        ],
        [
            turn(Face::F, Turn::Clockwise),
            turn(Face::B, Turn::CounterClockwise),
        ],
    ];
    let solved = Facelets::solved(PuzzleKind::Cube2);
    let mut seen = vec![(solved.clone(), MoveSequence::new(Vec::new()))];
    let mut at = 0;
    while at < seen.len() {
        for axis in &axes {
            let mut moves = seen[at].1.moves().to_vec();
            moves.extend(axis);
            let moves = MoveSequence::new(moves);
            let state = solved.apply(&moves).expect("2x2 turns fit");
            if !seen.iter().any(|(s, _)| *s == state) {
                seen.push((state, moves));
            }
        }
        at += 1;
    }
    seen.into_iter().map(|(_, moves)| moves).collect()
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
    fn a_2x2_has_24_rotations() {
        assert_eq!(rotations().len(), 24);
    }

    #[test]
    fn the_relative_state_of_equal_cubes_is_solved() {
        for kind in PuzzleKind::ALL {
            let cube = cube(kind, "R U F2 D' L B");
            assert!(
                relative_state(&cube, &cube).unwrap().is_solved(),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn solving_the_relative_state_leads_from_start_to_target() {
        // start S = R U F2 D' L B, target T = L2 U B' D F; S^-1 T takes S to T
        let start = cube(PuzzleKind::Cube3, "R U F2 D' L B");
        let target = cube(PuzzleKind::Cube3, "L2 U B' D F");
        let relative = relative_state(&start, &target).unwrap();
        let back: MoveSequence = "B' L' D F2 U' R' L2 U B' D F".parse().unwrap();
        assert!(relative.apply(&back).unwrap().is_solved());
        assert!(reaches(&start, &back, &target));
    }

    #[test]
    fn size_mismatch_and_impossible_pieces_are_reported() {
        let three = Facelets::solved(PuzzleKind::Cube3);
        let two = Facelets::solved(PuzzleKind::Cube2);
        assert!(matches!(
            relative_state(&three, &two),
            Err(RelativeError::SizeMismatch { .. })
        ));
        let mut stickers = three.stickers().to_vec();
        // a corner sticker and an edge sticker: the corner shows R twice
        stickers.swap(8, 10);
        let broken = Facelets::from_stickers(stickers).unwrap();
        assert_eq!(
            relative_state(&three, &broken),
            Err(RelativeError::ImpossiblePiece { which: "target" })
        );
        assert_eq!(
            relative_state(&broken, &three),
            Err(RelativeError::ImpossiblePiece { which: "start" })
        );
    }

    #[test]
    fn a_4x4_target_with_moved_centres_is_refused() {
        let start = Facelets::solved(PuzzleKind::Cube4);
        let target = cube(PuzzleKind::Cube4, "Rw");
        assert_eq!(
            relative_state(&start, &target),
            Err(RelativeError::CentresMoved)
        );
    }
}
