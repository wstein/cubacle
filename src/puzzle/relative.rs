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
    /// A piece of the named state shows colors no piece of the cube has.
    ImpossiblePiece { which: &'static str },
    /// The target shows centers of one kind (the centers of one orbit on one
    /// face) in different colors. The relative state only tells a center the
    /// face it goes to, not the place, so it cannot express the target; the
    /// route to such a target is planned by `centers::plan`, which aims at a
    /// nearby target with uniform centers and finishes with a center fix-up.
    CentersMoved,
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
            RelativeError::CentersMoved => write!(
                f,
                "the target has centers of different colors on one face, which a relative state \
                 cannot express; plan the route with `centers::plan`, which ends in a center fix-up"
            ),
            RelativeError::ImpossiblePiece { which } => {
                write!(
                    f,
                    "the {which} state is impossible (a piece has colors no piece has)"
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

/// What tells a piece in a cell from the others of its orbit: the colors of
/// its stickers in the cubie's order for a wing (whose two wings on an edge
/// differ in which color sits where), else the set of its colors.
fn piece_key(cubie: &Cubie, kind: PuzzleKind, colors: &[Face]) -> (Vec3, Vec<u8>) {
    let wing = colors.len() == 2
        && cubie
            .position
            .iter()
            .any(|&c| c.abs() < kind.size() as i32 - 1 && c != 0);
    let mut ids: Vec<u8> = colors.iter().map(|c| c.index() as u8).collect();
    if !wing {
        ids.sort_unstable();
    }
    (orbit_key(cubie.position), ids)
}

fn face_of(kind: PuzzleKind, index: usize) -> Face {
    Face::ALL[index / kind.face_len()]
}

/// The key of the piece in every cell of `state`.
fn piece_keys(
    state: &Facelets,
    cells: &[Cubie],
    which: &'static str,
) -> Result<Vec<(Vec3, Vec<u8>)>, RelativeError> {
    let kind = state.kind();
    cells
        .iter()
        .map(|cubie| {
            let colors: Vec<Face> = cubie
                .stickers
                .iter()
                .map(|&s| state.stickers()[s])
                .collect();
            let distinct = colors.iter().fold(0u8, |m, c| m | 1 << c.index());
            if distinct.count_ones() as usize != colors.len() {
                return Err(RelativeError::ImpossiblePiece { which });
            }
            Ok(piece_key(cubie, kind, &colors))
        })
        .collect()
}

/// For every cell, the cell of `target` that the start's piece in it has to
/// reach. Pieces of equal colors are interchangeable, so the pairing is a
/// choice: a piece already in place keeps its place (fixed points first), and
/// the rest of one kind are paired in cell order, the start's n-th loose piece
/// with the target's n-th free place. Wings differ in their color order and
/// so are never ambiguous; the centers are, and any pairing of equal centers
/// reaches the target color-wise, since the solvers cannot tell them apart.
fn assignment(
    start: &[(Vec3, Vec<u8>)],
    target: &[(Vec3, Vec<u8>)],
    reversed: bool,
) -> Result<Vec<usize>, RelativeError> {
    let mut to = vec![usize::MAX; start.len()];
    let mut taken = vec![false; target.len()];
    for cell in 0..start.len() {
        if start[cell] == target[cell] {
            to[cell] = cell;
            taken[cell] = true;
        }
    }
    let mut free: HashMap<&(Vec3, Vec<u8>), Vec<usize>> = HashMap::new();
    let mut open: Vec<usize> = (0..target.len()).filter(|&c| !taken[c]).collect();
    if !reversed {
        open.reverse();
    }
    for cell in open {
        free.entry(&target[cell]).or_default().push(cell);
    }
    for cell in 0..start.len() {
        if to[cell] != usize::MAX {
            continue;
        }
        // the color sets of the two states differ: not the same cube
        to[cell] = free
            .get_mut(&start[cell])
            .and_then(Vec::pop)
            .ok_or(RelativeError::ImpossiblePiece { which: "target" })?;
    }
    Ok(to)
}

/// Whether the target shows one color for the centers of one orbit on one
/// face. The color-based search only knows the face a center has to reach,
/// so the relative state can express a target only if that face fixes the
/// color.
fn centers_uniform(target: &Facelets, cells: &[Cubie]) -> bool {
    let kind = target.kind();
    let mut seen: HashMap<(usize, Vec3), Face> = HashMap::new();
    cells
        .iter()
        .filter(|cubie| cubie.stickers.len() == 1)
        .all(|cubie| {
            let sticker = cubie.stickers[0];
            let key = (sticker / kind.face_len(), orbit_key(cubie.position));
            *seen.entry(key).or_insert(target.stickers()[sticker]) == target.stickers()[sticker]
        })
}

/// The state whose solution takes `start` to `target`.
///
/// Solving the result with moves `y` means `start` followed by `y` shows
/// `target`, as far as the pieces are told apart by their colors. Equal
/// pieces are paired by [`assignment`]: fixed points first, then in cell
/// order.
///
/// Parity: wings differ in the order of their two colors (and the orbits of
/// the 5x5 and larger are kept apart), so the wings of a 4x4 or 6x6 are never
/// ambiguous and their parity is that of the real cube. Only centers are
/// paired freely, which can change the permutation parity of the centers; the
/// reduction solvers read centers by color and ignore it, so either parity
/// is a state they solve. A target whose centers of one kind differ in color
/// on one face is refused with [`RelativeError::CentersMoved`], because the
/// relative state only tells a center which face to reach, not which place;
/// `centers::plan` turns such a target into one with uniform centers and a
/// fix-up.
pub fn relative_state(start: &Facelets, target: &Facelets) -> Result<Facelets, RelativeError> {
    relative_by(start, target, false)
}

/// [`relative_state`] with the loose equal pieces paired in cell order or, for
/// tests that show the choice does not matter, in reverse order.
pub(crate) fn relative_by(
    start: &Facelets,
    target: &Facelets,
    reversed: bool,
) -> Result<Facelets, RelativeError> {
    if start.kind() != target.kind() {
        return Err(RelativeError::SizeMismatch {
            start: start.kind(),
            target: target.kind(),
        });
    }
    let kind = start.kind();
    let cells = cubies(kind);
    if start == target {
        return Ok(Facelets::solved(kind));
    }
    if !centers_uniform(target, &cells) {
        return Err(RelativeError::CentersMoved);
    }
    let from = piece_keys(start, &cells, "start")?;
    let to = piece_keys(target, &cells, "target")?;
    let place = assignment(&from, &to, reversed)?;
    let mut stickers = vec![Face::U; kind.facelet_count()];
    for (cubie, &goal) in cells.iter().zip(&place) {
        for &sticker in &cubie.stickers {
            let color = start.stickers()[sticker];
            let spot = cells[goal]
                .stickers
                .iter()
                .find(|&&t| target.stickers()[t] == color)
                .expect("the paired cell shows these colors");
            stickers[sticker] = face_of(kind, *spot);
        }
    }
    Ok(Facelets::from_stickers(stickers).expect("every place is used once"))
}

/// Whether `moves` take `start` to `target`.
///
/// A 2x2x2 has no centers to fix its frame, and its solver ends in whichever
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
pub(super) fn rotations() -> Vec<MoveSequence> {
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

    const KINDS: [PuzzleKind; 4] = [
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
        PuzzleKind::Cube6,
        PuzzleKind::Cube7,
    ];

    /// Pairs of a start (moves from solved), the moves `m` to the target and
    /// the inverse of `m`; the target keeps every face's centers one color.
    const ROUTES: [(&str, &str, &str); 3] = [
        ("", "R U2 F' L2 D B'", "B D' L2 F U2 R'"),
        ("R U F2 D' L B", "L2 U' B R2 D F'", "F D' R2 B' U L2"),
        ("U2 R' F D2 L", "R2 L2 U2 D2 F2 B2", "B2 F2 D2 U2 L2 R2"),
    ];

    #[test]
    fn either_pairing_of_equal_pieces_is_solved_by_the_route() {
        for kind in KINDS {
            for (start, route, back) in ROUTES {
                let a = cube(kind, start);
                let m: MoveSequence = route.parse().unwrap();
                let b = a.apply(&m).unwrap();
                for reversed in [false, true] {
                    let r = relative_by(&a, &b, reversed).unwrap();
                    assert!(r.apply(&m).unwrap().is_solved(), "{kind:?} {start} {route}");
                    let inverse = relative_by(&b, &a, reversed).unwrap();
                    let back: MoveSequence = back.parse().unwrap();
                    assert!(inverse.apply(&back).unwrap().is_solved(), "{kind:?} back");
                }
            }
        }
    }

    #[test]
    fn a_rotated_4x4_target_is_reached() {
        // the centers moved to other faces, but each face is still one color
        let a = cube(PuzzleKind::Cube4, "R U");
        let m: MoveSequence = "Rw2 Lw2".parse().unwrap();
        let b = a.apply(&m).unwrap();
        let r = relative_state(&a, &b).unwrap();
        assert!(r.apply(&m).unwrap().is_solved());
    }

    #[test]
    fn the_relative_state_of_a_cube_with_itself_is_solved_on_scrambled_centers() {
        for kind in KINDS {
            let a = cube(kind, "Rw U Fw2 D' Lw B");
            assert!(relative_state(&a, &a).unwrap().is_solved(), "{kind:?}");
        }
    }

    #[test]
    fn a_4x4_target_with_mixed_centers_on_a_face_is_refused() {
        let start = Facelets::solved(PuzzleKind::Cube4);
        let target = cube(PuzzleKind::Cube4, "Rw");
        assert_eq!(
            relative_state(&start, &target),
            Err(RelativeError::CentersMoved)
        );
    }
}
