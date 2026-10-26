//! Stages 2 and 3: the lighting of a preset and the option effects. Every
//! stage reads back the stored 8-bit pixels.

use super::draw::store;
use super::options::{Preset, Shadow};
use super::rng::{gauss, unit};
use super::shadow;

/// Applies `f(tx, ty, r, g, b, pixel index)` to every pixel; alpha is kept.
pub(super) fn each_pixel(
    data: &mut [u8],
    size: usize,
    f: impl Fn(f64, f64, [f64; 3], usize) -> [f64; 3],
) {
    let side = size as f64;
    for y in 0..size {
        for x in 0..size {
            let k = y * size + x;
            let i = k * 4;
            let rgb = [
                f64::from(data[i]),
                f64::from(data[i + 1]),
                f64::from(data[i + 2]),
            ];
            let out = f(x as f64 / side, y as f64 / side, rgb, k);
            for (c, value) in out.into_iter().enumerate() {
                data[i + c] = store(value);
            }
        }
    }
}

pub(crate) fn add_noise(data: &mut [u8], size: usize, seed: i32, sigma: f64) {
    each_pixel(data, size, |_, _, rgb, k| {
        let k = k as u64;
        [
            rgb[0] + gauss(seed, 3 * k) * sigma,
            rgb[1] + gauss(seed, 3 * k + 1) * sigma,
            rgb[2] + gauss(seed, 3 * k + 2) * sigma,
        ]
    });
}

pub(crate) fn brightness(data: &mut [u8], size: usize, factor: f64) {
    each_pixel(data, size, |_, _, rgb, _| rgb.map(|c| c * factor));
}

/// A separable box blur of an integer radius, edges replicated: horizontal
/// pass first, then vertical.
pub(crate) fn blur(data: &mut [u8], size: usize, radius: usize) {
    let mut middle = vec![0u8; data.len()];
    blur_pass(data, &mut middle, size, radius, true);
    blur_pass(&middle, data, size, radius, false);
}

fn blur_pass(from: &[u8], into: &mut [u8], size: usize, radius: usize, horizontal: bool) {
    let count = (2 * radius + 1) as f64;
    let last = size as isize - 1;
    for y in 0..size {
        for x in 0..size {
            let mut sums = [0u32; 3];
            for d in -(radius as isize)..=radius as isize {
                let (xx, yy) = if horizontal {
                    ((x as isize + d).clamp(0, last) as usize, y)
                } else {
                    (x, (y as isize + d).clamp(0, last) as usize)
                };
                let j = (yy * size + xx) * 4;
                for (c, sum) in sums.iter_mut().enumerate() {
                    *sum += u32::from(from[j + c]);
                }
            }
            let i = (y * size + x) * 4;
            for (c, sum) in sums.into_iter().enumerate() {
                into[i + c] = store(f64::from(sum) / count);
            }
            into[i + 3] = 255;
        }
    }
}

/// The blur radius of the `bad-camera` preset.
fn lens_radius(size: usize) -> usize {
    ((size as f64 * 0.006).round() as usize).max(1)
}

fn glare(data: &mut [u8], size: usize, seed: i32) {
    let spots: Vec<[f64; 4]> = (0..4u64)
        .map(|j| {
            [
                unit(seed, 5000 + 4 * j),
                unit(seed, 5001 + 4 * j),
                0.06 + unit(seed, 5002 + 4 * j) * 0.08,
                0.05 + unit(seed, 5003 + 4 * j) * 0.07,
            ]
        })
        .collect();
    each_pixel(data, size, |tx, ty, rgb, _| {
        let weight = spots.iter().fold(0.0f64, |acc, &[cx, cy, rx, ry]| {
            let q = (tx - cx) * (tx - cx) / (rx * rx) + (ty - cy) * (ty - cy) / (ry * ry);
            acc.max(if q < 1.0 { 0.7 * (1.0 - q) } else { 0.0 })
        });
        rgb.map(|c| c * (1.0 - weight) + 255.0 * weight)
    });
}

/// The lighting of a preset (stage 2).
pub(crate) fn apply_preset(
    data: &mut [u8],
    size: usize,
    seed: i32,
    preset: Preset,
    shadow: Shadow,
) {
    match preset {
        Preset::Clean | Preset::Scatter => {}
        Preset::Gradient => each_pixel(data, size, |tx, _, [r, g, b], _| {
            [r * (0.7 + 0.6 * tx), g, b * (1.3 - 0.6 * tx)]
        }),
        Preset::Shadow => shadow::apply(data, size, shadow),
        Preset::DimWarm => {
            each_pixel(data, size, |_, _, [r, g, b], _| {
                [r * 0.6, g * 0.5, b * 0.35]
            });
            add_noise(data, size, seed, 8.0);
        }
        Preset::Glare => glare(data, size, seed),
        Preset::Noise => add_noise(data, size, seed, 20.0),
        Preset::BadCamera => {
            blur(data, size, lens_radius(size));
            add_noise(data, size, seed, 15.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_keeps_a_flat_image_and_spreads_an_edge() {
        let size = 8;
        let mut flat = vec![100u8; size * size * 4];
        blur(&mut flat, size, 2);
        assert!(flat
            .chunks(4)
            .all(|p| p[..3] == [100, 100, 100] && p[3] == 255));
        let mut edge = vec![0u8; size * size * 4];
        for y in 0..size {
            for x in 4..size {
                edge[(y * size + x) * 4] = 200;
            }
        }
        blur(&mut edge, size, 1);
        let value = |x: usize| edge[(3 * size + x) * 4];
        assert_eq!(value(0), 0);
        assert!(value(3) > 0 && value(3) < 200);
        assert_eq!(value(7), 200);
    }

    #[test]
    fn brightness_scales_and_clamps() {
        let mut data = vec![100, 200, 50, 255];
        brightness(&mut data, 1, 2.0);
        assert_eq!(data, vec![200, 255, 100, 255]);
    }

    #[test]
    fn noise_is_deterministic_and_seeded() {
        let base = vec![128u8; 4 * 4 * 4];
        let mut a = base.clone();
        let mut b = base.clone();
        let mut c = base;
        add_noise(&mut a, 4, 1, 20.0);
        add_noise(&mut b, 4, 1, 20.0);
        add_noise(&mut c, 4, 2, 20.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
