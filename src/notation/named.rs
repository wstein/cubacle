//! Famous patterns that can be asked for by name, as the target of a solve.

use crate::puzzle::{Facelets, PuzzleKind};

/// The names [`named_target`] knows.
pub const NAMED_TARGETS: [&str; 3] = ["superflip", "checkerboard", "cube-in-cube"];

const SUPERFLIP: &str = "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2";
const CHECKERBOARD: &str = "U2 D2 R2 L2 F2 B2";
const CUBE_IN_CUBE: &str = "F L F U' R U F2 L2 U' L' B D' B' L2 U";

/// The cube a pattern name stands for, applied to the solved cube of `kind`.
///
/// `None` when `text` is not exactly one of [`NAMED_TARGETS`] (so a name is
/// only taken when nothing else could be meant); `Some(Err(..))` when the
/// pattern does not exist on that size. All three are 3x3x3 patterns: on the
/// larger cubes a chessboard needs turns of the inner layers, which move the
/// centers, and the solvers cannot aim for a target with moved centers.
pub fn named_target(text: &str, kind: PuzzleKind) -> Option<Result<Facelets, String>> {
    let moves = match text {
        "superflip" => SUPERFLIP,
        "checkerboard" => CHECKERBOARD,
        "cube-in-cube" => CUBE_IN_CUBE,
        _ => return None,
    };
    if kind != PuzzleKind::Cube3 {
        return Some(Err(format!(
            "{text} is a 3x3x3 pattern, not a {0}x{0}x{0} one",
            kind.size()
        )));
    }
    let moves = moves.parse().expect("the pattern is move notation");
    Some(
        Facelets::solved(kind)
            .apply(&moves)
            .map_err(|error| error.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Face;

    fn sticker(cube: &Facelets, face: usize, row: usize, col: usize) -> Face {
        cube.stickers()[face * 9 + row * 3 + col]
    }

    #[test]
    fn the_3x3_patterns_are_the_known_ones() {
        let flip = named_target("superflip", PuzzleKind::Cube3)
            .unwrap()
            .unwrap();
        // every edge flipped in place: corners and centers stay home
        for face in 0..6 {
            for at in 0..9 {
                let home = Face::ALL[face];
                let shown = sticker(&flip, face, at / 3, at % 3);
                assert_eq!(shown != home, at % 2 == 1, "face {face} sticker {at}");
            }
        }
        let flip_back = flip.apply(&SUPERFLIP.parse().unwrap()).unwrap();
        assert!(flip_back.is_solved());
        let board = named_target("checkerboard", PuzzleKind::Cube3)
            .unwrap()
            .unwrap();
        assert_eq!(
            board.to_string(),
            "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB"
        );
        assert!(named_target("cube-in-cube", PuzzleKind::Cube3)
            .unwrap()
            .is_ok());
    }

    #[test]
    fn a_name_is_only_taken_when_it_is_the_whole_text() {
        assert!(named_target("superflip R", PuzzleKind::Cube3).is_none());
        assert!(named_target("Superflip", PuzzleKind::Cube3).is_none());
        assert!(named_target("R U", PuzzleKind::Cube3).is_none());
    }

    #[test]
    fn every_name_is_refused_on_other_sizes() {
        for name in NAMED_TARGETS {
            let error = named_target(name, PuzzleKind::Cube4).unwrap().unwrap_err();
            assert!(
                error.contains("3x3x3") && error.contains("4x4x4"),
                "{error}"
            );
        }
    }
}
