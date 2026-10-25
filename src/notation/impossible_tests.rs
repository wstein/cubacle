use super::*;
use crate::puzzle::{Face, PuzzleKind};

fn edited(kind: PuzzleKind, edit: impl Fn(&mut Vec<Face>, usize)) -> Facelets {
    let mut stickers = Facelets::solved(kind).stickers().to_vec();
    edit(&mut stickers, kind.size());
    Facelets::from_stickers(stickers).unwrap()
}

/// The URF corner stickers of an `n`-cube: U, R and F.
fn urf(n: usize) -> [usize; 3] {
    [n * n - 1, n * n, 2 * n * n + n - 1]
}

/// The stickers of the UF and UB middle edges: (U, F) and (U, B).
fn uf_ub(n: usize) -> [[usize; 2]; 2] {
    let mid = n / 2;
    [
        [(n - 1) * n + mid, 2 * n * n + mid],
        [mid, 5 * n * n + mid],
    ]
}

#[test]
fn possible_states_pass_on_every_size() {
    for kind in PuzzleKind::ALL {
        let big = if kind.size() > 3 { " 2L Rw" } else { "" };
        let moves = format!("R U R' U' F D' B{big}").parse().unwrap();
        let cube = Facelets::solved(kind).apply(&moves).unwrap();
        assert_eq!(check(&cube), Ok(()), "{kind:?}");
        assert_eq!(check(&Facelets::solved(kind)), Ok(()), "{kind:?}");
    }
}

#[test]
fn a_twisted_corner_is_named_alike_on_every_size() {
    for kind in PuzzleKind::ALL {
        let cube = edited(kind, |s, n| {
            let [a, b, c] = urf(n);
            let (fa, fb, fc) = (s[a], s[b], s[c]);
            s[a] = fb;
            s[b] = fc;
            s[c] = fa;
        });
        let reason = check(&cube).unwrap_err();
        assert_eq!(reason, Impossible::CornerTwist, "{kind:?}");
        assert_eq!(
            reason.to_string(),
            "the corners are invalid: one corner is twisted on its own"
        );
        assert_eq!(reason.hint(), Some("turn one corner back"));
    }
}

#[test]
fn a_mirrored_corner_names_its_place_and_colours() {
    for kind in PuzzleKind::ALL {
        let cube = edited(kind, |s, n| {
            let [a, b, _] = urf(n);
            s.swap(a, b);
        });
        let reason = check(&cube).unwrap_err();
        assert_eq!(
            reason,
            Impossible::Sticker {
                kind: PieceKind::Corner,
                place: "URF".into(),
                seen: "RUF".into()
            },
            "{kind:?}"
        );
        assert_eq!(
            reason.to_string(),
            "the corners are invalid: the stickers RUF at corner URF are not a corner of the cube"
        );
        assert_eq!(reason.hint(), None);
    }
}

#[test]
fn a_flipped_edge_and_swapped_edges_on_odd_cubes() {
    for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
        let flipped = edited(kind, |s, n| {
            let [[u, f], _] = uf_ub(n);
            s.swap(u, f);
        });
        let reason = check(&flipped).unwrap_err();
        assert_eq!(reason, Impossible::EdgeFlip, "{kind:?}");
        assert_eq!(reason.hint(), Some("flip one edge back"));

        let swapped = edited(kind, |s, n| {
            let [[u1, f], [u2, b]] = uf_ub(n);
            s.swap(u1, u2);
            s.swap(f, b);
        });
        let reason = check(&swapped).unwrap_err();
        assert_eq!(reason, Impossible::Parity, "{kind:?}");
        assert_eq!(reason.hint(), Some("swap two edges"));
    }
}

#[test]
fn wings_and_centres_of_the_larger_cubes() {
    for kind in [
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
        PuzzleKind::Cube6,
        PuzzleKind::Cube7,
    ] {
        // The two stickers of the first wing on the U face exchanged.
        let wing = edited(kind, |s, n| s.swap(1, 5 * n * n + n - 2));
        let text = check(&wing).unwrap_err().to_string();
        assert!(text.starts_with("the wings are invalid"), "{kind:?} {text}");

        // A centre sticker exchanged with a sticker of another colour.
        // (The 4x4x4 has one centre orbit, so no such exchange breaks only it.)
        if kind != PuzzleKind::Cube4 {
            let centre = edited(kind, |s, n| s.swap(n + 1, 2 * n * n + n + 2));
            assert_eq!(check(&centre), Err(Impossible::CentreColours), "{kind:?}");
        }
    }
    // Fixed centres that are not a rotation.
    let moved = edited(PuzzleKind::Cube5, |s, _| s.swap(12, 37));
    assert_eq!(check(&moved), Err(Impossible::FixedCentres));
}
