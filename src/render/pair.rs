//! A whole cube as two pictures: seen from the Up-Right-Front corner, as every
//! icon is, and from the opposite Down-Left-Back corner.
//!
//! The first view shows U, R and F, the second L, B and D, so together they
//! show every sticker once. The second view is the first one of the cube turned
//! with `x2 y'`: the D face is on top, L is the flat front face and B is the
//! oblique right face. Physically: hold the cube with D up, L towards you and B
//! on your right. Turning the cube over (`x2`, front to back) and then a quarter
//! turn like `U'` (`y'`) gives that grip from the usual one (U up, F towards you).

use super::action::Action;
use super::{state_icon, state_icon_against, Icon, Style};
use crate::puzzle::Facelets;

/// The two rotations that bring the Down-Left-Back corner to the Up-Right-Front one.
const TURN: [&str; 2] = ["x2", "y'"];

/// The heading of the text picture of the first view.
const URF_HEADER: &str = "state, seen from URF: U up, F towards you, R on your right";
/// The heading of the text picture of the second view.
const LBD_HEADER: &str = "state, seen from DBL: D up, L towards you, B on your right";

/// The cube turned (`x2 y'`) so that its Down-Left-Back corner is where the
/// Up-Right-Front corner was. Its U, F and R faces are the D, L and B faces of `state`.
pub fn opposite_view(state: &Facelets) -> Facelets {
    TURN.iter().fold(state.clone(), |cube, token| {
        let action: Action = token.parse().expect("a rotation");
        action.apply(&cube).expect("a rotation fits every cube")
    })
}

/// [`state_icon`] twice: from the Up-Right-Front corner and from the opposite
/// Down-Left-Back corner (see the module text). Their text pictures say which.
pub fn state_icon_pair(state: &Facelets, style: &Style) -> [Icon; 2] {
    let mut pair = [
        state_icon(state, style),
        state_icon(&opposite_view(state), style),
    ];
    label(&mut pair);
    pair
}

/// [`state_icon_pair`] with an amber square on every sticker that differs from
/// `reference`: each of the six faces is in one of the two pictures, so none is missed.
pub fn state_icon_pair_against(state: &Facelets, reference: &Facelets, style: &Style) -> [Icon; 2] {
    let mut pair = [
        state_icon_against(state, reference, style),
        state_icon_against(&opposite_view(state), &opposite_view(reference), style),
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
    use crate::render::{differing_stickers, scene::Part};

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
            let mut covered = vec![0u8; count];
            for (i, c) in covered.iter_mut().enumerate() {
                *c += u8::from(seen(i));
            }
            // Each sticker of D, L, B is at a position of U, F, R after the turn:
            // (x, y, z) goes to (-z, -y, -x) and the normal with it.
            let mut hidden = 0;
            for i in (0..count).filter(|&i| !seen(i)) {
                let (p, normal) = locate(kind, i);
                let turn = |v: [i32; 3]| [-v[2], -v[1], -v[0]];
                let want = (turn(p), turn(normal));
                let j = (0..count).find(|&j| locate(kind, j) == want).unwrap();
                assert!(seen(j), "{kind:?} sticker {i} lands on a hidden face");
                assert_eq!(turned.stickers()[j], cube.stickers()[i], "{kind:?} {i}");
                covered[i] += 1;
                hidden += 1;
            }
            assert_eq!(hidden, count / 2);
            assert!(covered.iter().all(|&c| c == 1), "{kind:?}");
        }
    }

    #[test]
    fn on_a_solved_cube_the_second_view_shows_d_l_b() {
        let turned = opposite_view(&Facelets::solved(PuzzleKind::Cube3));
        let face = |f: Face| turned.stickers()[f as usize * 9];
        assert_eq!(
            (face(Face::U), face(Face::F), face(Face::R)),
            (Face::D, Face::L, Face::B)
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
}
