//! Brackets above a row of icons: one thin bracket with a number over every
//! group, and a thicker bracket over a block of groups.

use super::font;
use super::raster::{rasterise_canvas, Raster};
use super::scene::{Part, Shape};
use super::style::{Backdrop, Rgb};

/// A group of icons in a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpan {
    /// Pixels from the left edge of the row.
    pub x: u32,
    pub width: u32,
    /// The number written above the bracket.
    pub label: Option<String>,
}

/// A run of groups under one outer bracket (or none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpan {
    pub outer: bool,
    pub units: Vec<UnitSpan>,
}

/// Pixels per design unit: 1 at an icon height of 96.
fn unit(size: u32) -> f64 {
    (f64::from(size) / 96.0).max(1.0)
}

/// The height of the bracket band for icons of `size`; the outer lane is only
/// there when `outer` is set.
pub fn bracket_height(size: u32, outer: bool) -> u32 {
    let t = unit(size);
    let lanes = if outer { 30.0 } else { 22.0 };
    (lanes * t).round() as u32
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64, fill: Rgb) -> Shape {
    Shape::new(
        Part::Frame,
        vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        fill,
    )
}

/// The brackets of one row, `width` pixels wide, as a transparent raster to
/// put above the icons.
pub fn brackets(
    blocks: &[BlockSpan],
    width: u32,
    size: u32,
    outer: bool,
    backdrop: Backdrop,
) -> Raster {
    let t = unit(size);
    let height = bracket_height(size, outer);
    let ink = match backdrop {
        Backdrop::Dark => Rgb(0xb4, 0xbc, 0xc8),
        Backdrop::Light => Rgb(0x4a, 0x52, 0x5e),
    };
    // lanes from the top: outer line, [numbers], unit line
    let base = if outer { 8.0 * t } else { 0.0 };
    let text_top = base + 1.0 * t;
    let text_height = 11.0 * t;
    let unit_line = base + 14.0 * t;
    let mut shapes = Vec::new();
    for block in blocks {
        if block.outer {
            if let (Some(first), Some(last)) = (block.units.first(), block.units.last()) {
                let (x0, x1) = (f64::from(first.x), f64::from(last.x + last.width));
                let thick = 3.0 * t;
                shapes.push(rect(x0, 0.0, x1, thick, ink));
                shapes.push(rect(x0, 0.0, x0 + thick, 7.0 * t, ink));
                shapes.push(rect(x1 - thick, 0.0, x1, 7.0 * t, ink));
            }
        }
        for unit_span in &block.units {
            let (x0, x1) = (
                f64::from(unit_span.x),
                f64::from(unit_span.x + unit_span.width),
            );
            let thin = 1.6 * t;
            shapes.push(rect(x0, unit_line, x1, unit_line + thin, ink));
            shapes.push(rect(x0, unit_line, x0 + thin, unit_line + 6.0 * t, ink));
            shapes.push(rect(x1 - thin, unit_line, x1, unit_line + 6.0 * t, ink));
            if let Some(label) = &unit_span.label {
                let scale = text_height / font::GLYPH_HEIGHT;
                let at = (
                    x0 + (x1 - x0 - font::measure(label) * scale) / 2.0,
                    text_top,
                );
                shapes.extend(font::text(label, at, text_height, ink, Part::Label));
            }
        }
    }
    rasterise_canvas(&shapes, (f64::from(width), f64::from(height)), height, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_are_drawn_inside_the_row_and_leave_the_rest_clear() {
        let blocks = [BlockSpan {
            outer: true,
            units: vec![UnitSpan {
                x: 10,
                width: 80,
                label: Some("12".into()),
            }],
        }];
        let r = brackets(&blocks, 100, 96, true, Backdrop::Dark);
        assert_eq!((r.width, r.height), (100, bracket_height(96, true)));
        let alpha = |x: usize, y: usize| r.data[(y * 100 + x) * 4 + 3];
        assert_eq!(alpha(50, 1), 255, "outer line");
        assert_eq!(alpha(2, 1), 0, "left of the block is clear");
        assert_eq!(alpha(50, 8 + 14), 255, "unit line");
    }

    #[test]
    fn the_band_grows_with_the_icons() {
        assert_eq!(bracket_height(96, false), 22);
        assert_eq!(bracket_height(96, true), 30);
        assert_eq!(bracket_height(192, true), 60);
    }
}
