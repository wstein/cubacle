//! Sticker colors: the default palette, the OKLab helpers (section 5) and the
//! per-sticker scatter (section 6).
//!
//! The expressions keep the evaluation order of the reference, left to right,
//! with no fused multiply-add: Rust never contracts floating-point
//! operations on its own.

use super::rng::gauss;

/// A color with channels in 0..255 as doubles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    /// Red.
    pub r: f64,
    /// Green.
    pub g: f64,
    /// Blue.
    pub b: f64,
}

impl Rgb {
    /// A color from its three channels.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Rgb { r, g, b }
    }
}

pub(crate) const BACKDROP: Rgb = Rgb::new(20.0, 20.0, 24.0);
pub(crate) const UNKNOWN_STICKER: Rgb = Rgb::new(128.0, 128.0, 128.0);
pub(crate) const LABEL_COLOR: Rgb = Rgb::new(170.0, 170.0, 176.0);

/// The default palette color of a sticker letter.
pub(crate) fn default_palette(letter: char) -> Option<Rgb> {
    Some(match letter {
        'W' => Rgb::new(232.0, 232.0, 224.0),
        'Y' => Rgb::new(238.0, 206.0, 44.0),
        'O' => Rgb::new(238.0, 122.0, 34.0),
        'R' => Rgb::new(190.0, 36.0, 46.0),
        'G' => Rgb::new(34.0, 150.0, 74.0),
        'B' => Rgb::new(34.0, 84.0, 192.0),
        _ => return None,
    })
}

/// Hue sigma in degrees and chroma coefficient of variation of a letter.
fn scatter_spread(letter: char) -> (f64, f64) {
    match letter {
        'W' => (22.0, 0.35),
        'Y' => (2.5, 0.045),
        'O' => (1.2, 0.034),
        'R' => (1.4, 0.043),
        'G' => (0.7, 0.063),
        'B' => (1.0, 0.08),
        _ => (2.0, 0.05),
    }
}

pub(crate) fn clamp(v: f64, low: f64, high: f64) -> f64 {
    high.min(low.max(v))
}

fn srgb_to_linear(c: f64) -> f64 {
    let v = c / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// OKLCH of a color: lightness, chroma and hue in degrees.
pub(crate) fn rgb_to_oklch(rgb: Rgb) -> (f64, f64, f64) {
    let r = srgb_to_linear(rgb.r);
    let g = srgb_to_linear(rgb.g);
    let b = srgb_to_linear(rgb.b);
    let l_ = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m_ = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s_ = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    let l = 0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.428592205 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.808675766 * s_;
    let c = (a * a + b * b).sqrt();
    let degrees = b.atan2(a) * 180.0 / std::f64::consts::PI;
    (
        l,
        c,
        if degrees < 0.0 {
            degrees + 360.0
        } else {
            degrees
        },
    )
}

fn linear_to_srgb(v: f64) -> f64 {
    let v = clamp(v, 0.0, 1.0);
    let e = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    clamp((e * 255.0).round(), 0.0, 255.0)
}

/// OKLab to sRGB, with whole-number channels.
pub(crate) fn oklab_to_rgb(l: f64, a: f64, b: f64) -> Rgb {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.291485548 * b;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    Rgb::new(
        linear_to_srgb(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
        linear_to_srgb(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
        linear_to_srgb(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
    )
}

/// The color of sticker `k` (`row * n + col`) after the OKLCH round trip with
/// its scatter. The round trip happens with a scale of 0 too.
pub(crate) fn scatter_color(base: Rgb, label: char, seed: i32, k: u64, scale: f64) -> Rgb {
    let (hue_sigma, chroma_cv) = scatter_spread(label);
    let hue_scatter = hue_sigma * scale;
    let chroma_scatter = chroma_cv * scale;
    let (l, c, h) = rgb_to_oklch(base);
    let hue_noise = gauss(seed, 6000 + 2 * k);
    let chroma_noise = gauss(seed, 6001 + 2 * k);
    let hue = h + hue_noise * hue_scatter;
    let chroma = c * (1.0 + chroma_noise * chroma_scatter);
    let radians = hue * std::f64::consts::PI / 180.0;
    let chroma = chroma.max(0.0);
    oklab_to_rgb(l, chroma * radians.cos(), chroma * radians.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channels(c: Rgb) -> [f64; 3] {
        [c.r, c.g, c.b]
    }

    #[test]
    fn oklch_round_trip_without_scatter_stays_within_one_level() {
        for letter in ['W', 'Y', 'O', 'R', 'G', 'B'] {
            let base = default_palette(letter).unwrap();
            let back = scatter_color(base, letter, 7, 3, 0.0);
            for (a, b) in channels(base).into_iter().zip(channels(back)) {
                assert!((a - b).abs() <= 1.0, "{letter}: {base:?} became {back:?}");
            }
        }
    }

    #[test]
    fn scatter_moves_the_color_a_little_and_is_deterministic() {
        let red = default_palette('R').unwrap();
        let a = scatter_color(red, 'R', 7, 4, 1.0);
        assert_eq!(a, scatter_color(red, 'R', 7, 4, 1.0));
        assert_ne!(a, scatter_color(red, 'R', 8, 4, 1.0));
        for (x, y) in channels(red).into_iter().zip(channels(a)) {
            assert!((x - y).abs() < 40.0, "{red:?} became {a:?}");
        }
    }

    #[test]
    fn oklab_conversion_of_gray_is_gray() {
        let gray = oklab_to_rgb(0.6, 0.0, 0.0);
        assert_eq!(gray.r, gray.g);
        assert_eq!(gray.g, gray.b);
    }

    #[test]
    fn oklch_of_black_and_white() {
        let (l, c, _) = rgb_to_oklch(Rgb::new(255.0, 255.0, 255.0));
        assert!((l - 1.0).abs() < 1e-3 && c < 1e-3);
        let (l, _, _) = rgb_to_oklch(Rgb::new(0.0, 0.0, 0.0));
        assert!(l.abs() < 1e-9);
    }
}
