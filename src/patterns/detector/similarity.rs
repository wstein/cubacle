//! The library patterns a state is closest to, and the orbits in which it
//! equals one of them.
//!
//! The similarity to a pattern is the share of stickers that agree after the
//! best of the 48 symmetries and the best renaming of the six colors, so it
//! counts pieces that are equal in place and orientation. A cube that is
//! nothing like the pattern still agrees on a large share of its stickers
//! (a random cube about a third with the solved cube), so the number says
//! "closer", not "nearly the same".

use super::center_names::{center_classes, pair_up};
use super::geometry::tables;
use super::library::{entries, Entry};
use super::pieces::layout;
use crate::puzzle::PuzzleKind;

/// A library pattern and how much of the state agrees with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Similar {
    pub id: String,
    pub name: String,
    /// Percent of the stickers that agree, 0 to 100.
    pub percent: u8,
    /// How many blocks of extra moves a repair search found between the
    /// state and this pattern; `None` unless a search found one. The
    /// percentage counts equal stickers and is not a distance.
    pub blocks_away: Option<usize>,
}

/// The orbits in which the state equals the closest patterns, and those in
/// which it differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partial {
    pub id: String,
    pub name: String,
    pub same: Vec<String>,
    pub differ: Vec<String>,
}

fn color_permutations() -> &'static [[u8; 6]] {
    static ALL: std::sync::OnceLock<Vec<[u8; 6]>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let mut out = Vec::new();
        let mut p = [0u8, 1, 2, 3, 4, 5];
        fn go(k: usize, p: &mut [u8; 6], out: &mut Vec<[u8; 6]>) {
            if k == 6 {
                out.push(*p);
                return;
            }
            for i in k..6 {
                p.swap(k, i);
                go(k + 1, p, out);
                p.swap(k, i);
            }
        }
        go(0, &mut p, &mut out);
        out
    })
}

/// The agreeing stickers under the best symmetry and renaming, with that
/// symmetry and renaming: `state[perm[i]] == map[pattern[i]]`.
fn best_agreement(kind: PuzzleKind, state: &[u8], pattern: &[u8]) -> (usize, usize, [u8; 6]) {
    let tables = tables(kind);
    let mut best = (0, 0, [0u8; 6]);
    for g in 0..48 {
        let mut count = [[0usize; 6]; 6];
        for (i, &to) in tables.perm[g].iter().enumerate() {
            count[pattern[i] as usize][state[to as usize] as usize] += 1;
        }
        for map in color_permutations() {
            let score: usize = (0..6).map(|c| count[c][map[c] as usize]).sum();
            if score > best.0 {
                best = (score, g, *map);
            }
        }
    }
    best
}

fn orbit_sets(kind: PuzzleKind) -> Vec<(String, Vec<usize>)> {
    let layout = layout(kind);
    let odd = kind.size() % 2 == 1;
    let mut sets = vec![(
        "corners".to_owned(),
        layout.corners.iter().flatten().copied().collect::<Vec<_>>(),
    )];
    let mut depths: Vec<usize> = layout.wings.iter().map(|w| w.depth).collect();
    depths.sort_unstable();
    depths.dedup();
    for depth in depths {
        let wings: Vec<_> = layout.wings.iter().filter(|w| w.depth == depth).collect();
        let name = if odd && wings.iter().all(|w| w.middle) {
            "edges".to_owned()
        } else {
            format!("wings (depth {depth})")
        };
        sets.push((name, wings.iter().flat_map(|w| w.stickers).collect()));
    }
    let mut names: Vec<String> = center_classes(kind).into_iter().map(|c| c.name).collect();
    pair_up(names.iter_mut());
    for (name, orbit) in names.into_iter().zip(&layout.centers) {
        sets.push((name, orbit.clone()));
    }
    sets
}

/// The closest `count` library patterns (the solved cube included), and the
/// orbit-wise comparison with the first of them that has one.
pub(crate) fn closest(
    kind: PuzzleKind,
    state: &[u8],
    extra: &[Entry],
    count: usize,
) -> (Vec<Similar>, Option<Partial>) {
    let tables = tables(kind);
    let total = state.len();
    let mut scored: Vec<(usize, usize, [u8; 6], &Entry)> = entries(kind)
        .iter()
        .chain(extra)
        .map(|e| {
            let (score, g, map) = best_agreement(kind, state, &e.state);
            (score, g, map, e)
        })
        .collect();
    scored.sort_by_key(|s| std::cmp::Reverse(s.0));
    scored.truncate(count);
    let similar = scored
        .iter()
        .map(|(score, _, _, e)| Similar {
            id: e.id.clone(),
            name: e.name.clone(),
            percent: (100.0 * *score as f64 / total as f64).round() as u8,
            blocks_away: None,
        })
        .collect();
    let sets = orbit_sets(kind);
    let partial = scored.iter().find_map(|(score, g, map, e)| {
        if *score == total {
            return None;
        }
        let equal = |set: &[usize]| {
            set.iter()
                .all(|&i| state[tables.perm[*g][i] as usize] == map[e.state[i] as usize])
        };
        let (same, differ): (Vec<_>, Vec<_>) = sets.iter().partition(|(_, set)| equal(set));
        (!same.is_empty() && !differ.is_empty()).then(|| Partial {
            id: e.id.clone(),
            name: e.name.clone(),
            same: same.into_iter().map(|(n, _)| n.clone()).collect(),
            differ: differ.into_iter().map(|(n, _)| n.clone()).collect(),
        })
    });
    (similar, partial)
}
