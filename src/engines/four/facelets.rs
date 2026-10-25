//! Reading a facelet string into pieces, and writing pieces back.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/FullCube.java (the constructor
//! from a facelet array, `toFacelet`, the facelet tables), MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md. The
//! reference trusts its input; this reads it with checks and says what is
//! wrong.
//!
//! Stickers are numbered `face * 16 + row * 4 + column`, faces in the order
//! `URFDLB`, which is the order of [`Facelets`].

use super::cube::{Corners, Cube};
use crate::puzzle::{Face, Facelets, PuzzleKind};

const fn at(face: usize, i: usize) -> usize {
    face * 16 + i
}
const U: usize = 0;
const R: usize = 1;
const F: usize = 2;
const D: usize = 3;
const L: usize = 4;
const B: usize = 5;

/// The sticker of each of the 24 centres.
const CENTRE_FACELET: [usize; 24] = [
    at(U, 5),
    at(U, 6),
    at(U, 10),
    at(U, 9),
    at(D, 5),
    at(D, 6),
    at(D, 10),
    at(D, 9),
    at(F, 5),
    at(F, 6),
    at(F, 10),
    at(F, 9),
    at(B, 5),
    at(B, 6),
    at(B, 10),
    at(B, 9),
    at(R, 5),
    at(R, 6),
    at(R, 10),
    at(R, 9),
    at(L, 5),
    at(L, 6),
    at(L, 10),
    at(L, 9),
];

/// The two stickers of each of the 24 wings, in the order that defines the
/// wing's colours.
const EDGE_FACELET: [[usize; 2]; 24] = [
    [at(U, 13), at(F, 1)],
    [at(U, 4), at(L, 1)],
    [at(U, 2), at(B, 1)],
    [at(U, 11), at(R, 1)],
    [at(D, 13), at(B, 14)],
    [at(D, 4), at(L, 14)],
    [at(D, 2), at(F, 14)],
    [at(D, 11), at(R, 14)],
    [at(L, 11), at(F, 8)],
    [at(L, 4), at(B, 7)],
    [at(R, 11), at(B, 8)],
    [at(R, 4), at(F, 7)],
    [at(F, 2), at(U, 14)],
    [at(L, 2), at(U, 8)],
    [at(B, 2), at(U, 1)],
    [at(R, 2), at(U, 7)],
    [at(B, 13), at(D, 14)],
    [at(L, 13), at(D, 8)],
    [at(F, 13), at(D, 1)],
    [at(R, 13), at(D, 7)],
    [at(F, 4), at(L, 7)],
    [at(B, 11), at(L, 8)],
    [at(B, 4), at(R, 7)],
    [at(F, 11), at(R, 8)],
];

/// The three stickers of each of the 8 corners, the one on U or D first.
const CORNER_FACELET: [[usize; 3]; 8] = [
    [at(U, 15), at(R, 0), at(F, 3)],
    [at(U, 12), at(F, 0), at(L, 3)],
    [at(U, 0), at(L, 0), at(B, 3)],
    [at(U, 3), at(B, 0), at(R, 3)],
    [at(D, 3), at(F, 15), at(R, 12)],
    [at(D, 0), at(L, 15), at(F, 12)],
    [at(D, 12), at(B, 15), at(L, 12)],
    [at(D, 15), at(R, 15), at(B, 12)],
];

/// Colour (0 to 5) of a sticker of the home cube.
const fn home(sticker: usize) -> u8 {
    (sticker / 16) as u8
}

