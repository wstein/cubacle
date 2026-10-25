//! Pattern kinds: the sort of regularity a state shows.
//!
//! Patternness stays the one score; the kind says *what* is regular, so a
//! listing can be filtered ("every pattern that is a few moved pieces") and a
//! reader sees why a state scored as it did. The kinds are tried in order and
//! the first that fits wins:
//!
//! | Kind | Fits when |
//! | --- | --- |
//! | `SOLVED` | no sticker differs from a turned solved cube |
//! | `FEW` | at most 3 pieces, or at most 15 %, are moved |
//! | `MOSTLY` | the fixed share is at least 0.6 and the defect is symmetric (at least 0.5) |
//! | `SYM` | the symmetry share of patternness is at least 0.9 |
//! | `FACE` | the faces repeat one arrangement, or most faces are simple |
//! | `IRREG` | none of the above |
//!
//! The symmetry and face thresholds depend on the size; they were set from the
//! rates on short scrambles (`tools/bench/kind_rates.rs`, `docs/patterns.md`).
//! A kind describes the structure of one state; it is not a category of any
//! collection.

use std::fmt;
use std::str::FromStr;

use super::measures::{measures, Measures};
use crate::puzzle::Facelets;

/// The sort of regularity of a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternKind {
    Solved,
    Few,
    Mostly,
    Sym,
    Face,
    Irreg,
}

impl PatternKind {
    /// Every kind, in the order the rule tries them.
    pub const ALL: [PatternKind; 6] = [
        PatternKind::Solved,
        PatternKind::Few,
        PatternKind::Mostly,
        PatternKind::Sym,
        PatternKind::Face,
        PatternKind::Irreg,
    ];

    /// The upper-case name: `SOLVED`, `FEW`, `MOSTLY`, `SYM`, `FACE`, `IRREG`.
    pub fn name(self) -> &'static str {
        match self {
            PatternKind::Solved => "SOLVED",
            PatternKind::Few => "FEW",
            PatternKind::Mostly => "MOSTLY",
            PatternKind::Sym => "SYM",
            PatternKind::Face => "FACE",
            PatternKind::Irreg => "IRREG",
        }
    }

    /// One line on what the kind means.
    pub fn describe(self) -> &'static str {
        match self {
            PatternKind::Solved => "a solved cube, up to turning it",
            PatternKind::Few => "a few pieces moved, the rest in place",
            PatternKind::Mostly => "mostly in place, the moved part is symmetric",
            PatternKind::Sym => "a high symmetry",
            PatternKind::Face => "the faces repeat one arrangement or are simple",
            PatternKind::Irreg => "no regularity of the kinds above",
        }
    }
}

impl fmt::Display for PatternKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for PatternKind {
    type Err = String;

    fn from_str(text: &str) -> Result<PatternKind, String> {
        PatternKind::ALL
            .into_iter()
            .find(|k| k.name().eq_ignore_ascii_case(text.trim()))
            .ok_or_else(|| {
                let names: Vec<&str> = PatternKind::ALL.iter().map(|k| k.name()).collect();
                format!("no kind {text:?} (kinds: {})", names.join(", "))
            })
    }
}

/// The size-dependent thresholds, set from the rates of
/// `tools/bench/kind_rates.rs` on short scrambles (at most about 2 % of the
/// scrambles beyond FEW and MOSTLY, where the size allows it).
struct Thresholds {
    /// Fewest symmetries for SYM (the symmetry share must be 0.9 too).
    symmetries: usize,
    /// Share of faces with one arrangement.
    faces: f64,
    /// Share of simple faces; `None` where a simple face means nothing.
    simple: Option<f64>,
}

fn thresholds(size: usize) -> Thresholds {
    match size {
        2 => Thresholds {
            symmetries: 8,
            faces: 1.0,
            simple: None,
        },
        3 => Thresholds {
            symmetries: 4,
            faces: 0.67,
            simple: Some(0.67),
        },
        _ => Thresholds {
            symmetries: 4,
            faces: 0.5,
            simple: Some(0.67),
        },
    }
}

impl Measures {
    /// The kind these measures fall under (first match, see the module docs).
    pub fn kind(&self) -> PatternKind {
        let limits = thresholds(self.size);
        if self.moved == 0 {
            PatternKind::Solved
        } else if self.moved <= 3 || self.moved_share() <= 0.15 {
            PatternKind::Few
        } else if self.fixed >= 0.6 && self.mask_symmetry >= 0.5 {
            PatternKind::Mostly
        } else if self.symmetry >= 0.9 && self.symmetry_order >= limits.symmetries {
            PatternKind::Sym
        } else if self.faces >= limits.faces
            || limits.simple.is_some_and(|t| self.simple_faces >= t)
        {
            PatternKind::Face
        } else {
            PatternKind::Irreg
        }
    }
}

/// The kind of a cube state.
pub fn kind(cube: &Facelets) -> PatternKind {
    measures(cube).kind()
}
