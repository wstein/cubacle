//! Our own file for imported patterns, and the lookup of a state among them.
//!
//! ```text
//! # cubacle imported patterns 2
//! # source: <label>
//! id | catalogue | size | number | name | aliases (comma separated) | groups | provenance | description | moves
//!
//! `groups` is `de=en;de=en;de=en`, outermost first.
//! ```
//!
//! Lines with the same id and size are alternative generators.

use super::detector::{canonical, colours};
use super::{Generator, Group, Pattern, Span, Status};
use crate::puzzle::{Facelets, PuzzleKind};

const HEADER: &str = "# cubacle imported patterns 2";

/// Whether `text` starts with our header.
pub fn looks_imported(text: &str) -> bool {
    text.trim_start_matches('\u{feff}').starts_with(HEADER)
}

/// Text without the separators of the file format.
fn clean(text: &str) -> String {
    text.replace(['|', ';', '=', ',', '\n', '\r'], " ")
}

/// The text of a file that [`read_imported`] reads back.
pub fn write_imported(patterns: &[Pattern], source: &str) -> String {
    let mut out = format!("{HEADER}\n# source: {source}\n");
    for p in patterns {
        for g in p.generators() {
            out.push_str(&format!(
                "{} | {} | {} | {} | {} | {} | {} | {} | {} | {}\n",
                p.id,
                p.catalogue,
                g.size,
                g.number,
                clean(&p.name),
                p.aliases
                    .iter()
                    .map(|a| clean(a))
                    .collect::<Vec<_>>()
                    .join(","),
                p.group
                    .iter()
                    .map(|g| format!("{}={}", clean(&g.de), clean(&g.en)))
                    .collect::<Vec<_>>()
                    .join(";"),
                clean(&p.provenance),
                clean(&p.description),
                g.moves
            ));
        }
    }
    out
}

/// Reads such a file; errors name the line.
pub fn read_imported(text: &str, origin: &str) -> Result<Vec<Pattern>, String> {
    let mut list: Vec<Pattern> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |m: &str| format!("line {}: {m}", index + 1);
        let f: Vec<&str> = line.split(" | ").map(str::trim).collect();
        if f.len() != 10 {
            return Err(at("expected 10 fields separated by ' | '"));
        }
        let size: usize = f[2].parse().map_err(|_| at("bad size"))?;
        let number: usize = f[3].parse().map_err(|_| at("bad number"))?;
        if PuzzleKind::of_size(size).is_none() {
            return Err(at("no cube of that size"));
        }
        let generator = Generator {
            size,
            metric: Span::Outer,
            status: Status::BestKnown,
            length: f[9].split_whitespace().count(),
            number,
            moves: f[9].to_owned(),
        };
        generator.sequence().map_err(|e| at(&e))?;
        match list.iter_mut().find(|p| p.id == f[0]) {
            Some(p) => p.generators.push(generator),
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
                p.generators.push(generator);
                list.push(p);
            }
        }
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