/// Reads the pieces of a cube in the standard colour scheme (`U` opposite
/// `D`, `R` opposite `L`, `F` opposite `B`, as in a solved cube).
pub(super) fn read(facelets: &Facelets) -> Result<Cube, &'static str> {
    if facelets.kind() != PuzzleKind::Cube4 {
        return Err("not a 4x4x4");
    }
    let mut f = [0u8; 96];
    for (slot, face) in f.iter_mut().zip(facelets.stickers()) {
        *slot = face.index() as u8;
    }

    let mut cube = Cube::solved();
    let mut count = [0u8; 6];
    for i in 0..24 {
        cube.ct[i] = f[CENTRE_FACELET[i]];
        count[usize::from(cube.ct[i])] += 1;
    }
    if count != [4; 6] {
        return Err("each colour must appear four times among the centres");
    }

    let mut wing_of = [[None; 6]; 6];
    for (j, pair) in EDGE_FACELET.iter().enumerate() {
        wing_of[usize::from(home(pair[0]))][usize::from(home(pair[1]))] = Some(j as u8);
    }
    let mut used = [false; 24];
    for i in 0..24 {
        let a = usize::from(f[EDGE_FACELET[i][0]]);
        let b = usize::from(f[EDGE_FACELET[i][1]]);
        let Some(j) = wing_of[a][b] else {
            return Err("an edge piece shows colours that no edge piece has");
        };
        if std::mem::replace(&mut used[usize::from(j)], true) {
            return Err("an edge piece appears twice");
        }
        cube.ep[i] = j;
    }

    let mut corner_of = [[None; 6]; 6];
    for (j, c) in CORNER_FACELET.iter().enumerate() {
        corner_of[usize::from(home(c[1]))][usize::from(home(c[2]))] = Some(j as u8);
    }
    let mut used = [false; 8];
    let mut twist = 0u32;
    let mut corners = Corners::SOLVED;
    for i in 0..8 {
        let is_ud = |ori: usize| matches!(f[CORNER_FACELET[i][ori]], 0 | 3);
        let uds: Vec<usize> = (0..3).filter(|&o| is_ud(o)).collect();
        let [ori] = uds[..] else {
            return Err("a corner piece does not show exactly one U or D colour");
        };
        let col1 = usize::from(f[CORNER_FACELET[i][(ori + 1) % 3]]);
        let col2 = usize::from(f[CORNER_FACELET[i][(ori + 2) % 3]]);
        let Some(j) = corner_of[col1][col2] else {
            return Err("a corner piece shows colours that no corner piece has");
        };
        if f[CORNER_FACELET[i][ori]] != home(CORNER_FACELET[usize::from(j)][0]) {
            return Err("a corner piece shows its colours in the wrong order");
        }
        if std::mem::replace(&mut used[usize::from(j)], true) {
            return Err("a corner piece appears twice");
        }
        corners.cp[i] = j;
        corners.co[i] = ori as u8;
        twist += ori as u32;
    }
    if !twist.is_multiple_of(3) {
        return Err("one corner is twisted on its own");
    }
    cube.corners = corners;
    Ok(cube)
}

/// The 96 stickers (colours 0 to 5) of a cube.
pub(super) fn stickers(cube: &Cube) -> [u8; 96] {
    let mut f = [0u8; 96];
    for i in 0..24 {
        f[CENTRE_FACELET[i]] = cube.ct[i];
    }
    for i in 0..24 {
        let w = usize::from(cube.ep[i]);
        f[EDGE_FACELET[i][0]] = home(EDGE_FACELET[w][0]);
        f[EDGE_FACELET[i][1]] = home(EDGE_FACELET[w][1]);
    }
    for c in 0..8 {
        let j = usize::from(cube.corners.cp[c]);
        let ori = usize::from(cube.corners.co[c]);
        for n in 0..3 {
            f[CORNER_FACELET[c][(n + ori) % 3]] = home(CORNER_FACELET[j][n]);
        }
    }
    f
}

/// The cube as a [`Facelets`] value.
#[cfg(test)]
pub(super) fn to_facelets(cube: &Cube) -> Facelets {
    let faces = stickers(cube)
        .iter()
        .map(|&c| Face::ALL[usize::from(c)])
        .collect();
    Facelets::from_stickers(faces).expect("sixteen stickers of each colour")
}

/// The 3x3x3 hidden in a cube whose wings are paired, in the colours of its
/// own centres (so that the cube it describes is in the standard scheme).
/// `None` if the centres are not solved or do not make a rotation.
pub(super) fn three_by_three(cube: &Cube) -> Option<Facelets> {
    let f = stickers(cube);
    // Which face each colour is the centre of.
    let mut face_of = [usize::MAX; 6];
    for face in 0..6 {
        let c = usize::from(f[at(face, 5)]);
        if f[at(face, 6)] as usize != c
            || f[at(face, 9)] as usize != c
            || f[at(face, 10)] as usize != c
        {
            return None;
        }
        face_of[c] = face;
    }
    if face_of.contains(&usize::MAX) {
        return None;
    }
    let mut out = Vec::with_capacity(54);
    for face in 0..6 {
        for row in 0..3 {
            for col in 0..3 {
                // 3x3 place -> sticker of the 4x4 (rows and columns 0,1,3).
                let (r4, c4) = ([0, 1, 3][row], [0, 1, 3][col]);
                out.push(Face::ALL[face_of[usize::from(f[at(face, r4 * 4 + c4)])]]);
            }
        }
    }
    Facelets::from_stickers(out).ok()
}

