//! The fallback for terminals without graphics: the three visible faces as letters.
//!
//! Moving stickers are upper case, the others lower case. With `ansi` the
//! stickers also get background colours and the resting ones are darkened.

use super::style::{face_light, Rgb, Style, DIM};
use crate::puzzle::{locate, Face, Facelets};

use super::action::Action;
use super::arrow::Direction;
use super::geometry::Camera;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cell {
    pub letter: char,
    pub moving: bool,
    pub colour: Rgb,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Grid {
    pub n: usize,
    pub header: String,
    /// Seen from below: the faces are `D`, `F`, `R` and `D` is drawn under the others.
    pub below: bool,
    /// Faces `U`, `F`, `R` in that order, row-major (`D`, `F`, `R` when `below`).
    pub faces: [Vec<Cell>; 3],
    /// The flat net: faces `U`, `L`, `F`, `R`, `B`, `D`, row-major. Then the other fields are unused.
    pub net: Option<Vec<Vec<Cell>>>,
    /// With `net`: the folded view, the corner U F R with L, D and B flat around it.
    pub folded: bool,
}

pub(crate) fn grid(state: &Facelets, action: &Action, style: &Style, direction: Direction) -> Grid {
    let kind = state.kind();
    let faces = [Face::U, Face::F, Face::R].map(|face| {
        let start = face as usize * kind.face_len();
        (start..start + kind.face_len())
            .map(|index| {
                let sticker = state.stickers()[index];
                Cell {
                    letter: style.palette.letter(sticker),
                    moving: action.moves(kind, locate(kind, index).0),
                    colour: style.palette.colour(sticker),
                }
            })
            .collect()
    });
    let half = if action.is_half() { " (half turn)" } else { "" };
    Grid {
        n: kind.size(),
        header: format!("{action} {}{half}", direction.glyph()),
        below: false,
        faces,
        net: None,
        folded: false,
    }
}

/// A picture of the state with nothing marked: every cell is "moving", so
/// the letters are upper case and the colours are not darkened.
pub(crate) fn state_grid(state: &Facelets, style: &Style, camera: Camera) -> Grid {
    let kind = state.kind();
    let faces = [camera.cap(), Face::F, Face::R].map(|face| {
        let start = face as usize * kind.face_len();
        (start..start + kind.face_len())
            .map(|index| {
                let sticker = state.stickers()[index];
                Cell {
                    letter: style.palette.letter(sticker),
                    moving: true,
                    colour: style.palette.colour(sticker),
                }
            })
            .collect()
    });
    Grid {
        n: kind.size(),
        header: "state".to_owned(),
        below: camera == Camera::Below,
        faces,
        net: None,
        folded: false,
    }
}

fn cell(c: &Cell, face: Face, ansi: bool) -> String {
    if !ansi {
        let letter = if c.moving {
            c.letter.to_ascii_uppercase()
        } else {
            c.letter.to_ascii_lowercase()
        };
        return format!("{letter} ");
    }
    let dim = if c.moving { 1.0 } else { DIM };
    let Rgb(r, g, b) = c.colour.scale(face_light(face) * dim);
    format!(
        "\x1b[48;2;{r};{g};{b}m\x1b[38;2;0;0;0m {} \x1b[0m",
        c.letter.to_ascii_uppercase()
    )
}

impl Grid {
    /// The cross: U above F, then L F R B in a row, D below F; one space
    /// between two faces.
    fn render_net(&self, faces: &[Vec<Cell>], ansi: bool) -> String {
        let n = self.n;
        let order = [Face::U, Face::L, Face::F, Face::R, Face::B, Face::D];
        let row = |at: usize, r: usize| -> String {
            (0..n)
                .map(|c| cell(&faces[at][r * n + c], order[at], ansi))
                .collect::<String>()
        };
        let width = if ansi { 3 * n } else { 2 * n };
        let mut out = format!("{}\n", self.header);
        let strip = |out: &mut String, at: usize, lead: usize| {
            for r in 0..n {
                out.push_str(&" ".repeat(lead));
                out.push_str(row(at, r).trim_end());
                out.push('\n');
            }
        };
        strip(&mut out, 0, width + 1);
        for r in 0..n {
            let line: Vec<String> = (1..5).map(|at| row(at, r)).collect();
            out.push_str(line.join(" ").trim_end());
            out.push('\n');
        }
        strip(&mut out, 5, width + 1);
        out
    }

    /// The folded view: the corner (U leaning away over F, R beside F) with
    /// L left of F, D below F and B right of R, top aligned with U.
    fn render_folded(&self, faces: &[Vec<Cell>], ansi: bool) -> String {
        let n = self.n;
        let order = [Face::U, Face::L, Face::F, Face::R, Face::B, Face::D];
        let cw = if ansi { 3 } else { 2 };
        let row = |at: usize, r: usize| -> String {
            (0..n)
                .map(|c| cell(&faces[at][r * n + c], order[at], ansi))
                .collect()
        };
        // visible column of the first cell of B
        let right = 3 * cw * n + 2;
        let lead = cw * n + 1;
        let mut lines: Vec<(String, usize)> = Vec::new();
        for r in 0..n {
            let pad = lead + n - r;
            lines.push((format!("{}{}", " ".repeat(pad), row(0, r)), pad + cw * n));
        }
        for r in 0..n {
            lines.push((
                format!("{} {}{}", row(1, r), row(2, r), row(3, r)),
                3 * cw * n + 1,
            ));
        }
        for r in 0..n {
            let pad = lead;
            lines.push((format!("{}{}", " ".repeat(pad), row(5, r)), pad + cw * n));
        }
        let mut out = format!("{}\n", self.header);
        for (i, (text, width)) in lines.into_iter().enumerate() {
            out.push_str(&text);
            if i < n {
                out.push_str(&" ".repeat(right - width));
                out.push_str(&row(4, i));
            }
            let trimmed = out.trim_end_matches(' ').len();
            out.truncate(trimmed);
            out.push('\n');
        }
        out
    }

    pub(crate) fn render(&self, ansi: bool) -> String {
        if let Some(faces) = &self.net {
            if self.folded {
                return self.render_folded(faces, ansi);
            }
            return self.render_net(faces, ansi);
        }
        let n = self.n;
        let mut out = format!("{}\n", self.header);
        let row = |face: usize, kind: Face, r: usize| -> String {
            (0..n)
                .map(|c| cell(&self.faces[face][r * n + c], kind, ansi))
                .collect()
        };
        let cap = if self.below { Face::D } else { Face::U };
        if !self.below {
            for r in 0..n {
                out.push_str(&" ".repeat(n - r));
                out.push_str(row(0, cap, r).trim_end());
                out.push('\n');
            }
        }
        for r in 0..n {
            out.push_str(&row(1, Face::F, r));
            if ansi {
                out.push(' ');
            }
            out.push_str(row(2, Face::R, r).trim_end());
            out.push('\n');
        }
        if self.below {
            // the nearest row of D touches F, the others lean away to the right
            for r in 0..n {
                out.push_str(&" ".repeat(r + 1));
                out.push_str(row(0, cap, r).trim_end());
                out.push('\n');
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::arrow::plan;

    #[test]
    fn plain_text_marks_the_moving_layer_without_escapes() {
        let state = Facelets::solved(crate::puzzle::PuzzleKind::Cube3);
        let action: Action = "R".parse().unwrap();
        let direction = plan(&action, 3).direction;
        let text = grid(&state, &action, &Style::default(), direction).render(false);
        assert_eq!(
            text,
            "R \u{2191}\n   w w W\n  w w W\n w w W\ng g G R R R\ng g G R R R\ng g G R R R\n"
        );
        assert!(!text.contains('\x1b'));
    }

    #[test]
    fn ansi_text_uses_background_colours() {
        let state = Facelets::solved(crate::puzzle::PuzzleKind::Cube2);
        let action: Action = "U2".parse().unwrap();
        let direction = plan(&action, 2).direction;
        let text = grid(&state, &action, &Style::default(), direction).render(true);
        assert!(text.starts_with("U2 \u{2190} (half turn)\n"));
        assert!(text.contains("\x1b[48;2;"));
        assert!(text.contains("\x1b[0m"));
    }
}
