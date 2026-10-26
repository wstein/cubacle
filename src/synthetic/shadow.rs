//! The shadow stage: a soft-edged region that takes light from the photo.

use super::color::clamp;
use super::effects::each_pixel;
use super::options::{Shadow, ShadowShape};

const DEFAULT_STRENGTH: f64 = 0.15;
const DEFAULT_SOFTNESS: f64 = 0.03;

/// How far inside the shadow the point is, 0 (outside) to 1 (inside).
fn inside(shape: ShadowShape, tx: f64, ty: f64, width: f64) -> f64 {
    match shape {
        ShadowShape::Band => {
            let t = (tx + ty) / 2.0;
            clamp((t - 0.4) / width, 0.0, 1.0) * clamp((0.65 - t) / width, 0.0, 1.0)
        }
        ShadowShape::Half => clamp(((tx + ty) / 2.0 - 0.5) / width + 0.5, 0.0, 1.0),
        ShadowShape::Spot => {
            let d = ((tx - 0.65) * (tx - 0.65) + (ty - 0.35) * (ty - 0.35)).sqrt();
            clamp((0.25 - d) / width, 0.0, 1.0)
        }
        ShadowShape::Corner => {
            let d = ((tx - 1.0) * (tx - 1.0) + (ty - 1.0) * (ty - 1.0)).sqrt();
            clamp((0.7 - d) / width, 0.0, 1.0)
        }
    }
}

/// Multiplies every channel by `1 - strength * inside`.
pub(crate) fn apply(data: &mut [u8], size: usize, shadow: Shadow) {
    let strength = shadow.strength.unwrap_or(DEFAULT_STRENGTH);
    let shape = shadow.shape.unwrap_or_default();
    let width = shadow.softness.unwrap_or(DEFAULT_SOFTNESS).max(1e-6);
    each_pixel(data, size, |tx, ty, rgb, _| {
        let factor = 1.0 - strength * inside(shape, tx, ty, width);
        rgb.map(|c| c * factor)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_are_dark_where_expected() {
        assert_eq!(inside(ShadowShape::Band, 0.5, 0.5, 0.03), 1.0);
        assert_eq!(inside(ShadowShape::Band, 0.1, 0.1, 0.03), 0.0);
        assert_eq!(inside(ShadowShape::Half, 0.9, 0.9, 0.03), 1.0);
        assert_eq!(inside(ShadowShape::Half, 0.1, 0.1, 0.03), 0.0);
        assert_eq!(inside(ShadowShape::Spot, 0.65, 0.35, 0.03), 1.0);
        assert_eq!(inside(ShadowShape::Corner, 1.0, 1.0, 0.03), 1.0);
        assert_eq!(inside(ShadowShape::Corner, 0.0, 0.0, 0.03), 0.0);
    }

    #[test]
    fn unknown_shape_is_an_error() {
        let error = "ring".parse::<ShadowShape>().unwrap_err();
        assert_eq!(error.to_string(), "Unknown shadow shape: ring");
    }
}