#[cfg(test)]
mod tests {
    use super::super::cube::CornerMoves;
    use super::*;
    use crate::puzzle::MoveSequence;

    const SCRAMBLE: &str =
        "R U2 F' Dw L2 Bw R' Uw' F2 Rw D2 Lw B' R2 Uw F Dw' L' Bw2 R Uw2 Fw L Rw'";

    fn model(moves: &str) -> Cube {
        let cm = CornerMoves::new();
        let mut cube = Cube::solved();
        for mv in moves.parse::<MoveSequence>().unwrap().moves() {
            let face = mv.face.index();
            let id = (face + if mv.wide { 6 } else { 0 }) * 3
                + match mv.turn {
                    crate::puzzle::Turn::Clockwise => 0,
                    crate::puzzle::Turn::Half => 1,
                    crate::puzzle::Turn::CounterClockwise => 2,
                };
            cube.apply(&cm, id);
        }
        cube
    }

    #[test]
    fn the_model_turns_like_the_facelets() {
        for moves in [
            "R", "U", "F", "D", "L", "B", "Rw", "Uw", "Fw", "Dw", "Lw", "Bw", "R2 Uw' F", SCRAMBLE,
        ] {
            let solved = Facelets::solved(PuzzleKind::Cube4);
            let turned = solved.apply(&moves.parse().unwrap()).unwrap();
            let cube = model(moves);
            assert_eq!(to_facelets(&cube), turned, "{moves}");
            assert_eq!(read(&turned), Ok(cube), "{moves}");
        }
    }

    #[test]
    fn the_solved_cube_reads_as_the_solved_model() {
        assert_eq!(
            read(&Facelets::solved(PuzzleKind::Cube4)),
            Ok(Cube::solved())
        );
    }

    #[test]
    fn impossible_cubes_are_named() {
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let base: Vec<Face> = solved.stickers().to_vec();
        let with = |edit: &dyn Fn(&mut Vec<Face>)| {
            let mut s = base.clone();
            edit(&mut s);
            read(&Facelets::from_stickers(s).unwrap())
        };
        // a corner twisted on its own
        let twisted = with(&|s| {
            let (a, b, c) = (at(U, 15), at(R, 0), at(F, 3));
            let t = s[a];
            s[a] = s[b];
            s[b] = s[c];
            s[c] = t;
        });
        assert_eq!(twisted, Err("one corner is twisted on its own"));
        // the two stickers of one wing exchanged: that colour pair is now present twice
        let swapped = with(&|s| s.swap(at(U, 13), at(F, 1)));
        assert!(swapped.is_err());
        // centres of the wrong count
        let centres = with(&|s| s.swap(at(U, 5), at(F, 1)));
        assert!(centres.is_err());
        // not a 4x4x4
        assert!(read(&Facelets::solved(PuzzleKind::Cube3)).is_err());
    }

    #[test]
    fn a_mirrored_colour_scheme_is_refused() {
        // Swap the colours R and L everywhere: a valid cube in another scheme.
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let mirrored: Vec<Face> = solved
            .stickers()
            .iter()
            .map(|&f| match f {
                Face::R => Face::L,
                Face::L => Face::R,
                other => other,
            })
            .collect();
        assert!(read(&Facelets::from_stickers(mirrored).unwrap()).is_err());
    }

    #[test]
    fn the_hidden_3x3x3_of_a_turned_reduced_cube_is_a_cube() {
        // Face turns keep the wings paired and the centres solved.
        let cube = model("R U2 F' D L2 B R' U'");
        let three = three_by_three(&cube).expect("a 3x3x3");
        let want = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R U2 F' D L2 B R' U'".parse().unwrap())
            .unwrap();
        assert_eq!(three, want);
        // Wide turns move the centres: not a reduced cube.
        assert!(three_by_three(&model("Rw U")).is_none());
    }
}
