//! Lettering for the notation and the `2x` mark: outlines of Quicksand Bold,
//! embedded as polygons (see `glyphs`, `NOTICE.md`), so SVG, pixels and Kitty
//! output match without fonts.
//!
//! Glyph units: cap height 1.4, `y` pointing down from the cap line.

use super::glyphs::{Glyph, QUICKSAND};
use super::scene::{Part, Shape};
use super::style::Rgb;

pub(crate) const GLYPH_HEIGHT: f64 = 1.4;

fn glyph(c: char) -> Option<&'static Glyph> {
    QUICKSAND.iter().find(|g| g.0 == c)
}

/// The width of `text` in glyph units: advances up to the last glyph, then its ink.
pub(crate) fn measure(text: &str) -> f64 {
    let mut chars = text.chars().rev();
    let last = chars.next().and_then(glyph).map_or(0.0, |g| {
        g.2.iter()
            .flat_map(|poly| poly.iter().map(|p| p[0]))
            .fold(0.0, f64::max)
    });
    chars.filter_map(glyph).map(|g| g.1).sum::<f64>() + last
}

/// Filled polygons for `text`; `height` is the cap height in canvas units and
/// `(x, y)` the top-left corner.
pub(crate) fn text(
    text: &str,
    (x, y): (f64, f64),
    height: f64,
    fill: Rgb,
    part: Part,
) -> Vec<Shape> {
    let scale = height / GLYPH_HEIGHT;
    let mut shapes = Vec::new();
    let mut cursor = 0.0;
    for glyph in text.chars().filter_map(glyph) {
        for poly in glyph.2 {
            let points = poly
                .iter()
                .map(|p| [x + (cursor + p[0]) * scale, y + p[1] * scale])
                .collect();
            shapes.push(Shape::new(part, points, fill));
        }
        cursor += glyph.1;
    }
    shapes
}

/// The text grown by `grow` canvas units on every side, with round joins: its
/// union is the glyphs shifted around a circle.
pub(crate) fn halo(
    text_: &str,
    at: (f64, f64),
    height: f64,
    grow: f64,
    fill: Rgb,
    part: Part,
) -> Vec<Shape> {
    const STEPS: usize = 16;
    (0..STEPS)
        .flat_map(|i| {
            let a = std::f64::consts::TAU * i as f64 / STEPS as f64;
            let shifted = (at.0 + grow * a.cos(), at.1 + grow * a.sin());
            text(text_, shifted, height, fill, part)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_label_character_has_a_glyph_and_stays_in_its_box() {
        for c in "0123456789URFDLBwx'".chars() {
            let glyph = glyph(c).unwrap_or_else(|| panic!("{c}"));
            assert!(!glyph.2.is_empty() && glyph.1 > 0.0);
            for p in glyph.2.iter().flat_map(|poly| poly.iter()) {
                assert!(p[1] > -0.3 && p[1] < 1.7, "{c} {p:?}");
            }
        }
        assert!(glyph('Q').is_none());
        assert!(measure("3Lw2'") > measure("3Lw2"));
    }

    #[test]
    fn text_stays_in_its_box() {
        let shapes = text("3Rw'", (10.0, 20.0), 14.0, Rgb(0, 0, 0), Part::Label);
        let width = measure("3Rw'") * 10.0;
        for p in shapes.iter().flat_map(|s| s.points.iter()) {
            assert!(p[0] >= 8.0 && p[0] <= 10.0 + width + 2.0, "{p:?}");
            assert!(p[1] >= 17.0 && p[1] <= 20.0 + 14.0 + 2.0, "{p:?}");
        }
    }
}
