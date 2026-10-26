//! The `.cubacle` definition file: `key: value` lines that define one template.
//!
//! `#` starts a comment, a line that starts with white space continues the
//! value of the previous key, a blank line ends it.

use crate::notation::{ColorScheme, SchemeError};
use crate::puzzle::PuzzleKind;

use super::error::TemplateError;
use super::guard::Guard;
use super::schema;
use super::Template;

const KEYS: [&str; 17] = [
    "pattern-name",
    "pattern-title",
    "pattern-aliases",
    "cube-size",
    "color-scheme",
    "color-letters",
    "sticker-grid",
    "color-constraints",
    "move-alphabet",
    "search-limits",
    "pattern-tags",
    "required-symmetry",
    "required-antisymmetry",
    "sequence-order",
    "movable-orbits",
    "fixed-orbits",
    "format-version",
];

/// A parsed `.cubacle` file. The search keys (`alphabet`, `limits`) are kept as
/// written; nothing here searches or counts.
#[derive(Debug, Clone)]
pub struct GenFile {
    pub name: String,
    pub title: Option<String>,
    pub aliases: Vec<String>,
    pub scheme: ColorScheme,
    /// The text of the `color-constraints` key, empty when there is none.
    pub guard: String,
    pub alphabet: Option<String>,
    pub limits: Option<String>,
    pub tags: Vec<String>,
    /// One template per size of the `cube-size` key.
    pub templates: Vec<Template>,
}

struct Entry {
    key: String,
    /// Line of the key, from 1.
    line: usize,
    /// Column of the start of the value, from 1.
    column: usize,
    /// The value with the columns of the file kept: padded on the left to its
    /// column, continuation lines as they are.
    text: String,
}

impl Entry {
    fn error(&self, message: impl Into<String>) -> TemplateError {
        TemplateError::new(self.line, self.column, message)
    }

    fn value(&self) -> &str {
        self.text.trim()
    }
}

impl GenFile {
    pub fn parse(text: &str) -> Result<GenFile, TemplateError> {
        let entries = entries(text)?;
        let find = |key: &str| entries.iter().find(|entry| entry.key == key);
        let require = |key: &str| {
            find(key)
                .ok_or_else(|| TemplateError::new(1, 1, format!("missing required key \"{key}\"")))
        };

        let name_entry = require("pattern-name")?;
        let name = name_entry.value().to_owned();
        check_id(name_entry, &name)?;
        let size_entry = require("cube-size")?;
        let schema_entry = require("sticker-grid")?;
        if schema_entry.value().is_empty() {
            return Err(schema_entry.error("the sticker grid is empty"));
        }

        let mut aliases = Vec::new();
        if let Some(entry) = find("pattern-aliases") {
            for alias in list(entry.value()) {
                check_id(entry, alias)?;
                aliases.push(alias.to_owned());
            }
        }
        let scheme = match find("color-scheme") {
            None => ColorScheme::new(['U', 'R', 'F', 'D', 'L', 'B']).expect("distinct letters"),
            Some(entry) => entry.value().parse().map_err(|e: SchemeError| {
                entry.error(format!(
                    "{}, expected URFDLB, western (boy), japanese or six distinct letters",
                    scheme_problem(&e)
                ))
            })?,
        };
        let guard_entry = find("color-constraints");
        let guard = guard_entry.map_or("", Entry::value).to_owned();

        let mut templates = Vec::new();
        for kind in sizes(size_entry)? {
            let schema = schema::parse(&schema_entry.text, kind, &scheme)
                .map_err(|e| e.shifted(schema_entry.line - 1))?;
            let guard = match guard_entry {
                Some(entry) => Guard::parse(&entry.text, &schema.names)
                    .map_err(|e| e.shifted(entry.line - 1))?,
                None => Guard::parse("", &schema.names)?,
            };
            templates.push(Template::from_parts(kind, schema, guard));
        }

        Ok(GenFile {
            name,
            title: find("pattern-title").map(|entry| entry.value().to_owned()),
            aliases,
            scheme,
            guard,
            alphabet: find("move-alphabet").map(|entry| entry.value().to_owned()),
            limits: find("search-limits").map(|entry| entry.value().to_owned()),
            tags: find("pattern-tags")
                .map(|entry| list(entry.value()).map(str::to_owned).collect())
                .unwrap_or_default(),
            templates,
        })
    }
}

fn scheme_problem(error: &SchemeError) -> String {
    match error {
        SchemeError::Unknown(text) => format!("unknown color scheme \"{text}\""),
        SchemeError::Duplicate(letter) => format!("the letter '{letter}' is used twice"),
        SchemeError::NoCenters(_) => "the scheme cannot be read".to_owned(),
    }
}

fn list(text: &str) -> impl Iterator<Item = &str> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
}

fn check_id(entry: &Entry, id: &str) -> Result<(), TemplateError> {
    let ok = !id.is_empty()
        && !id.starts_with('-')
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(entry.error(format!(
            "\"{id}\" is not an id, expected lowercase letters, digits and \"-\", not starting with \"-\""
        )))
    }
}

fn sizes(entry: &Entry) -> Result<Vec<PuzzleKind>, TemplateError> {
    let mut kinds = Vec::new();
    for word in entry.value().split([',', ' ']).filter(|w| !w.is_empty()) {
        let kind = word
            .parse::<usize>()
            .ok()
            .and_then(PuzzleKind::of_size)
            .ok_or_else(|| {
                entry.error(format!("\"{word}\" is not a cube size, expected 2 to 7"))
            })?;
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    if kinds.is_empty() {
        return Err(entry.error("expected a cube size, 2 to 7"));
    }
    Ok(kinds)
}

/// Splits the file into entries, keeping the line and column of each value.
fn entries(text: &str) -> Result<Vec<Entry>, TemplateError> {
    let mut out: Vec<Entry> = Vec::new();
    let mut open = false;
    for (index, raw) in text.split('\n').enumerate() {
        let line = index + 1;
        let content: String = raw
            .trim_end_matches('\r')
            .chars()
            .take_while(|&c| c != '#')
            .collect();
        if content.trim().is_empty() {
            // A blank line ends an entry, a comment line does not.
            if raw.trim().is_empty() {
                open = false;
            }
            continue;
        }
        if content.starts_with([' ', '\t']) {
            let column = content.chars().take_while(|c| c.is_whitespace()).count() + 1;
            match out.last_mut().filter(|_| open) {
                Some(entry) => {
                    entry.text.push('\n');
                    entry.text.push_str(&content);
                }
                None => {
                    return Err(TemplateError::new(
                        line,
                        column,
                        "a continuation line needs a key line before it",
                    ))
                }
            }
            continue;
        }
        let Some((key_raw, rest)) = content.split_once(':') else {
            return Err(TemplateError::new(
                line,
                content.chars().count() + 1,
                "expected \"key: value\"",
            ));
        };
        let key = key_raw.trim_end();
        if !KEYS.contains(&key) {
            return Err(TemplateError::new(
                line,
                1,
                format!("unknown key \"{key}\", expected one of {}", KEYS.join(", ")),
            ));
        }
        if out.iter().any(|entry| entry.key == key) {
            return Err(TemplateError::new(
                line,
                1,
                format!("the key \"{key}\" is given twice"),
            ));
        }
        let lead = rest.chars().take_while(|c| c.is_whitespace()).count();
        let column = key_raw.chars().count() + 1 + lead + 1;
        out.push(Entry {
            key: key.to_owned(),
            line,
            column,
            text: format!("{}{}", " ".repeat(column - 1), rest.trim_start()),
        });
        open = true;
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
