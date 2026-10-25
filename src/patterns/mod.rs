//! Well-known cube patterns ("pretty patterns") for the 3x3x3 to 7x7x7, each
//! defined by its structure and paired with a generator found by our own search.
//!
//! A pattern is a colour for every sticker as a function of the cube size
//! (see `model.rs`); a generator is a move sequence that, applied to the
//! solved cube, produces that state. The data lives in two plain text files
//! next to this module (`patterns.txt`, `generators.txt`), format in
//! `parse.rs`. Generators are searched by `examples/pattern_search.rs`.
//!
//! ```
//! use cubacle::patterns;
//! use cubacle::puzzle::{Facelets, PuzzleKind};
//!
//! let pattern = patterns::find("chessboard").expect("known alias");
//! let generator = pattern.generator(3, None).expect("a 3x3x3 generator");
//! let cube = Facelets::solved(PuzzleKind::Cube3).apply(&generator.sequence().unwrap()).unwrap();
//! assert_eq!(Some(cube), pattern.state(PuzzleKind::Cube3));
//! ```

mod bases;
mod detector;
mod groups;
mod import;
mod imported_file;
mod model;
mod parse;

pub use bases::{bases, Base};
pub use detector::{
    cycles_of_moves, cycles_of_state, detect, kind, measures, patternness, Cycle, Cycles, Detailed,
    Fingerprint, Match, Measures, Membership, Operation, OrbitCycles, OrbitStat, Order, Partial,
    PatternKind, Report, Similar, Symmetry, Verdict, PATTERNNESS_VERSION,
};
pub use import::{
    decode_text, import_tab, read_plain, ImportOptions, ImportProblem, ImportReport, ImportRow,
    Imported,
};
pub use imported_file::{
    identify_imported, imported_is_stale, looks_imported, read_imported, write_imported,
};

use std::sync::OnceLock;

use crate::puzzle::{Face, Facelets, LayerMove, MoveSequence, PuzzleKind, Turn};
use model::Def;

/// What one move is in a generator's length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    /// Outer-block turns: `R`, `Rw`, `3Rw` (a block from one face). A slice
    /// needs two of them.
    Outer,
    /// Any block of adjacent layers is one move, written `2-3Rw`.
    Range,
}

impl Span {
    pub fn name(self) -> &'static str {
        match self {
            Span::Outer => "outer",
            Span::Range => "range",
        }
    }

    fn parse(text: &str) -> Option<Span> {
        match text {
            "outer" => Some(Span::Outer),
            "range" => Some(Span::Range),
            _ => None,
        }
    }
}

/// How far a generator is known to be shortest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Exhaustive search over quarter and half turns found nothing shorter.
    Optimal,
    /// Exhaustive over half turns only; quarter turns were not searched at that length.
    HalfTurnOptimal,
    /// The best found within the search budget; shorter ones may exist.
    BestKnown,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Optimal => "optimal",
            Status::HalfTurnOptimal => "half-turn-optimal",
            Status::BestKnown => "best-known",
        }
    }

    fn parse(text: &str) -> Option<Status> {
        match text {
            "optimal" => Some(Status::Optimal),
            "half-turn-optimal" => Some(Status::HalfTurnOptimal),
            "best-known" => Some(Status::BestKnown),
            _ => None,
        }
    }
}

/// A move sequence that builds a pattern from the solved cube.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generator {
    pub size: usize,
    pub metric: Span,
    pub status: Status,
    /// Number of moves in `metric`.
    pub length: usize,
    /// Running number among the generators of the pattern (`id#n`), from 1.
    pub number: usize,
    /// WCA text; ranges (`2-3Rw`) in the `Range` metric.
    pub moves: String,
}

impl Generator {
    /// The plain moves, with ranges expanded.
    pub fn sequence(&self) -> Result<MoveSequence, String> {
        let mut out = Vec::new();
        for token in self.moves.split_whitespace() {
            match parse_range(token) {
                Some(mv) => out.extend(mv.to_moves()),
                None => out.push(token.parse().map_err(|_| format!("bad move {token:?}"))?),
            }
        }
        Ok(MoveSequence::new(out))
    }
}

/// `2-3Rw'` as a layer block, `None` for any other token.
fn parse_range(token: &str) -> Option<LayerMove> {
    let (from, rest) = token.split_once('-')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    let (to, rest) = rest.split_at(digits);
    let mut chars = rest.chars();
    let face = Face::from_letter(chars.next()?)?;
    let suffix = chars.as_str().strip_prefix('w')?;
    Some(LayerMove::new(
        face,
        from.parse().ok()?,
        to.parse().ok()?,
        Turn::from_suffix(suffix)?,
    ))
}

