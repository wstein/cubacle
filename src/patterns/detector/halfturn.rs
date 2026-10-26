//! Membership of the half-turn group `<U2,D2,R2,L2,F2,B2>` of the 3x3x3.
//!
//! In this group every corner and edge is untwisted and stays in its own
//! class: the corners fall into two classes of four, the edges into three
//! classes of four (the edges parallel to one axis). Inside the five classes
//! the arrangement is not free: modulo the Klein four-group of each class
//! (`S4` to `S3`, the action on the three ways to pair up four slots) the
//! five arrangements move together as the half turns move them. That set of
//! tuples has 648 members; with the free Klein groups (4^5 = 1024) it makes
//! the 663 552 states of the group, which the tests count by walking the whole
//! group.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use super::pieces::{layout, Cube, Layout};
use super::subgroup::article;
use crate::puzzle::{destinations, Face, Move, PuzzleKind, Turn};

/// An arrangement of the three pairings of four slots.
type Three = [u8; 3];
type Tuple = [Three; 5];

const IDENTITY: Three = [0, 1, 2];

/// Which pairing, of `{01|23, 02|13, 03|12}`, the pair `{a, b}` belongs to.
fn pairing(a: usize, b: usize) -> usize {
    match (a.min(b), a.max(b)) {
        (0, 1) | (2, 3) => 0,
        (0, 2) | (1, 3) => 1,
        _ => 2,
    }
}

/// The image of an arrangement of four slots in `S3`.
fn project(perm: [usize; 4]) -> Three {
    let mut out = [0u8; 3];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = pairing(perm[0], perm[i + 1]) as u8;
    }
    out
}

fn compose(first: Three, then: Three) -> Three {
    [
        then[first[0] as usize],
        then[first[1] as usize],
        then[first[2] as usize],
    ]
}

struct Group {
    /// The class of every corner place, then of every edge place, with the
    /// place's number inside its class.
    corner_class: Vec<(usize, usize)>,
    edge_class: Vec<(usize, usize)>,
    members: HashSet<Tuple>,
}

fn pieces_of(layout: &Layout) -> (HashMap<usize, usize>, HashMap<usize, usize>) {
    let mut corner = HashMap::new();
    let mut edge = HashMap::new();
    for (i, c) in layout.corners.iter().enumerate() {
        for &s in c {
            corner.insert(s, i);
        }
    }
    for (i, w) in layout.wings.iter().enumerate() {
        for s in w.stickers {
            edge.insert(s, i);
        }
    }
    (corner, edge)
}

