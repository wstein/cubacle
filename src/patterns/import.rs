//! Import of tab-separated pattern tables (the `PatternExportTab.txt` layout
//! of Werner Randelshofer's CubeTwister), written from the structure of the
//! format, not from its contents.
//!
//! One line is one variant of one pattern. Fields are separated by tabs:
//!
//! | # | field |
//! |---|-------|
//! | 1 | pattern code, such as `A200.01` |
//! | 2 | variant number: lines with the same code are alternative sequences |
//! | 3-8 | three levels of grouping (category, subgroup, sub-subgroup), each German then English; the last is the pattern's name |
//! | 9 | the move sequence |
//! | 10-13 | four counts (block turns, face turns, quarter turns, order; unverified, an `*` marks a minimum) |
//! | 14 | cycle notation of the parts (not imported) |
//! | 15-17 | author and year, notes |
//!
//! Moves are face letters with `2` for a half turn and `-` (or `'`) for the
//! inverse. The text may be UTF-8 or a legacy 8-bit encoding; the importer
//! reads the bytes as Mac Roman when UTF-8 fails (see `decode`). Every sequence is replayed on a
//! solved cube of the requested size; lines that do not parse, do not apply or
//! disagree with their sibling variants are reported and skipped.

use super::{Generator, Group, Pattern, Span, Status};
use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

/// How to read a table.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Cube size of the table (2 to 7).
    pub size: usize,
    /// Put in front of every pattern id and name.
    pub name_prefix: String,
    /// Where the data came from, shown by `patterns show`.
    pub origin: String,
    /// Catalogue tag in front of the code, for example `ct`.
    pub source: String,
}

/// One line that was not imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportProblem {
    /// 1-based line number in the file (0 for a problem with the options).
    pub line: usize,
    pub message: String,
}

/// What an import did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Non-empty lines read.
    pub lines: usize,
    /// Lines whose sequence was replayed and kept.
    pub accepted: usize,
    pub problems: Vec<ImportProblem>,
}

/// An accepted line, kept for the optional check of its claimed counts.
#[derive(Debug, Clone)]
pub struct ImportRow {
    pub line: usize,
    pub code: String,
    pub moves: MoveSequence,
    /// Fields 10 to 13 as written: block turns, face turns, quarter turns, order.
    pub claimed: [String; 4],
}

/// The patterns of a table and the report on it.
#[derive(Debug, Clone)]
pub struct Imported {
    pub patterns: Vec<Pattern>,
    pub report: ImportReport,
    pub rows: Vec<ImportRow>,
}

/// Lower-case ASCII words joined by `-`.
pub(crate) fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_owned()
}

/// His spelling to ours: `R-` and `R2-` are inverses.
fn normalise_move(token: &str) -> String {
    match token.strip_suffix('-') {
        Some(base) if base.ends_with('2') => base.to_owned(),
        Some(base) => format!("{base}'"),
        None => token.to_owned(),
    }
}

/// The plain reader: whitespace-separated face turns, `-` or `'` for the inverse.
pub fn read_plain(field: &str) -> Result<MoveSequence, String> {
    let text: Vec<String> = field.split_whitespace().map(normalise_move).collect();
    text.join(" ")
        .parse()
        .map_err(|e| format!("cannot read the moves: {e}"))
}

type Reader<'a> = &'a dyn Fn(&str) -> Result<MoveSequence, String>;

/// The moves without a trailing note. The tables put a vertical tab and then
/// the pieces the pattern moves in brackets after the moves, such as
/// `(uf,dr)` or `(+lb)`: lower-case piece names, commas, `+` and `-` only.
/// Anywhere else a vertical tab is white space between moves.
pub(crate) fn strip_note(field: &str) -> &str {
    let Some((head, tail)) = field.trim_end().rsplit_once('\u{b}') else {
        return field;
    };
    let note = tail.trim();
    let piece = |c: char| c.is_ascii_lowercase() || matches!(c, ',' | '+' | '-');
    match note.strip_prefix('(').and_then(|n| n.strip_suffix(')')) {
        Some(inner) if !head.trim().is_empty() && !inner.is_empty() && inner.chars().all(piece) => {
            head
        }
        _ => field,
    }
}