/// One level of the category tree, with its name in both languages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub de: String,
    pub en: String,
}

/// A named pattern with its definitions and generators.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    /// Catalogue id, at most 12 characters: `cu:<code>` for the built-in library, `<source>:<code>`
    /// for imported tables (for example `ct:A200.01`).
    pub catalogue: String,
    /// The place in the tree of categories, outermost first (up to three levels).
    pub group: Vec<Group>,
    /// Author and year, as the source gives them.
    pub provenance: String,
    /// Where an imported pattern came from; `None` for the built-in library.
    pub origin: Option<String>,
    /// Patternness per size as stored with an imported pattern.
    scores: Vec<(usize, u8)>,
    defs: Vec<(usize, Def)>,
    generators: Vec<Generator>,
}

impl Pattern {
    fn new(id: &str) -> Pattern {
        Pattern {
            id: id.to_owned(),
            name: id.to_owned(),
            aliases: Vec::new(),
            description: String::new(),
            catalogue: format!("cu:{id}"),
            group: Vec::new(),
            provenance: String::new(),
            origin: None,
            scores: Vec::new(),
            defs: Vec::new(),
            generators: Vec::new(),
        }
    }

    /// The patternness of the pattern on a cube of `size`: the value stored
    /// with an imported pattern, else computed from its state.
    pub fn patternness(&self, size: usize) -> Option<u8> {
        if let Some((_, score)) = self.scores.iter().find(|(s, _)| *s == size) {
            return Some(*score);
        }
        let state = self.state(PuzzleKind::of_size(size)?)?;
        Some(patternness(&state))
    }

    /// The sizes the pattern is defined for.
    pub fn sizes(&self) -> Vec<usize> {
        self.defs.iter().map(|(size, _)| *size).collect()
    }

    /// The pattern as a cube state, `None` if it is not defined for that size.
    pub fn state(&self, kind: PuzzleKind) -> Option<Facelets> {
        let Some((_, def)) = self.defs.iter().find(|(size, _)| *size == kind.size()) else {
            // No structural definition (an imported pattern): replay its generator.
            let sequence = self.generator(kind.size(), None)?.sequence().ok()?;
            return Facelets::solved(kind).apply(&sequence).ok();
        };
        Some(def.state(kind))
    }

    /// Every generator of the pattern.
    pub fn generators(&self) -> &[Generator] {
        &self.generators
    }

    /// The shortest generator for a size, in `metric` or (None) in either metric.
    /// Every outer-block sequence is also a range sequence, so a `Range`
    /// request can be answered by an `Outer` generator.
    pub fn generator(&self, size: usize, metric: Option<Span>) -> Option<&Generator> {
        self.generators
            .iter()
            .filter(|g| g.size == size && metric.is_none_or(|m| g.metric == m || m == Span::Range))
            .min_by_key(|g| g.length)
    }

    /// Whether `key` is the id, name or an alias (case, spaces and hyphens ignored).
    pub fn is_named(&self, key: &str) -> bool {
        self.matches(key)
    }

    fn matches(&self, key: &str) -> bool {
        let key = squash(key);
        squash(&self.id) == key
            || squash(&self.catalogue) == key
            || squash(&self.name) == key
            || self.aliases.iter().any(|a| squash(a) == key)
    }
}

/// Case, spaces, hyphens and underscores do not matter in a name.
fn squash(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

static REGISTRY: OnceLock<Vec<Pattern>> = OnceLock::new();

/// All patterns, in file order.
pub fn all() -> &'static [Pattern] {
    REGISTRY.get_or_init(|| {
        let mut list = parse::parse_patterns(include_str!("patterns.txt"))
            .unwrap_or_else(|e| panic!("patterns.txt: {e}"));
        let generators = parse::parse_generators(include_str!("generators.txt"))
            .unwrap_or_else(|e| panic!("generators.txt: {e}"));
        for (id, mut generator) in generators {
            let pattern = list
                .iter_mut()
                .find(|p| p.id == id)
                .unwrap_or_else(|| panic!("generators.txt: unknown pattern {id:?}"));
            generator.number = pattern.generators.len() + 1;
            pattern.generators.push(generator);
        }
        for pattern in &mut list {
            pattern.group = groups::builtin_group(&pattern.id);
            pattern.catalogue = format!("cu:{}", groups::builtin_code(&pattern.id));
        }
        list
    })
}

/// The pattern with this id, name or alias (case, spaces and hyphens ignored).
pub fn find(name: &str) -> Option<&'static Pattern> {
    all().iter().find(|p| p.matches(name))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod import_tests;