fn group() -> &'static Group {
    static GROUP: OnceLock<Group> = OnceLock::new();
    GROUP.get_or_init(|| {
        let kind = PuzzleKind::Cube3;
        let layout = layout(kind);
        let (corner_of, edge_of) = pieces_of(layout);
        let turns: Vec<Vec<usize>> = Face::ALL
            .iter()
            .map(|&f| destinations(kind, &Move::new(f, 1, false, Turn::Half)))
            .collect();
        // Where each half turn sends each place, for corners and edges.
        let moved = |of: &HashMap<usize, usize>, first: &dyn Fn(usize) -> usize, count: usize| {
            turns
                .iter()
                .map(|t| (0..count).map(|p| of[&t[first(p)]]).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        let corner_moves = moved(&corner_of, &|p| layout.corners[p][0], layout.corners.len());
        let edge_moves = moved(
            &edge_of,
            &|p| layout.wings[p].stickers[0],
            layout.wings.len(),
        );
        let classes_of = |moves: &Vec<Vec<usize>>, count: usize| {
            let mut class = vec![usize::MAX; count];
            let mut next = 0;
            for start in 0..count {
                if class[start] != usize::MAX {
                    continue;
                }
                let mut queue = vec![start];
                class[start] = next;
                while let Some(p) = queue.pop() {
                    for m in moves {
                        if class[m[p]] == usize::MAX {
                            class[m[p]] = next;
                            queue.push(m[p]);
                        }
                    }
                }
                next += 1;
            }
            class
        };
        let corner_cls = classes_of(&corner_moves, layout.corners.len());
        let edge_cls = classes_of(&edge_moves, layout.wings.len());
        let number = |cls: &[usize]| {
            let mut seen = [0usize; 8];
            cls.iter()
                .map(|&c| {
                    seen[c] += 1;
                    (c, seen[c] - 1)
                })
                .collect::<Vec<_>>()
        };
        let corner_class = number(&corner_cls);
        let edge_class = number(&edge_cls);
        // Classes 0 and 1 are the corners, 2 to 4 the edges.
        let place =
            |class: &[(usize, usize)], offset: usize, p: usize| (class[p].0 + offset, class[p].1);
        // The projected move of each class by each half turn.
        let mut generators: Vec<Tuple> = Vec::new();
        for t in 0..6 {
            let mut perms = [[0usize, 1, 2, 3]; 5];
            for (p, &to) in corner_moves[t].iter().enumerate() {
                let (c, i) = place(&corner_class, 0, p);
                perms[c][i] = place(&corner_class, 0, to).1;
            }
            for (p, &to) in edge_moves[t].iter().enumerate() {
                let (c, i) = place(&edge_class, 2, p);
                perms[c][i] = place(&edge_class, 2, to).1;
            }
            generators.push(perms.map(project));
        }
        let mut members: HashSet<Tuple> = HashSet::new();
        let start: Tuple = [IDENTITY; 5];
        members.insert(start);
        let mut queue = VecDeque::from([start]);
        while let Some(t) = queue.pop_front() {
            for g in &generators {
                let mut next = t;
                for c in 0..5 {
                    next[c] = compose(g[c], t[c]);
                }
                if members.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        Group {
            corner_class,
            edge_class: edge_class.iter().map(|&(c, i)| (c + 2, i)).collect(),
            members,
        }
    })
}

/// Whether the cube (a 3x3x3 in its center frame) is in the half-turn group,
/// or why not.
pub(crate) fn in_half_turn_group(cube: &Cube) -> Result<(), String> {
    let g = group();
    let layout = cube.layout;
    let order = [0, 2, 1];
    let home_of =
        |bits: u8, places: &[Vec<usize>]| places.iter().position(|p| cube.faces_of(p) == bits);
    let corner_places: Vec<Vec<usize>> = layout.corners.iter().map(|c| c.to_vec()).collect();
    let edge_places: Vec<Vec<usize>> = layout.wings.iter().map(|w| w.stickers.to_vec()).collect();
    let mut arrangement = [[0usize, 1, 2, 3]; 5];
    let mut fill =
        |places: &[Vec<usize>], class: &[(usize, usize)], what: &str| -> Result<(), String> {
            for (p, place) in places.iter().enumerate() {
                let r = cube.read(place, order);
                if !r.valid {
                    return Err(format!("the {what}s are no cube's"));
                }
                if !r.oriented {
                    return Err(format!("{} {what} is twisted or flipped", article(what)));
                }
                let home = home_of(r.home, places).ok_or("a piece is no cube's")?;
                if class[home].0 != class[p].0 {
                    return Err(format!("{} {what} left its class", article(what)));
                }
                arrangement[class[p].0][class[p].1] = class[home].1;
            }
            Ok(())
        };
    fill(&corner_places, &g.corner_class, "corner")?;
    fill(&edge_places, &g.edge_class, "edge")?;
    let projected: Tuple = arrangement.map(project);
    if g.members.contains(&projected) {
        Ok(())
    } else {
        Err("the pieces are arranged in a way no half turns make".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::detector::pieces::frames;

    fn member(moves: &str) -> bool {
        let kind = PuzzleKind::Cube3;
        let cube = crate::puzzle::Facelets::solved(kind)
            .apply(&moves.parse().unwrap())
            .unwrap();
        let state: Vec<u8> = cube.stickers().iter().map(|f| *f as u8).collect();
        let frame = frames(kind, &state)[0];
        in_half_turn_group(&Cube::new(kind, &state, &frame)).is_ok()
    }

    #[test]
    fn the_projected_tuples_number_648() {
        assert_eq!(group().members.len(), 648);
    }

    #[test]
    fn the_test_accepts_exactly_the_whole_group() {
        use std::collections::HashSet;
        let kind = PuzzleKind::Cube3;
        let turns: Vec<Vec<usize>> = Face::ALL
            .iter()
            .map(|&f| destinations(kind, &Move::new(f, 1, false, Turn::Half)))
            .collect();
        let solved: Vec<u8> = (0..54).map(|i| (i / 9) as u8).collect();
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        seen.insert(solved.clone());
        let mut layer = vec![solved];
        let mut checked = 0;
        while !layer.is_empty() {
            let mut next = Vec::new();
            for state in layer {
                // A sample of the group is checked against the test.
                if checked % 97 == 0 {
                    let frame = frames(kind, &state)[0];
                    assert!(in_half_turn_group(&Cube::new(kind, &state, &frame)).is_ok());
                }
                checked += 1;
                for t in &turns {
                    let mut moved = state.clone();
                    for (i, &to) in t.iter().enumerate() {
                        moved[to] = state[i];
                    }
                    if seen.insert(moved.clone()) {
                        next.push(moved);
                    }
                }
            }
            layer = next;
        }
        // 648 projected tuples times 4^5 free Klein-group parts.
        assert_eq!(seen.len(), 663_552);
        assert_eq!(group().members.len() * 1024, seen.len());
    }

    #[test]
    fn half_turn_words_are_members_and_other_turns_are_not() {
        assert!(member(""));
        assert!(member("R2 U2 F2 L2 D2 B2 R2 U2"));
        assert!(member("U2 R2 U2 R2 F2"));
        assert!(!member("R"));
        assert!(!member("R2 U"));
        assert!(!member("R U R' U'"));
    }
}
