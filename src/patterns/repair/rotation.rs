//! The 24 whole-cube rotations as sticker permutations, and the color
//! comparisons that go with them.

use crate::puzzle::{Face, Move, PuzzleKind, Turn};

use crate::meet::perm_of;

pub(super) struct Rotation {
    pub(super) perm: Vec<u16>,
    /// The rotations that make it, as `x y2 x'`; empty for the identity.
    pub(super) name: String,
}

fn axis_turn(kind: PuzzleKind, wide_face: Face, back_face: Face) -> Vec<u16> {
    let n = kind.size() as u8;
    perm_of(
        kind,
        &[
            Move::new(wide_face, n - 1, n - 1 > 1, Turn::Clockwise),
            Move::new(back_face, 1, false, Turn::CounterClockwise),
        ],
    )
}

pub(super) fn compose(first: &[u16], then: &[u16]) -> Vec<u16> {
    first.iter().map(|&i| then[i as usize]).collect()
}

pub(super) fn inverse(perm: &[u16]) -> Vec<u16> {
    let mut out = vec![0u16; perm.len()];
    for (i, &p) in perm.iter().enumerate() {
        out[p as usize] = i as u16;
    }
    out
}

/// All 24 rotations, the identity first.
pub(super) fn rotations(kind: PuzzleKind) -> Vec<Rotation> {
    let x = axis_turn(kind, Face::R, Face::L);
    let y = axis_turn(kind, Face::U, Face::D);
    let identity: Vec<u16> = (0..kind.facelet_count() as u16).collect();
    let mut found = vec![(identity, Vec::<char>::new())];
    let mut at = 0;
    while at < found.len() {
        for (perm, letter) in [(&x, 'x'), (&y, 'y')] {
            let next = compose(&found[at].0, perm);
            if found.iter().all(|(p, _)| *p != next) {
                let mut path = found[at].1.clone();
                path.push(letter);
                found.push((next, path));
            }
        }
        at += 1;
    }
    found
        .into_iter()
        .map(|(perm, path)| Rotation {
            perm,
            name: name_of(&path),
        })
        .collect()
}

fn name_of(path: &[char]) -> String {
    let mut parts = Vec::new();
    let mut i = 0;
    while i < path.len() {
        let run = path[i..].iter().take_while(|&&c| c == path[i]).count();
        let suffix = match run % 4 {
            1 => "",
            2 => "2",
            _ => "'",
        };
        parts.push(format!("{}{suffix}", path[i]));
        i += run;
    }
    parts.join(" ")
}

/// Whether two colorings are the same up to renaming the colors.
pub(super) fn same_up_to_colors(a: &[u8], b: &[u8]) -> bool {
    let (mut ab, mut ba) = ([u8::MAX; 6], [u8::MAX; 6]);
    a.iter().zip(b).all(|(&x, &y)| {
        let (x, y) = (x as usize, y as usize);
        if ab[x] == u8::MAX && ba[y] == u8::MAX {
            ab[x] = y as u8;
            ba[y] = x as u8;
        }
        ab[x] as usize == y && ba[y] as usize == x
    })
}

/// How many stickers differ at least, over every renaming of the colors.
pub(super) fn mismatches(a: &[u8], b: &[u8], renamings: &[[u8; 6]]) -> usize {
    let mut table = [[0usize; 6]; 6];
    for (&x, &y) in a.iter().zip(b) {
        table[x as usize][y as usize] += 1;
    }
    let best = renamings
        .iter()
        .map(|r| (0..6).map(|c| table[c][r[c] as usize]).sum::<usize>())
        .max()
        .unwrap_or(0);
    a.len() - best
}

/// The 720 ways to rename six colors.
pub(super) fn renamings() -> Vec<[u8; 6]> {
    fn go(prefix: &mut Vec<u8>, out: &mut Vec<[u8; 6]>) {
        if prefix.len() == 6 {
            out.push(prefix.as_slice().try_into().expect("six colors"));
            return;
        }
        for c in 0..6u8 {
            if !prefix.contains(&c) {
                prefix.push(c);
                go(prefix, out);
                prefix.pop();
            }
        }
    }
    let mut out = Vec::with_capacity(720);
    go(&mut Vec::new(), &mut out);
    out
}
