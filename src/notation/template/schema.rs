//! The schema grid: which stickers share a color.
//!
//! ```text
//! schema   = face { face-sep face }
//! face     = token { sep token }          exactly N*N tokens for size N
//! token    = lower [ digits ] | UPPER | "." | "?"
//! sep      = "," | space
//! face-sep = "/" | newline | blank line   exactly 6 faces, in URFDLB order
//! ```
//!
//! A lowercase token (`a`, `b12`) names a placeholder: all stickers with the
//! same name show one color. An uppercase letter is a fixed color, read
//! through the color scheme. `.` and `?` match anything, each independently.

use std::collections::HashMap;

use crate::notation::ColorScheme;
use crate::puzzle::{Face, PuzzleKind};

use super::error::{Source, TemplateError};

/// What one sticker of the template must show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    Any,
    Fixed(Face),
    /// The color of the placeholder with this index.
    Var(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct Schema {
    /// One slot per sticker, in facelet order (`URFDLB`, row by row).
    pub(crate) slots: Vec<Slot>,
    /// The placeholder names in order of first appearance.
    pub(crate) names: Vec<String>,
}

pub(crate) fn parse(
    text: &str,
    kind: PuzzleKind,
    scheme: &ColorScheme,
) -> Result<Schema, TemplateError> {
    let source = Source::new(text);
    let chars = source.chars();
    let size = kind.size();
    let face_len = kind.face_len();
    let mut slots: Vec<Slot> = Vec::with_capacity(kind.facelet_count());
    let mut names: Vec<String> = Vec::new();
    let mut known: HashMap<String, usize> = HashMap::new();
    let mut faces = 0;
    let mut on_face = 0;
    // The last face was ended by a newline, so a "/" right after is harmless.
    let mut ended_by_newline = false;
    let mut i = 0;

    let finish = |at: usize, faces: &mut usize, on_face: &mut usize| {
        if *on_face != face_len {
            return Err(source.error(
                at,
                format!(
                    "face {} has {} stickers, expected {face_len} ({size}x{size})",
                    *faces + 1,
                    *on_face
                ),
            ));
        }
        *faces += 1;
        *on_face = 0;
        Ok(())
    };

    while i < chars.len() {
        let c = chars[i];
        match c {
            '/' => {
                if on_face == 0 {
                    if !ended_by_newline {
                        return Err(source.error(i, "empty face before \"/\""));
                    }
                } else {
                    finish(i, &mut faces, &mut on_face)?;
                }
                ended_by_newline = false;
                i += 1;
            }
            '\n' => {
                if on_face > 0 {
                    finish(i, &mut faces, &mut on_face)?;
                    ended_by_newline = true;
                }
                i += 1;
            }
            c if c == ',' || c.is_whitespace() => i += 1,
            _ => {
                if faces == 6 {
                    return Err(source.error(i, "a cube has 6 faces, the grid has more"));
                }
                if on_face == face_len {
                    return Err(source.error(
                        i,
                        format!(
                            "face {} has more than {face_len} stickers ({size}x{size})",
                            faces + 1
                        ),
                    ));
                }
                ended_by_newline = false;
                let start = i;
                i += 1;
                let slot = match c {
                    'a'..='z' => {
                        while chars.get(i).is_some_and(char::is_ascii_digit) {
                            i += 1;
                        }
                        let name: String = chars[start..i].iter().collect();
                        let next = names.len();
                        let id = *known.entry(name.clone()).or_insert(next);
                        if id == next {
                            names.push(name);
                        }
                        Slot::Var(id)
                    }
                    'A'..='Z' => match scheme.face_of(c) {
                        Some(face) => Slot::Fixed(face),
                        None => {
                            return Err(source.error(
                                start,
                                format!("unknown color '{c}', the scheme has {scheme}"),
                            ))
                        }
                    },
                    '.' | '?' => Slot::Any,
                    other => {
                        return Err(source.error(
                            start,
                            format!(
                                "unexpected character {other:?}, expected a lowercase letter, an uppercase letter, \".\" or \"?\""
                            ),
                        ))
                    }
                };
                if let Some(&next) = chars.get(i) {
                    if next != ',' && next != '/' && !next.is_whitespace() {
                        return Err(source.error(
                            i,
                            format!("expected a separator after the token, found {next:?}"),
                        ));
                    }
                }
                slots.push(slot);
                on_face += 1;
            }
        }
    }
    if on_face > 0 {
        finish(chars.len(), &mut faces, &mut on_face)?;
    }
    if faces != 6 {
        return Err(source.error(chars.len(), format!("expected 6 faces, found {faces}")));
    }
    Ok(Schema { slots, names })
}

#[cfg(test)]
mod tests;
