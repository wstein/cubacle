//! Naming a state by looking its canonical form up in the pattern library.
//!
//! Every pattern of the library, for every size it is defined for, and the
//! solved cube are put into canonical form once. A state is that pattern when
//! the forms are equal; the symmetry and renaming that carry one onto the
//! other are then found directly, to say how the two differ.

use std::sync::OnceLock;

use super::canon::canonical;
use super::colours;
use super::geometry::{symmetries, tables};
use super::symmetry::colour_map;
use crate::patterns::{self, Pattern};
use crate::puzzle::{Facelets, PuzzleKind};

/// A library pattern that a state is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The id in the pattern library, `solved` for the solved cube.
    pub id: String,
    pub name: String,
    /// How the state differs from the library's colouring: `exactly`,
    /// `turned`, `mirrored`, or `with colours renamed` (and turned or mirrored).
    pub relation: String,
}

pub(super) struct Entry {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) state: Vec<u8>,
    canon: Vec<u8>,
}

fn build(kind: PuzzleKind, list: Vec<(String, String, Facelets)>) -> Vec<Entry> {
    list.into_iter()
        .map(|(id, name, cube)| {
            let state = colours(&cube);
            Entry {
                id,
                name,
                canon: canonical(kind, &state),
                state,
            }
        })
        .collect()
}

/// The solved cube and the built-in patterns of a size, built once.
pub(super) fn entries(kind: PuzzleKind) -> &'static [Entry] {
    static CELLS: [OnceLock<Vec<Entry>>; 6] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    CELLS[kind.size() - 2].get_or_init(|| {
        let mut list = vec![(
            "solved".to_owned(),
            "Solved cube".to_owned(),
            Facelets::solved(kind),
        )];
        for pattern in patterns::all() {
            if let Some(state) = pattern.state(kind) {
                list.push((pattern.id.clone(), pattern.name.clone(), state));
            }
        }
        build(kind, list)
    })
}

/// The entries of imported patterns that are defined for a size; the id is
/// the catalogue id.
pub(super) fn extra_entries(kind: PuzzleKind, extra: &[Pattern]) -> Vec<Entry> {
    let list = extra
        .iter()
        .filter_map(|p| {
            p.state(kind)
                .map(|state| (p.catalogue.clone(), p.name.clone(), state))
        })
        .collect();
    build(kind, list)
}

/// How `state` differs from `pattern`, the plainest way found.
fn relation(kind: PuzzleKind, state: &[u8], pattern: &[u8]) -> String {
    let tables = tables(kind);
    let best = (0..symmetries().len())
        .filter_map(|g| colour_map(state, pattern, &tables.perm[g]).map(|map| (g, map)))
        .min_by_key(|(g, map)| {
            let renamed = map.iter().enumerate().any(|(c, &to)| c != to as usize);
            (renamed, *g != 0, symmetries()[*g].is_mirror())
        });
    let Some((g, map)) = best else {
        return "equivalent".to_owned();
    };
    let renamed = map.iter().enumerate().any(|(c, &to)| c != to as usize);
    let how = match (g == 0, symmetries()[g].is_mirror()) {
        (true, _) => "",
        (false, false) => "turned",
        (false, true) => "mirrored",
    };
    match (renamed, how) {
        (false, "") => "exactly".to_owned(),
        (false, how) => how.to_owned(),
        (true, "") => "with colours renamed".to_owned(),
        (true, how) => format!("{how}, with colours renamed"),
    }
}

/// The library patterns that `state` is, given its canonical form.
pub(crate) fn matches(kind: PuzzleKind, state: &[u8], canon: &[u8], extra: &[Entry]) -> Vec<Match> {
    entries(kind)
        .iter()
        .chain(extra)
        .filter(|entry| entry.canon == canon)
        .map(|entry| Match {
            id: entry.id.clone(),
            name: entry.name.clone(),
            relation: relation(kind, state, &entry.state),
        })
        .collect()
}
