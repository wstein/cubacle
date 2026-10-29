//! Named subsets of the solved cube as partial targets: a fixed mask over the
//! solved cube, written as a single word for `solve --to`.

use crate::puzzle::{Face, Facelets, Mask, PuzzleKind, Target};

/// The names [`named_subset`] knows. A face letter after a dash picks the face:
/// `cross-u`, `cross-r`, `cross-f`, `cross-d`, `cross-l`, `cross-b`.
/// `centers` is the solved centers of a 4x4x4 to 7x7x7.
pub const NAMED_SUBSETS: [&str; 3] = ["cross", "cross-<face>", "centers"];

/// The two facelets of each 3x3x3 edge slot (UR UF UL UB DR DF DL DB FR FL BL BR).
const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

/// The target a subset name stands for on a cube of `kind`.
///
/// `None` when `text` is not exactly one of the names (a name is only taken when
/// it is the whole text); `Some(Err(..))` when the subset does not exist on that
/// size. `cross` is `cross-d`: the four edges of the down face, both stickers
/// each, and the centers, over the solved cube.
pub fn named_subset(text: &str, kind: PuzzleKind) -> Option<Result<Target, String>> {
    if text == "centers" {
        return Some(centers(kind));
    }
    let face = match text {
        "cross" => Face::D,
        _ => {
            let letter = text.strip_prefix("cross-")?;
            let mut chars = letter.chars();
            let face = Face::from_letter(chars.next()?.to_ascii_uppercase())?;
            if chars.next().is_some() || !letter.chars().all(|c| c.is_ascii_lowercase()) {
                return None;
            }
            face
        }
    };
    if kind != PuzzleKind::Cube3 {
        return Some(Err(format!(
            "{text} is a 3x3x3 subset, not a {0}x{0}x{0} one",
            kind.size()
        )));
    }
    let mut keep = [false; 54];
    for pair in EDGE_FACELETS {
        if pair.iter().any(|&f| f / 9 == face.index()) {
            for f in pair {
                keep[f] = true;
            }
        }
    }
    for center in 0..6 {
        keep[center * 9 + 4] = true;
    }
    Some(Ok(Target::masked(&Facelets::solved(kind), |i| keep[i])))
}

/// `centers`: every center sticker of the solved cube, the rest open. It needs
/// a cube with centers that move, so a 4x4x4 to 7x7x7.
fn centers(kind: PuzzleKind) -> Result<Target, String> {
    if kind.size() < 4 {
        return Err(format!(
            "centers is a 4x4x4 to 7x7x7 subset, not a {0}x{0}x{0} one",
            kind.size()
        ));
    }
    Ok(Target::masked_by(
        &Facelets::solved(kind),
        &Mask::centers(kind),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_down_cross_has_eight_edge_stickers_and_the_centers() {
        let target = named_subset("cross", PuzzleKind::Cube3).unwrap().unwrap();
        assert_eq!(target.constrained(), 8 + 6);
        assert_eq!(
            Some(target),
            named_subset("cross-d", PuzzleKind::Cube3).map(Result::unwrap)
        );
    }

    #[test]
    fn every_face_has_a_cross() {
        for face in Face::ALL {
            let name = format!("cross-{}", face.letter().to_ascii_lowercase());
            let target = named_subset(&name, PuzzleKind::Cube3).unwrap().unwrap();
            assert_eq!(target.constrained(), 14, "{name}");
            // the edge stickers on the face itself are the face color
            let on_face = (0..9)
                .filter(|&i| i % 2 == 1)
                .all(|i| target.sets()[face.index() * 9 + i] == 1 << face.index());
            assert!(on_face, "{name}");
        }
    }

    #[test]
    fn names_are_whole_words_and_sizes_are_checked() {
        assert!(named_subset("cross R", PuzzleKind::Cube3).is_none());
        assert!(named_subset("Cross", PuzzleKind::Cube3).is_none());
        assert!(named_subset("cross-x", PuzzleKind::Cube3).is_none());
        assert!(named_subset("R U", PuzzleKind::Cube3).is_none());
        let error = named_subset("cross", PuzzleKind::Cube4)
            .unwrap()
            .unwrap_err();
        assert!(
            error.contains("3x3x3") && error.contains("4x4x4"),
            "{error}"
        );
    }

    #[test]
    fn centers_constrain_only_the_centers_of_a_big_cube() {
        for (kind, count) in [
            (PuzzleKind::Cube4, 24),
            (PuzzleKind::Cube5, 54),
            (PuzzleKind::Cube6, 96),
            (PuzzleKind::Cube7, 150),
        ] {
            let target = named_subset("centers", kind).unwrap().unwrap();
            assert_eq!(target.constrained(), count, "{kind:?}");
            assert_eq!(target.care(), Mask::centers(kind));
        }
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3] {
            let error = named_subset("centers", kind).unwrap().unwrap_err();
            assert!(error.contains("4x4x4 to 7x7x7"), "{error}");
        }
        assert!(named_subset("center", PuzzleKind::Cube4).is_none());
    }
}