fn replay(kind: PuzzleKind, field: &str, read: Reader) -> Result<(MoveSequence, Facelets), String> {
    let field = strip_note(field);
    if field.trim().is_empty() {
        return Err("no moves".to_owned());
    }
    let sequence = read(field)?;
    let state = Facelets::solved(kind)
        .apply(&sequence)
        .map_err(|e| format!("does not apply to a {0}x{0}x{0}: {e}", kind.size()))?;
    Ok((sequence, state))
}

/// Reads a table with `read` for the move field. Never aborts: bad lines land
/// in the report.
pub fn import_tab(text: &str, options: &ImportOptions, read: Reader) -> Imported {
    let mut report = ImportReport::default();
    let mut rows = Vec::new();
    if !super::groups::valid_source(&options.source) {
        report.problems.push(ImportProblem {
            line: 0,
            message: format!(
                "source tag {:?} is not two lowercase letters",
                options.source
            ),
        });
        return Imported {
            patterns: Vec::new(),
            report,
            rows,
        };
    }
    let Some(kind) = PuzzleKind::of_size(options.size) else {
        report.problems.push(ImportProblem {
            line: 0,
            message: format!("no cube of size {}", options.size),
        });
        return Imported {
            patterns: Vec::new(),
            report,
            rows,
        };
    };
    // Each pattern with the state of its first generator.
    let mut patterns: Vec<(Pattern, Facelets)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let raw = raw.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
        if raw.is_empty() {
            continue;
        }
        report.lines += 1;
        let fields: Vec<&str> = raw.split('\t').map(str::trim).collect();
        let mut problem = |message: String| report.problems.push(ImportProblem { line, message });
        if fields.len() < 9 {
            problem(format!("{} fields, at least 9 expected", fields.len()));
            continue;
        }
        let code = fields[0];
        if !super::groups::valid_code(code) {
            problem(format!(
                "code {code:?} is not 1 to 9 characters of letters, digits, '.' and '-'"
            ));
            continue;
        }
        let (sequence, state) = match replay(kind, fields[8], read) {
            Ok(done) => done,
            Err(message) => {
                problem(format!("{code}: {message}"));
                continue;
            }
        };
        let id = slug(&format!("{}{code}", options.name_prefix));
        let mut generator = Generator {
            size: options.size,
            metric: Span::Outer,
            status: Status::BestKnown,
            length: sequence.moves().len(),
            number: fields[1].parse().unwrap_or(0),
            moves: sequence.to_string(),
        };
        let field = |i: usize| fields.get(i).copied().unwrap_or("");
        match patterns.iter_mut().find(|(p, _)| p.id == id) {
            Some((pattern, first)) => {
                if *first != state {
                    problem(format!(
                        "{code}: generator {} builds another state",
                        fields[1]
                    ));
                    continue;
                }
                if generator.number == 0 {
                    generator.number = pattern.generators.len() + 1;
                }
                pattern.generators.push(generator);
            }
            None => {
                let mut pattern = Pattern::new(&id);
                let name = if field(7).is_empty() { code } else { field(7) };
                pattern.name = format!("{}{name}", options.name_prefix);
                if !field(6).is_empty() && field(6) != field(7) {
                    pattern.aliases.push(field(6).to_owned());
                }
                pattern.catalogue = format!("{}:{code}", options.source);
                pattern.group = (0..3)
                    .map(|level| Group {
                        de: field(2 + 2 * level).to_owned(),
                        en: field(3 + 2 * level).to_owned(),
                    })
                    .filter(|g| !g.de.is_empty() || !g.en.is_empty())
                    .collect();
                pattern.description = format!("Catalogue pattern {code}.");
                pattern.provenance = field(14).to_owned();
                pattern.origin = Some(options.origin.clone());
                if generator.number == 0 {
                    generator.number = 1;
                }
                pattern
                    .scores
                    .push((options.size, super::patternness(&state)));
                pattern.generators.push(generator);
                patterns.push((pattern, state));
            }
        }
        rows.push(ImportRow {
            line,
            code: code.to_owned(),
            moves: sequence,
            claimed: [field(9), field(10), field(11), field(12)].map(str::to_owned),
        });
        report.accepted += 1;
    }
    Imported {
        patterns: patterns.into_iter().map(|(p, _)| p).collect(),
        report,
        rows,
    }
}
