//! The notation: centred below the cube in one fixed glyph size, so `R` and
//! `3Lw2` look alike on one baseline for every icon, in solid ink that fits
//! the backdrop.

use super::font;
use super::scene::{Part, Shape};
use super::style::Backdrop;

/// Glyph units to canvas units for the notation and for the `2x` mark.
pub(crate) const LABEL_SCALE: f64 = 8.0;
/// The cube is centred at this x; the label centres under it.
const CUBE_CENTRE: f64 = 50.0;
/// Where the label starts below the cube.
const TOP: f64 = 94.0;

pub(crate) fn shapes(label: &str, backdrop: Backdrop) -> Vec<Shape> {
    let x = CUBE_CENTRE - font::measure(label) * LABEL_SCALE / 2.0;
    font::text(
        label,
        (x, TOP),
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
    }
}
