//! A whole cube as two pictures: seen from the Up-Right-Front corner, as every
//! icon is, and from the opposite Down-Left-Back corner.
//!
//! The first view shows U, R and F, the second B, L and D, so together they
//! show every sticker once. The four side faces form one belt from left to
//! right: F and R in the first picture, B and L in the second, all upright (their
//! U edge on top). The second picture is the cube turned around the vertical
//! axis (`y2`, so that B is in front and L on the right) and seen from below:
//! B is the large face, L the narrow side and D lies under B. Physically: turn
//! the cube around so that B faces you, keep U up, and crouch to look up at it
//! from the left-hand side.

use super::action::Action;
use super::geometry::Camera;
use super::{
    state_icon, state_icon_against, state_icon_against_seen, state_icon_seen, Icon, Style,
};
use crate::puzzle::Facelets;

/// The rotation that brings B to the front and L to the right.
const TURN: &str = "y2";

/// The heading of the text picture of the first view.
const URF_HEADER: &str = "state, seen from URF: U up, F towards you, R on your right";
/// The heading of the text picture of the second view.
const LBD_HEADER: &str = "state, seen from DBL, turned so that B is in front: B, L, D below";

/// The cube turned (`y2`) so that B is in front and L on the right. Its F and R
/// faces are the B and L faces of `state`; the second picture shows it from below.
pub fn opposite_view(state: &Facelets) -> Facelets {
    let action: Action = TURN.parse().expect("a rotation");
    action.apply(state).expect("a rotation fits every cube")
}

/// [`state_icon`] twice: from the Up-Right-Front corner and from the opposite
/// Down-Left-Back corner (see the module text). Their text pictures say which.
pub fn state_icon_pair(state: &Facelets, style: &Style) -> [Icon; 2] {
    let mut pair = [
        state_icon(state, style),
        state_icon_seen(&opposite_view(state), style, Camera::Below),
    ];
    label(&mut pair);
    pair
}

/// [`state_icon_pair`] with an amber square on every sticker that differs from
/// `reference`: each of the six faces is in one of the two pictures, so none is missed.
pub fn state_icon_pair_against(state: &Facelets, reference: &Facelets, style: &Style) -> [Icon; 2] {
    let mut pair = [
        state_icon_against(state, reference, style),
        state_icon_against_seen(
            &opposite_view(state),
            &opposite_view(reference),
            style,
            Camera::Below,
        ),
    ];
    label(&mut pair);
    pair
}

