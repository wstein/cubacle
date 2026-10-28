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
//! One-face form: a grid of at most N*N tokens in all is one face, the same
//! on all six. Then `/` and newlines only break rows, a placeholder gets the
//! index of its face (`a` is `a1` on U, `a2` on R, ...) and the center letter
//! of the U face stands for the letter of the face it is on.
//!
//! A lowercase token (`a`, `b12`) names a placeholder: all stickers with the
//! same name show one color. An uppercase letter is a fixed color, read
//! through the color scheme. `.` and `?` match anything, each independently.

use std::collections::HashMap;

use crate::notation::ColorScheme;
use crate::puzzle::{Face, PuzzleKind};

use super::error::{Source, TemplateError};

/// A token as written, before the faces are known.
enum Token {
    Any,
    Fixed(Face),
    Var(String),
}

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
    let mut tokens: Vec<Token> = Vec::with_capacity(kind.facelet_count());
    let written = chars
        .iter()
        .collect::<String>()
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|word| !word.is_empty())
        .count();
    // A grid of one face: the rows may be broken by "/" and newlines.
    let one_face = written > 0 && written <= face_len;
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
            '/' | '\n' if one_face => i += 1,
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
                        Token::Var(chars[start..i].iter().collect())
                    }
                    'A'..='Z' => match scheme.face_of(c) {
                        Some(face) => Token::Fixed(face),
                        None => {
                            return Err(source.error(
                                start,
                                format!("unknown color '{c}', the scheme has {scheme}"),
                            ))
                        }
                    },
                    '.' | '?' => Token::Any,
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
                tokens.push(slot);
                on_face += 1;
            }
        }
    }
    if one_face {
        if on_face != face_len {
            return Err(source.error(
                chars.len(),
                format!("the grid of one face has {on_face} stickers, expected {face_len} ({size}x{size})"),
            ));
        }
        return Ok(build(&tokens, kind, true));
    }
    if on_face > 0 {
        finish(chars.len(), &mut faces, &mut on_face)?;
    }
    if faces != 6 {
        return Err(source.error(chars.len(), format!("expected 6 faces, found {faces}")));
    }
    Ok(build(&tokens, kind, false))
}

/// The slots of the six faces. With `one_face` the tokens are the U face and
/// are repeated on all six.
fn build(tokens: &[Token], kind: PuzzleKind, one_face: bool) -> Schema {
    let face_len = kind.face_len();
    let center = (kind.size() % 2 == 1).then_some(face_len / 2);
    let mut slots: Vec<Slot> = Vec::with_capacity(kind.facelet_count());
    let mut names: Vec<String> = Vec::new();
    let mut known: HashMap<String, usize> = HashMap::new();
    let mut intern = |name: String| {
        let next = names.len();
        let id = *known.entry(name.clone()).or_insert(next);
        if id == next {
            names.push(name);
        }
        id
    };
    let repeats = if one_face { 6 } else { 1 };
    for f in 0..repeats {
        for (at, token) in tokens.iter().enumerate() {
            slots.push(match token {
                Token::Any => Slot::Any,
                Token::Fixed(face) if one_face && Some(at) == center && *face == Face::U => {
                    Slot::Fixed(Face::ALL[f])
                }
                Token::Fixed(face) => Slot::Fixed(*face),
                Token::Var(name) if one_face => Slot::Var(intern(format!("{name}{}", f + 1))),
                Token::Var(name) => Slot::Var(intern(name.clone())),
            });
        }
    }
    Schema { slots, names }
}

#[cfg(test)]
mod tests;
