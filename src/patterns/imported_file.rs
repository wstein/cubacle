//! Our own file for imported patterns, and the lookup of a state among them.
//!
//! A tab-separated file (`.tsv`):
//!
//! ```text
//! #cubacle-patterns 1 patternness-v1
//! #source <label>
//! id  catalogue  size  number  name  aliases  groups  provenance  description  patternness  moves
//! ```
//!
//! `aliases` are comma separated, `groups` is `de=en;de=en;de=en` (outermost
//! first), `patternness` is an integer from 0 to 100 or `-`, and the moves are
//! the last field. Lines with the same id and size are alternative generators.
//! When the formula version in the header is not the code's, the stored
//! values are ignored and computed again.

use super::detector::{canonical, colours, PATTERNNESS_VERSION};
use super::{Generator, Group, Pattern, Span, Status};
use crate::puzzle::{Facelets, PuzzleKind};

const MAGIC: &str = "#cubacle-patterns 1";

/// Whether `text` starts with our header.
pub fn looks_imported(text: &str) -> bool {
    text.trim_start_matches('\u{feff}').starts_with(MAGIC)
}

/// Whether the file was written with another patternness formula.
pub fn imported_is_stale(text: &str) -> bool {
    let first = text.trim_start_matches('\u{feff}').lines().next();
    first.is_none_or(|l| l.split_whitespace().nth(2) != Some(PATTERNNESS_VERSION))
}

/// Text without the separators of the file format.
fn clean(text: &str) -> String {
    text.replace(['\t', ';', '=', ',', '\n', '\r'], " ")
}

/// The text of a file that [`read_imported`] reads back; the patternness is
/// computed for a pattern that has none stored.
pub fn write_imported(patterns: &[Pattern], source: &str) -> String {
    let mut out = format!("{MAGIC} {PATTERNNESS_VERSION}\n#source {source}\n");
    for p in patterns {
        for g in p.generators() {
            let score = p
                .patternness(g.size)
                .map_or("-".to_owned(), |s| s.to_string());
            let groups = p
                .group
                .iter()
                .map(|g| format!("{}={}", clean(&g.de), clean(&g.en)))
                .collect::<Vec<_>>()
                .join(";");
            let aliases = p
                .aliases
                .iter()
                .map(|a| clean(a))
                .collect::<Vec<_>>()
                .join(",");
            let fields = [
                p.id.clone(),
                p.catalogue.clone(),
                g.size.to_string(),
                g.number.to_string(),
                clean(&p.name),
                aliases,
                groups,
                clean(&p.provenance),
                clean(&p.description),
                score,
                g.moves.clone(),
            ];
            out.push_str(&fields.join("\t"));
            out.push('\n');
        }
    }
    out
}

/// Reads such a file; errors name the line.
pub fn read_imported(text: &str, origin: &str) -> Result<Vec<Pattern>, String> {
    let keep_scores = !imported_is_stale(text);
    let mut list: Vec<Pattern> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_matches(|c: char| c == '\r' || c == '\u{feff}');
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |m: &str| format!("line {}: {m}", index + 1);
        let f: Vec<&str> = line.split('\t').map(str::trim).collect();
        if f.len() != 11 {
            return Err(at("expected 11 fields separated by tabs"));
        }
        let size: usize = f[2].parse().map_err(|_| at("bad size"))?;
        let number: usize = f[3].parse().map_err(|_| at("bad number"))?;
        if PuzzleKind::of_size(size).is_none() {
            return Err(at("no cube of that size"));
        }
        let score: Option<u8> = match f[9] {
            "-" => None,
            text => Some(
                text.parse()
                    .ok()
                    .filter(|&v| v <= 100)
                    .ok_or_else(|| at("bad patternness"))?,
            ),
        };
        let generator = Generator {
            size,
            metric: Span::Outer,
            status: Status::BestKnown,
            length: f[10].split_whitespace().count(),
            number,
            moves: f[10].to_owned(),
        };
        generator.sequence().map_err(|e| at(&e))?;
        let index = match list.iter().position(|p| p.id == f[0]) {
            Some(i) => i,
            None => {
                let mut p = Pattern::new(f[0]);
                p.catalogue = f[1].to_owned();
                p.name = f[4].to_owned();
                p.group = f[6]
                    .split(';')
                    .filter_map(|g| g.split_once('='))
                    .map(|(de, en)| Group {
                        de: de.to_owned(),
                        en: en.to_owned(),
                    })
                    .collect();
                p.provenance = f[7].to_owned();
                p.aliases = f[5]
                    .split(',')
                    .map(str::trim)
                    .filter(|a| !a.is_empty())
                    .map(str::to_owned)
                    .collect();
                p.description = f[8].to_owned();
                p.origin = Some(origin.to_owned());
                list.push(p);
                list.len() - 1
            }
        };
        let p = &mut list[index];
        if let (true, Some(score)) = (keep_scores, score) {
            if !p.scores.iter().any(|(s, _)| *s == size) {
                p.scores.push((size, score));
            }
        }
        p.generators.push(generator);
    }
    Ok(list)
}

/// The imported patterns whose state is `state` up to turning, mirroring and
/// recolouring (equal canonical forms).
pub fn identify_imported<'a>(patterns: &'a [Pattern], state: &Facelets) -> Vec<&'a Pattern> {
    let kind = state.kind();
    let wanted = canonical(kind, &colours(state));
    patterns
        .iter()
        .filter(|p| {
            p.state(kind)
                .is_some_and(|s| canonical(kind, &colours(&s)) == wanted)
        })
        .collect()
}