fn label(pair: &mut [Icon; 2]) {
    pair[0].grid.header = URF_HEADER.to_owned();
    pair[1].grid.header = LBD_HEADER.to_owned();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{locate, Face, MoveSequence, PuzzleKind};
    use crate::render::{
        differing_stickers,
        scene::{Part, Shape},
    };

    fn scrambled(kind: PuzzleKind) -> Facelets {
        let moves: MoveSequence = match kind.size() {
            2 => "R U2 F' R U' F2",
            3 => "R U2 F' D L2 B' R' U F2",
            _ => "R U2 F' D L2 B' 2R 2U' 2F Rw Uw' 3Fw",
        }
        .parse()
        .unwrap();
        Facelets::solved(kind).apply(&moves).unwrap()
    }

    #[test]
    fn the_two_views_show_every_sticker_once_in_the_right_place() {
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube5,
            PuzzleKind::Cube7,
        ] {
            let cube = scrambled(kind);
            let turned = opposite_view(&cube);
            let len = kind.face_len();
            let count = kind.facelet_count();
            let seen = |index: usize| matches!(Face::ALL[index / len], Face::U | Face::R | Face::F);
            let seen_turned =
                |index: usize| matches!(Face::ALL[index / len], Face::D | Face::R | Face::F);
            let mut covered = vec![0u8; count];
            for (i, c) in covered.iter_mut().enumerate() {
                *c += u8::from(seen(i));
            }
            // Each sticker of D, L, B is at a position of D, R, F after the turn:
            // (x, y, z) goes to (-x, y, -z) and the normal with it.
            let mut hidden = 0;
            for i in (0..count).filter(|&i| !seen(i)) {
                let (p, normal) = locate(kind, i);
                let turn = |v: [i32; 3]| [-v[0], v[1], -v[2]];
                let want = (turn(p), turn(normal));
                let j = (0..count).find(|&j| locate(kind, j) == want).unwrap();
                assert!(
                    seen_turned(j),
                    "{kind:?} sticker {i} lands on a hidden face"
                );
                assert_eq!(turned.stickers()[j], cube.stickers()[i], "{kind:?} {i}");
                covered[i] += 1;
                hidden += 1;
            }
            assert_eq!(hidden, count / 2);
            assert!(covered.iter().all(|&c| c == 1), "{kind:?}");
        }
    }

    #[test]
    fn on_a_solved_cube_the_second_view_shows_b_l_d() {
        let turned = opposite_view(&Facelets::solved(PuzzleKind::Cube3));
        let face = |f: Face| turned.stickers()[f as usize * 9];
        assert_eq!(
            (face(Face::F), face(Face::R), face(Face::D)),
            (Face::B, Face::L, Face::D)
        );
    }

    #[test]
    fn marks_cover_the_hidden_faces_in_one_of_the_two_views() {
        let solved = Facelets::solved(PuzzleKind::Cube5);
        let turned = scrambled(PuzzleKind::Cube5);
        let pair = state_icon_pair_against(&turned, &solved, &Style::default());
        let marks = |icon: &Icon| icon.shapes.iter().filter(|s| s.part == Part::Mark).count();
        assert_eq!(
            marks(&pair[0]) + marks(&pair[1]),
            differing_stickers(&turned, &solved)
        );
        let plain = state_icon_pair(&solved, &Style::default());
        assert_eq!(marks(&plain[0]) + marks(&plain[1]), 0);
        assert!(plain[1].to_text(false).starts_with("state, seen from DBL"));
        assert!(plain[0].to_text(false).starts_with("state, seen from URF"));
    }

    /// The centers of the stickers of `cube` that are drawn differently from the
    /// solved cube in `view` of the pair: each given sticker is swapped with one that is out of sight there.
    fn painted(kind: PuzzleKind, view: usize, indices: &[usize]) -> Vec<[f64; 2]> {
        let style = Style::default();
        let solved = Facelets::solved(kind);
        let mut stickers = solved.stickers().to_vec();
        // swap each chosen sticker with one on a face that this view does not draw
        let len = kind.face_len();
        let hidden = if view == 0 {
            [Face::D, Face::L, Face::B]
        } else {
            [Face::U, Face::R, Face::F]
        };
        let own = Face::ALL[indices[0] / len];
        let donor = hidden.into_iter().find(|&f| f != own).unwrap();
        for (k, &i) in indices.iter().enumerate() {
            stickers.swap(i, donor as usize * len + k);
        }
        let cube = Facelets::from_stickers(stickers).unwrap();
        let sticker_shapes = |icon: &Icon| -> Vec<(Vec<[f64; 2]>, crate::render::Rgb)> {
            icon.shapes
                .iter()
                .filter(|s| s.part == Part::Sticker)
                .map(|s| (s.points.clone(), s.fill))
                .collect()
        };
        let before = sticker_shapes(&state_icon_pair(&solved, &style)[view]);
        let after = sticker_shapes(&state_icon_pair(&cube, &style)[view]);
        before
            .iter()
            .zip(&after)
            .filter(|(a, b)| a.1 != b.1)
            .map(|(_, b)| center(&b.0))
            .collect()
    }

    fn center(points: &[[f64; 2]]) -> [f64; 2] {
        let n = points.len() as f64;
        [
            points.iter().map(|p| p[0]).sum::<f64>() / n,
            points.iter().map(|p| p[1]).sum::<f64>() / n,
        ]
    }

    fn face_indices(kind: PuzzleKind, face: Face, keep: impl Fn([i32; 3]) -> bool) -> Vec<usize> {
        let len = kind.face_len();
        let start = face as usize * len;
        (start..start + len)
            .filter(|&i| keep(locate(kind, i).0))
            .collect()
    }

    #[test]
    fn the_belt_reads_f_r_then_b_l_with_every_face_upright() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let n = kind.size() as i32 - 1;
            let whole = |face: Face, view: usize| {
                center(&painted(kind, view, &face_indices(kind, face, |_| true)))
            };
            // left to right: F R in the first picture, B L in the second
            assert!(whole(Face::F, 0)[0] < whole(Face::R, 0)[0], "{kind:?}");
            assert!(whole(Face::B, 1)[0] < whole(Face::L, 1)[0], "{kind:?}");
            // D lies under the side faces
            assert!(whole(Face::D, 1)[1] > whole(Face::B, 1)[1], "{kind:?}");
            assert!(whole(Face::D, 1)[1] > whole(Face::L, 1)[1], "{kind:?}");
            // U is above F in the first picture
            assert!(whole(Face::U, 0)[1] < whole(Face::F, 0)[1], "{kind:?}");
            // B and L are upright: the row next to U is above the row next to D
            for face in [Face::B, Face::L] {
                let row =
                    |y: i32| center(&painted(kind, 1, &face_indices(kind, face, |p| p[1] == y)));
                let (u_row, d_row) = (row(n), row(-n));
                // locate has y up, so y = n is the row next to U
                assert!(u_row[1] < d_row[1], "{kind:?} {face:?} is upside down");
            }
            // B meets L along x = -n, z = -n: B's column at x = -n is its right edge
            let column = |x: i32| {
                center(&painted(
                    kind,
                    1,
                    &face_indices(kind, Face::B, |p| p[0] == x),
                ))
            };
            assert!(column(-n)[0] > column(n)[0], "{kind:?} B is mirrored");
            // L's column nearest B (z = -n) is its left edge
            let column = |z: i32| {
                center(&painted(
                    kind,
                    1,
                    &face_indices(kind, Face::L, |p| p[2] == z),
                ))
            };
            assert!(column(-n)[0] < column(n)[0], "{kind:?} L is mirrored");
        }
    }

    #[test]
    fn marks_stay_on_their_stickers_in_both_views() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5] {
            let solved = Facelets::solved(kind);
            let cube = scrambled(kind);
            let style = Style::default();
            let against = state_icon_pair_against(&cube, &solved, &style);
            let plain = (
                state_icon_pair(&cube, &style),
                state_icon_pair(&solved, &style),
            );
            for (view, marked) in against.iter().enumerate() {
                let stickers = |icon: &Icon| -> Vec<Shape> {
                    let all = icon.shapes.iter();
                    all.filter(|s| s.part == Part::Sticker).cloned().collect()
                };
                let (now, then) = (stickers(&plain.0[view]), stickers(&plain.1[view]));
                assert_eq!(now.len(), then.len());
                let changed: Vec<[f64; 2]> = now
                    .iter()
                    .zip(&then)
                    .filter(|(a, b)| a.fill != b.fill)
                    .map(|(a, _)| center(&a.points))
                    .collect();
                let marks: Vec<[f64; 2]> = marked
                    .shapes
                    .iter()
                    .filter(|s| s.part == Part::Mark)
                    .map(|s| center(&s.points))
                    .collect();
                assert_eq!(marks.len(), changed.len(), "{kind:?} view {view}");
                for m in &marks {
                    assert!(
                        changed
                            .iter()
                            .any(|c| (c[0] - m[0]).hypot(c[1] - m[1]) < 0.2),
                        "{kind:?} view {view}: a mark is not on a changed sticker"
                    );
                }
            }
        }
    }

    #[test]
    fn the_second_text_picture_shows_b_and_l_above_d() {
        let text = state_icon_pair(&Facelets::solved(PuzzleKind::Cube3), &Style::default())[1]
            .to_text(false);
        assert_eq!(
            text,
            "state, seen from DBL, turned so that B is in front: B, L, D below\n\
             B B B O O O\nB B B O O O\nB B B O O O\n Y Y Y\n  Y Y Y\n   Y Y Y\n"
        );
    }
}
