//! Stage 1: the face itself, before any lighting.

use std::collections::BTreeMap;

use super::color::{clamp, default_palette, scatter_color, Rgb, BACKDROP, UNKNOWN_STICKER};
use super::options::{Geometry, Logo};
use super::rng::unit;

pub(crate) const DEFAULT_MARGIN: f64 = 0.035;
const DEFAULT_GAP: f64 = 0.07;
const DEFAULT_RADIUS: f64 = 0.14;

/// Stores a float as a pixel byte: NaN is 0, others are clamped to 0..255
/// and rounded to the nearest whole number.
pub(crate) fn store(value: f64) -> u8 {
    if value.is_nan() {
        0
    } else {
        value.clamp(0.0, 255.0).round() as u8
    }
}

pub(crate) fn put(data: &mut [u8], index: usize, color: Rgb) {
    data[index] = store(color.r);
    data[index + 1] = store(color.g);
    data[index + 2] = store(color.b);
    data[index + 3] = 255;
}

/// The margin in whole pixels.
pub(crate) fn margin_pixels(size: usize, geometry: &Geometry) -> usize {
    (size as f64 * geometry.margin.unwrap_or(DEFAULT_MARGIN)).round() as usize
}

/// The parameters of one face.
pub(crate) struct Face<'a> {
    pub grid: &'a [Vec<char>],
    pub seed: i32,
    pub size: usize,
    pub scatter: f64,
    pub lightness: f64,
    pub palette: &'a BTreeMap<char, Rgb>,
    pub logo: Option<&'a Logo>,
    pub geometry: &'a Geometry,
}

impl Face<'_> {
    fn color_of(&self, letter: char) -> Rgb {
        self.palette
            .get(&letter)
            .copied()
            .or_else(|| default_palette(letter))
            .unwrap_or(UNKNOWN_STICKER)
    }

    /// Draws the face into `data` (RGBA, `size * size * 4` bytes).
    pub(crate) fn draw(&self, data: &mut [u8]) {
        let n = self.grid.len();
        let size = self.size;
        let side = size as f64;
        let margin = margin_pixels(size, self.geometry) as f64;
        let inner = side - 2.0 * margin;
        let cell = inner / n as f64;
        let gap = (cell * self.geometry.gap.unwrap_or(DEFAULT_GAP)).max(2.0);
        let radius = cell * self.geometry.radius.unwrap_or(DEFAULT_RADIUS);
        let half = cell / 2.0 - gap / 2.0;
        let corner = half - radius;
        // Colors of the stickers do not depend on the pixel: scatter once.
        let bases: Vec<Rgb> = (0..n * n)
            .map(|k| {
                let letter = self.grid[k / n][k % n];
                scatter_color(
                    self.color_of(letter),
                    letter,
                    self.seed,
                    k as u64,
                    self.scatter,
                )
            })
            .collect();
        for y in 0..size {
            for x in 0..size {
                let fx = x as f64 + 0.5 - margin;
                let fy = y as f64 + 0.5 - margin;
                let color = if fx < 0.0 || fy < 0.0 || fx >= inner || fy >= inner {
                    BACKDROP
                } else {
                    let col = ((fx / cell).floor() as usize).min(n - 1);
                    let row = ((fy / cell).floor() as usize).min(n - 1);
                    let dx = fx - (col as f64 + 0.5) * cell;
                    let dy = fy - (row as f64 + 0.5) * cell;
                    let qx = dx.abs() - corner;
                    let qy = dy.abs() - corner;
                    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
                    let distance = outside + qx.max(qy).min(0.0) - radius;
                    let coverage = clamp(0.5 - distance, 0.0, 1.0);
                    let k = row * n + col;
                    let mut base = bases[k];
                    if let Some(logo) = self.logo {
                        if logo.row == row && logo.col == col {
                            let reach = logo.size.unwrap_or(0.5) * half;
                            if (dx * dx + dy * dy).sqrt() <= reach {
                                base = self.color_of(logo.color);
                            }
                        }
                    }
                    let rim = 1.0 - 0.1 * clamp(1.0 - -distance / (cell * 0.08), 0.0, 1.0);
                    let slope = 1.0 + 0.04 * (-(dx + dy) / cell);
                    let jitter =
                        1.0 + (unit(self.seed, k as u64 + 1000) - 0.5) * 0.05 * self.lightness;
                    let shade = rim * slope * jitter;
                    Rgb::new(
                        BACKDROP.r * (1.0 - coverage) + base.r * shade * coverage,
                        BACKDROP.g * (1.0 - coverage) + base.g * shade * coverage,
                        BACKDROP.b * (1.0 - coverage) + base.b * shade * coverage,
                    )
                };
                let cx = x as f64 / side - 0.5;
                let cy = y as f64 / side - 0.5;
                let falloff = 1.0 - 0.28 * (cx * cx + cy * cy);
                let shaded = Rgb::new(color.r * falloff, color.g * falloff, color.b * falloff);
                put(data, (y * size + x) * 4, shaded);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::store;

    #[test]
    fn store_clamps_rounds_and_maps_nan_to_zero() {
        assert_eq!(store(f64::NAN), 0);
        assert_eq!(store(-5.0), 0);
        assert_eq!(store(300.0), 255);
        assert_eq!(store(10.4), 10);
        assert_eq!(store(10.6), 11);
    }
}
