//! The sticker permutation of a cube: where every sticker of the solved cube
//! has gone.
//!
//! From a move sequence it is exact. From colours alone it is exact only
//! where pieces can be told apart (every corner, and every edge and centre of
//! the 2x2x2 and 3x3x3); pieces that look alike are matched to the nearest
//! free place in index order, which reproduces the colours but is one of
//! several permutations that do.

use std::collections::HashMap;

use super::pieces::{frames, layout, Cube};
use crate::puzzle::{destinations, Facelets, MoveSequence, PuzzleKind};

/// `pi[i]` is the sticker place that sticker `i` of the solved cube is at.
pub(crate) type Permutation = Vec<usize>;

/// The permutation a move sequence makes; `None` if a move does not fit.
pub(crate) fn of_moves(kind: PuzzleKind, moves: &MoveSequence) -> Option<Permutation> {
    if moves.moves().iter().any(|mv| !mv.fits(kind)) {
        return None;
    }
    let mut at: Vec<usize> = (0..kind.facelet_count()).collect();
    for mv in moves.moves() {
        let to = destinations(kind, mv);
        for place in &mut at {
            *place = to[*place];
        }
    }
    Some(at)
}

/// Which stickers sit on corners or wings.
pub(crate) fn covered(kind: PuzzleKind) -> Vec<bool> {
    let layout = layout(kind);
    let mut covered = vec![false; kind.facelet_count()];
    for s in layout.corners.iter().flatten() {
        covered[*s] = true;
    }
    for w in &layout.wings {
        for s in w.stickers {
            covered[s] = true;
        }
    }
    covered
}

/// The centre sticker groups: each orbit of the movable ones, then the face
/// middles of an odd cube.
pub(crate) fn centre_groups(kind: PuzzleKind) -> Vec<Vec<usize>> {
    let layout = layout(kind);
    let covered = covered(kind);
    let mut groups: Vec<Vec<usize>> = layout.centres.clone();
    let used: Vec<usize> = groups.iter().flatten().copied().collect();
    let rest: Vec<usize> = (0..kind.facelet_count())
        .filter(|&s| !covered[s] && !used.contains(&s))
        .collect();
    if !rest.is_empty() {
        groups.push(rest);
    }
    groups
}

/// A frame for a cube without fixed centres: the piece at the D, B, L corner
/// is taken to be home, and the colours it never shares a corner with are the
/// opposite faces.
fn corner_frame(kind: PuzzleKind, state: &[u8]) -> Option<[u8; 6]> {
    let layout = layout(kind);
    let face_len = layout.face_len;
    let mut meets = [[false; 6]; 6];
    for corner in &layout.corners {
        for &a in corner {
            for &b in corner {
                meets[state[a] as usize][state[b] as usize] = true;
            }
        }
    }
    let opposite = |c: usize| (0..6).find(|&o| !meets[c][o]);
    let home = layout.corners.iter().find(|c| {
        let mut faces: Vec<usize> = c.iter().map(|s| s / face_len).collect();
        faces.sort_unstable();
        faces == [3, 4, 5]
    })?;
    let mut frame = [u8::MAX; 6];
    for &s in home {
        let (colour, face) = (state[s] as usize, s / face_len);
        frame[colour] = face as u8;
        frame[opposite(colour)?] = ((face + 3) % 6) as u8;
    }
    let mut seen = [false; 6];
    for f in frame {
        if f == u8::MAX || std::mem::replace(&mut seen[f as usize], true) {
            return None;
        }
    }
    Some(frame)
}

/// The permutation of a colour state, read in its best colour frame. `Err`
/// when the pieces are no cube's.
pub(crate) fn of_state(cube: &Facelets) -> Result<Permutation, String> {
    let kind = cube.kind();
    let state: Vec<u8> = cube.stickers().iter().map(|f| *f as u8).collect();
    let frame = if kind.size() % 2 == 1 {
        frames(kind, &state).into_iter().next()
    } else {
        corner_frame(kind, &state)
    }
    .ok_or("the colours are no cube's")?;
    let lab = Cube::new(kind, &state, &frame).lab;
    let layout = layout(kind);
    let face = |s: usize| s / layout.face_len;
    let mut pi = vec![usize::MAX; kind.facelet_count()];
    let bits = |list: &[usize], f: &dyn Fn(usize) -> usize| {
        list.iter().fold(0u8, |acc, &s| acc | 1 << f(s))
    };
    let colour = |s: usize| lab[s] as usize;

    // Corners: a corner is the set of its colours.
    let mut corner_home: HashMap<u8, usize> = HashMap::new();
    for (h, c) in layout.corners.iter().enumerate() {
        corner_home.insert(bits(c, &face), h);
    }
    let mut taken = vec![false; layout.corners.len()];
    for slot in &layout.corners {
        let h = *corner_home
            .get(&bits(slot, &colour))
            .ok_or("the corners are no cube's")?;
        if std::mem::replace(&mut taken[h], true) {
            return Err("a corner appears twice".to_owned());
        }
        for &j in slot {
            let home = layout.corners[h]
                .iter()
                .find(|&&s| face(s) == colour(j))
                .ok_or("the corners are no cube's")?;
            pi[*home] = j;
        }
    }

    // Wings: the free homes of a colour pair and depth, fixed ones first.
    let mut free: HashMap<(u8, usize), Vec<usize>> = HashMap::new();
    for (h, w) in layout.wings.iter().enumerate() {
        free.entry((bits(&w.stickers, &face), w.depth))
            .or_default()
            .push(h);
    }
    let mut home_of = vec![usize::MAX; layout.wings.len()];
    for (w, wing) in layout.wings.iter().enumerate() {
        let key = (bits(&wing.stickers, &colour), wing.depth);
        if key.0 == bits(&wing.stickers, &face) {
            if let Some(list) = free.get_mut(&key) {
                if let Some(at) = list.iter().position(|&h| h == w) {
                    list.remove(at);
                    home_of[w] = w;
                }
            }
        }
    }
    for (w, wing) in layout.wings.iter().enumerate() {
        if home_of[w] != usize::MAX {
            continue;
        }
        let key = (bits(&wing.stickers, &colour), wing.depth);
        let list = free.get_mut(&key).ok_or("the edges are no cube's")?;
        if list.is_empty() {
            return Err("the edges are no cube's".to_owned());
        }
        home_of[w] = list.remove(0);
    }
    for (w, wing) in layout.wings.iter().enumerate() {
        for &j in &wing.stickers {
            let home = layout.wings[home_of[w]]
                .stickers
                .iter()
                .find(|&&s| face(s) == colour(j))
                .ok_or("the edges are no cube's")?;
            pi[*home] = j;
        }
    }

    // Centres: a sticker keeps its place if it can, then the nearest free one
    // of its colour in its orbit.
    for group in centre_groups(kind) {
        let mut free: Vec<usize> = group.clone();
        let mut placed = vec![usize::MAX; kind.facelet_count()];
        for &j in &group {
            if colour(j) == face(j) {
                free.retain(|&h| h != j);
                placed[j] = j;
            }
        }
        for &j in &group {
            if placed[j] != usize::MAX {
                continue;
            }
            let at = free
                .iter()
                .position(|&h| face(h) == colour(j))
                .ok_or("the centres are no cube's")?;
            placed[j] = free.remove(at);
        }
        for &j in &group {
            pi[placed[j]] = j;
        }
    }
    if pi.contains(&usize::MAX) {
        return Err("the pieces are no cube's".to_owned());
    }
    Ok(pi)
}
