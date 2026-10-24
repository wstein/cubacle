//! The move number in the top-left corner of an icon: small, dim grey, so that
//! it reads on a dark and on a light backdrop and does not compete with the
//! arrow, the `2x` mark or the notation.

use super::font;
use super::scene::{Part, Shape};
use super::style::{Backdrop, Rgb};

/// Cap height of the digits in canvas units (the notation has 11.2).
const HEIGHT: f64 = 8.0;
/// Distance of the digits from the left and the top edge of the canvas.
const MARGIN: f64 = 2.0;

fn ink(backdrop: Backdrop) -> Rgb {
    match backdrop {
        Backdrop::Dark => Rgb(0x9a, 0x9f, 0xa8),
        Backdrop::Light => Rgb(0x68, 0x6e, 0x78),
    }
}

pub(crate) fn shapes(label: &str, backdrop: Backdrop) -> Vec<Shape> {
    font::text(label, (MARGIN, MARGIN), HEIGHT, ink(backdrop), Part::Number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Facelets, PuzzleKind};
    use crate::render::raster::rasterise_canvas;
    use crate::render::scene::{HEIGHT as CANVAS_H, WIDTH as CANVAS_W};
    use crate::render::{icon, Action, Style};

    const SIZE: u32 = 218;

    fn coverage(shapes: &[Shape]) -> Vec<u8> {
        let r = rasterise_canvas(shapes, (CANVAS_W, CANVAS_H), SIZE, None);
        r.data.chunks(4).map(|p| p[3]).collect()
    }

    /// No move of any size has the number over the cube, the arrow, the `2x`
    /// mark or the notation, also for the widest numbers in use.
    #[test]
    fn the_number_never_covers_the_drawing() {
        let turns = ["", "2", "'"];
        let mut checked = 0;
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            let state = Facelets::solved(kind);
            let mut names: Vec<String> = Vec::new();
            for f in ["U", "R", "F", "D", "L", "B"] {
                for t in turns {
                    for layer in ["", "2", "3"] {
                        names.push(format!("{layer}{f}{t}"));
                        names.push(format!("{layer}{f}w{t}"));
                    }
                }
            }
            names.extend(["x", "y", "z", "x2", "y'", "z2"].map(str::to_owned));
            for name in names {
                let Ok(action) = name.parse::<Action>() else {
                    continue;
                };
                let Ok(base) = icon(&state, &action, &Style::default()) else {
                    continue;
                };
                let base = coverage(&base.shapes);
                for n in [1, 88, 888, 1888] {
                    let digits = coverage(&shapes(&n.to_string(), Backdrop::Dark));
                    for (i, (d, b)) in digits.iter().zip(&base).enumerate() {
                        assert!(
                            *d == 0 || *b == 0,
                            "{kind:?} {name} number {n} covers pixel {i}"
                        );
                    }
                }
                checked += 1;
            }
        }
        assert!(checked > 300, "{checked}");
    }

    #[test]
    fn the_number_is_small_and_in_the_corner() {
        let cover = coverage(&shapes("128", Backdrop::Light));
        let width = cover.len() / SIZE as usize;
        let rows = SIZE as usize;
        let hit = |x: usize, y: usize| cover[y * width + x] > 0;
        assert!((0..rows).any(|y| (0..width / 4).any(|x| hit(x, y))));
        assert!((0..rows).all(|y| (width / 3..width).all(|x| !hit(x, y))));
        assert!((rows / 5..rows).all(|y| (0..width).all(|x| !hit(x, y))));
    }
}
