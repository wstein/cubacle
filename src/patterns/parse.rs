//! The plain-text data files: `patterns.txt` (definitions) and
//! `generators.txt` (searched generators).
//!
//! `patterns.txt` has one block per pattern, blocks separated by blank lines,
//! `#` starts a comment. Keys: `id`, `name`, `alias` (repeatable), `desc`,
//! `def <sizes> <spec>` (repeatable; sizes are `*` or a list such as `4,6`).
//! Specs: `parity`, `superflip`, `rings[:FACES] map...`, `blocks[:FACES] map...`,
//! `classes[:FACES] corner edge inner`.
//!
//! `generators.txt` has one line per generator:
//! `id | size | metric | status | length | moves`.

use super::model::{Def, Map};
use super::{Generator, Pattern, Span, Status};
use crate::puzzle::Face;

fn faces_of(text: &str) -> Result<Vec<Face>, String> {
    text.chars()
        .map(|c| Face::from_letter(c).ok_or_else(|| format!("bad face {c:?}")))
        .collect()
}

fn maps_of<'a>(words: impl Iterator<Item = &'a str>) -> Result<Vec<Map>, String> {
    words
        .map(|w| Map::parse(w).ok_or_else(|| format!("bad map {w:?}")))
        .collect()
}

fn parse_spec(spec: &str) -> Result<Def, String> {
    let mut words = spec.split_whitespace();
    let head = words.next().ok_or("empty definition")?;
    let (kind, faces) = match head.split_once(':') {
        Some((kind, faces)) => (kind, faces_of(faces)?),
        None => (head, Face::ALL.to_vec()),
    };
    match kind {
        "parity" => Ok(Def::Parity),
        "superflip" => Ok(Def::Superflip),
        "rings" => {
            let maps = maps_of(words)?;
            if maps.is_empty() {
                return Err("rings needs at least one map".into());
            }
            Ok(Def::Rings { maps, faces })
        }
        "blocks" => {
            let maps = maps_of(words)?;
            if maps.is_empty() {
                return Err("blocks needs at least one map".into());
            }
            Ok(Def::Blocks { maps, faces })
        }
        "classes" => match maps_of(words)?.as_slice() {
            &[corner, edge, inner] => Ok(Def::Classes {
                corner,
                edge,
                inner,
                faces,
            }),
            _ => Err("classes needs three maps".into()),
        },
        _ => Err(format!("unknown definition {kind:?}")),
    }
}

fn parse_sizes(text: &str) -> Result<Vec<usize>, String> {
    if text == "*" {
        return Ok((3..=7).collect());
    }
    text.split(',')
        .map(|s| s.parse().map_err(|_| format!("bad size {s:?}")))
        .collect()
}

pub(super) fn parse_patterns(text: &str) -> Result<Vec<Pattern>, String> {
    let mut out: Vec<Pattern> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();
        if key == "id" {
            out.push(Pattern::new(rest));
            continue;
        }
        let current = out.last_mut().ok_or("a key before the first id")?;
        match key {
            "name" => current.name = rest.to_owned(),
            "alias" => current.aliases.push(rest.to_owned()),
            "desc" => current.description = rest.to_owned(),
            "def" => {
                let (sizes, spec) = rest.split_once(' ').ok_or("def needs sizes and a spec")?;
                let def = parse_spec(spec).map_err(|e| format!("{}: {e}", current.id))?;
                for size in parse_sizes(sizes)? {
                    current.defs.push((size, def.clone()));
                }
            }
            _ => return Err(format!("unknown key {key:?}")),
        }
    }
    Ok(out)
}

pub(super) fn parse_generators(text: &str) -> Result<Vec<(String, Generator)>, String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.splitn(6, '|').map(str::trim).collect();
        let [id, size, metric, status, length, moves] = f[..] else {
            return Err(format!("bad generator line {line:?}"));
        };
        out.push((
            id.to_owned(),
            Generator {
                size: size.parse().map_err(|_| format!("bad size in {line:?}"))?,
                metric: Span::parse(metric).ok_or_else(|| format!("bad metric in {line:?}"))?,
                status: Status::parse(status).ok_or_else(|| format!("bad status in {line:?}"))?,
                length: length
                    .parse()
                    .map_err(|_| format!("bad length in {line:?}"))?,
                number: 0,
                moves: moves.to_owned(),
            },
        ));
    }
    Ok(out)
}
