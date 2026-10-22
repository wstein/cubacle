//! A tiny stroke font for move labels, so SVG and pixels match without fonts.
//!
//! Glyphs live in a box 1 wide and 1.4 tall with `y` pointing down.

use super::scene::{Part, Shape};
use super::style::Rgb;

type Stroke = &'static [(f64, f64)];

pub(crate) const GLYPH_HEIGHT: f64 = 1.4;

fn strokes(c: char) -> &'static [Stroke] {
    match c {
        'U' => &[&[
            (0., 0.),
            (0., 1.1),
            (0.25, 1.4),
            (0.75, 1.4),
            (1., 1.1),
            (1., 0.),
        ]],
        'R' => &[
            &[
                (0., 1.4),
                (0., 0.),
                (0.8, 0.),
                (1., 0.2),
                (1., 0.5),
                (0.8, 0.7),
                (0., 0.7),
            ],
            &[(0.5, 0.7), (1., 1.4)],
        ],
        'F' => &[&[(1., 0.), (0., 0.), (0., 1.4)], &[(0., 0.7), (0.7, 0.7)]],
        'D' => &[&[
            (0., 0.),
            (0., 1.4),
            (0.6, 1.4),
            (1., 1.),
            (1., 0.4),
            (0.6, 0.),
            (0., 0.),
        ]],
        'L' => &[&[(0., 0.), (0., 1.4), (1., 1.4)]],
        'B' => &[
            &[
                (0., 0.),
                (0.75, 0.),
                (0.95, 0.2),
                (0.95, 0.5),
                (0.75, 0.7),
                (0., 0.7),
            ],
            &[(0.75, 0.7), (1., 0.9), (1., 1.2), (0.75, 1.4), (0., 1.4)],
            &[(0., 0.), (0., 1.4)],
        ],
        'w' => &[&[(0., 0.5), (0.25, 1.4), (0.5, 0.8), (0.75, 1.4), (1., 0.5)]],
        'x' => &[&[(0., 0.5), (1., 1.4)], &[(1., 0.5), (0., 1.4)]],
        'y' => &[
            &[(0., 0.5), (0.5, 1.2), (1., 0.5)],
            &[(0.5, 1.2), (0.2, 1.8)],
        ],
        'z' => &[&[(0., 0.5), (1., 0.5), (0., 1.4), (1., 1.4)]],
        '\'' => &[&[(0.2, 0.), (0.05, 0.45)]],
        '1' => &[&[(0.2, 0.25), (0.6, 0.), (0.6, 1.4)]],
        '2' => &[&[
            (0., 0.3),
            (0.3, 0.),
            (0.7, 0.),
            (1., 0.3),
            (1., 0.6),
            (0., 1.4),
            (1., 1.4),
        ]],
        '3' => &[&[
            (0., 0.),
            (1., 0.),
            (0.5, 0.65),
            (0.85, 0.7),
            (1., 0.95),
            (1., 1.2),
            (0.7, 1.4),
            (0.3, 1.4),
            (0., 1.15),
        ]],
        '4' => &[&[(0.75, 1.4), (0.75, 0.), (0., 1.), (1., 1.)]],
        '5' => &[&[
            (1., 0.),
            (0., 0.),
            (0., 0.65),
            (0.7, 0.6),
            (1., 0.85),
            (1., 1.15),
            (0.7, 1.4),
            (0., 1.3),
        ]],
        '6' => &[&[
            (0.9, 0.05),
            (0.4, 0.),
            (0., 0.5),
            (0., 1.1),
            (0.3, 1.4),
            (0.7, 1.4),
            (1., 1.1),
            (1., 0.85),
            (0.7, 0.65),
            (0.3, 0.65),
            (0., 0.9),
        ]],
        '7' => &[&[(0., 0.), (1., 0.), (0.35, 1.4)]],
        _ => &[],
    }
}

fn advance(c: char) -> f64 {
    if c == '\'' {
        0.5
    } else {
        1.4
    }
}

/// The width of `text` in glyph units.
pub(crate) fn measure(text: &str) -> f64 {
    let mut chars = text.chars().rev();
    let last = chars
        .next()
        .map_or(0.0, |c| if c == '\'' { 0.2 } else { 1.0 });
    chars.map(advance).sum::<f64>() + last
}

/// Filled quads for each stroke segment; `height` is the glyph height in
/// canvas units, `stroke` the line width, `(x, y)` the top-left corner.
pub(crate) fn text(
    text: &str,
    x: f64,
    y: f64,
    height: f64,
    stroke: f64,
    fill: Rgb,
    part: Part,
) -> Vec<Shape> {
    let scale = height / GLYPH_HEIGHT;
    let mut shapes = Vec::new();
    let mut cursor = 0.0;
    for c in text.chars() {
        for line in strokes(c) {
            for pair in line.windows(2) {
                let a = [x + (cursor + pair[0].0) * scale, y + pair[0].1 * scale];
                let b = [x + (cursor + pair[1].0) * scale, y + pair[1].1 * scale];
                shapes.push(Shape::new(part, segment(a, b, stroke), fill));
            }
        }
        cursor += advance(c);
    }
    shapes
}

/// A quad around `a..b` with square caps.
fn segment(a: [f64; 2], b: [f64; 2], width: f64) -> Vec<[f64; 2]> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (ux, uy) = (dx / len * width / 2.0, dy / len * width / 2.0);
    let (nx, ny) = (-uy, ux);
    vec![
        [a[0] - ux + nx, a[1] - uy + ny],
        [b[0] + ux + nx, b[1] + uy + ny],
        [b[0] + ux - nx, b[1] + uy - ny],
        [a[0] - ux - nx, a[1] - uy - ny],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_label_character_has_a_glyph() {
        for c in "URFDLBwxyz'1234567".chars() {
            assert!(!strokes(c).is_empty(), "{c}");
        }
        assert!(strokes('Q').is_empty());
    }

    #[test]
    fn text_stays_in_its_box() {
        let shapes = text("3Rw'", 10.0, 20.0, 14.0, 2.0, Rgb(0, 0, 0), Part::Label);
        let width = measure("3Rw'") * 10.0;
        for p in shapes.iter().flat_map(|s| s.points.iter()) {
            assert!(p[0] >= 8.0 && p[0] <= 10.0 + width + 2.0, "{p:?}");
            assert!(p[1] >= 17.0 && p[1] <= 20.0 + 14.0 + 2.0, "{p:?}");
        }
    }
}
