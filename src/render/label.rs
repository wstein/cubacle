//! The notation: centered below the cube in one fixed glyph size, so `R` and
//! `3Lw2` look alike on one baseline for every icon, in solid ink that fits
//! the backdrop.

use super::font;
use super::scene::{Part, Shape};
use super::style::Backdrop;

/// Glyph units to canvas units for the notation and for the `2x` mark.
pub(crate) const LABEL_SCALE: f64 = 8.0;
/// The cube is centered at this x; the label centers under it.
const CUBE_CENTER: f64 = 50.0;
/// Where the label starts below the cube.
const TOP: f64 = 94.0;

pub(crate) fn shapes(label: &str, backdrop: Backdrop) -> Vec<Shape> {
    let x = CUBE_CENTER - font::measure(label) * LABEL_SCALE / 2.0;
    // The descender needs room for the raster border margin at small sizes.
    let top = if label.contains('y') { TOP - 4.0 } else { TOP };
    font::text(
        label,
        (x, top),
        LABEL_SCALE * font::GLYPH_HEIGHT,
        backdrop.ink(),
        Part::Label,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::scene::{HEIGHT, WIDTH};

    #[test]
    fn rotation_labels_have_glyphs_at_the_same_scale_and_fit() {
        let bounds = |s: &[Shape]| {
            let points: Vec<_> = s.iter().flat_map(|shape| &shape.points).collect();
            assert!(!points.is_empty());
            (
                points.iter().map(|p| p[1]).fold(f64::MAX, f64::min),
                points.iter().map(|p| p[1]).fold(f64::MIN, f64::max),
            )
        };
        let x = shapes("x", Backdrop::Dark);
        let (x_top, x_bottom) = bounds(&x);
        for axis in ["x", "y", "z"] {
            let axis_shapes = shapes(axis, Backdrop::Dark);
            let (top, bottom) = bounds(&axis_shapes);
            let offset = if axis == "y" { 4.0 } else { 0.0 };
            assert!((top + offset - x_top).abs() < 0.2, "{axis}: top {top}");
            assert!(
                ((bottom + offset).min(TOP + LABEL_SCALE * font::GLYPH_HEIGHT) - x_bottom).abs()
                    < 0.2,
                "{axis}: bottom {bottom}"
            );
            for suffix in ["", "2", "'"] {
                let label = format!("{axis}{suffix}");
                let rendered = shapes(&label, Backdrop::Dark);
                assert!(
                    rendered.len() >= axis_shapes.len()
                        && rendered.iter().all(|s| s.part == Part::Label),
                    "{label}"
                );
                for point in rendered.iter().flat_map(|s| &s.points) {
                    assert!(
                        (0.0..=WIDTH).contains(&point[0]) && (0.0..=HEIGHT).contains(&point[1]),
                        "{label}: {point:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_label_has_a_fixed_size_and_fits_the_canvas() {
        assert!((TOP + LABEL_SCALE * font::GLYPH_HEIGHT + 4.0 - HEIGHT).abs() < 0.01);
        let top = |s: &[Shape]| {
            s.iter()
                .flat_map(|p| p.points.iter().map(|q| q[1]))
                .fold(f64::MAX, f64::min)
        };
        let (a, b) = (shapes("R", Backdrop::Dark), shapes("3Lw2'", Backdrop::Dark));
        // The same size: the top edges differ only by glyph shape.
        assert!((top(&a) - top(&b)).abs() < 1.5);
        let right = b.iter().flat_map(|p| p.points.iter().map(|q| q[0]));
        assert!(right.fold(f64::MIN, f64::max) < WIDTH);
        // the longest label of a layer range
        let range = shapes("2-4Lw2'", Backdrop::Dark);
        let right = range.iter().flat_map(|p| p.points.iter().map(|q| q[0]));
        let left = range.iter().flat_map(|p| p.points.iter().map(|q| q[0]));
        assert!(right.fold(f64::MIN, f64::max) < WIDTH);
        assert!(left.fold(f64::MAX, f64::min) > 0.0);
    }
}
