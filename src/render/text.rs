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
    pub(crate) fn render(&self, ansi: bool) -> String {
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
