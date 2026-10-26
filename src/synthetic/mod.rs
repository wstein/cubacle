//! Synthetic cube face photos: a deterministic renderer that turns a grid of
//! sticker letters into an RGBA image (`docs/specs/synthetic-face.md`).
//!
//! The photo looks like a cropped camera shot: rounded stickers on a dark
//! backdrop, a thin border, a vignette, and a strip that says
//! `SYNTHETIC <PRESET>`. The module is pure and depends on nothing else in
//! the crate.
//!
//! # Guarantee
//!
//! The same binary with the same `(grid, preset, seed, size, options)` always
//! returns the same bytes. The random numbers come from an integer hash that
//! is identical everywhere, but the colors and the noise use `f64` functions
//! of the standard library (`powf`, `cbrt`, `atan2`, `ln`, `cos`); another
//! platform or libm may differ in the last bit and so in a few pixels by one
//! level. The tests pin the output of this implementation on the platforms of
//! the CI.
//!
//! ```
//! use cubacle::synthetic::{render_face, Options, Preset};
//!
//! let grid: Vec<Vec<char>> = ["WY", "RG"].iter().map(|r| r.chars().collect()).collect();
//! let rgba = render_face(&grid, Preset::Clean, 7, 64, &Options::default()).unwrap();
//! assert_eq!(rgba.len(), 64 * 64 * 4);
//! ```

mod color;
mod draw;
mod effects;
mod label;
mod options;
mod rng;

pub use color::Rgb;
pub use options::{Geometry, Logo, Options, Preset, SyntheticError};

/// Renders one face as `size * size * 4` bytes of RGBA, row-major, top row
/// first. `grid` holds the sticker letters, top row first; `W Y O R G B` are
/// known colors and any other letter is drawn gray.
pub fn render_face(
    grid: &[Vec<char>],
    preset: Preset,
    seed: i32,
    size: usize,
    options: &Options,
) -> Result<Vec<u8>, SyntheticError> {
    if grid.is_empty() || grid.iter().any(|row| row.len() != grid.len()) {
        return Err(SyntheticError::BadGrid);
    }
    if size == 0 {
        return Err(SyntheticError::ZeroSize);
    }
    let mut data = vec![0u8; size * size * 4];
    let face = draw::Face {
        grid,
        seed,
        size,
        scatter: options
            .scatter
            .unwrap_or(if preset == Preset::Scatter { 1.0 } else { 0.0 }),
        lightness: options.lightness.unwrap_or(1.0),
        palette: &options.palette,
        logo: options.logo.as_ref(),
        geometry: &options.geometry,
    };
    face.draw(&mut data);
    effects::apply_preset(&mut data, size, seed, preset);
    if let Some(factor) = options.brightness {
        effects::brightness(&mut data, size, factor);
    }
    if let Some(radius) = options.blur.filter(|&radius| radius >= 1.0) {
        effects::blur(&mut data, size, radius.round() as usize);
    }
    if let Some(sigma) = options.noise.filter(|&sigma| sigma > 0.0) {
        effects::add_noise(&mut data, size, seed.wrapping_add(7919), sigma);
    }
    label::draw_label(
        &mut data,
        size,
        preset.name(),
        draw::margin_pixels(size, &options.geometry),
    );
    Ok(data)
}
