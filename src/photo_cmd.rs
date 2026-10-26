//! The `photo` subcommand: synthetic photos of the six faces of a cube, as
//! PNG files (see `docs/specs/synthetic-face.md`).

use std::path::PathBuf;

use clap::Args;
use cubacle::notation::ColorScheme;
use cubacle::puzzle::Face;
use cubacle::synthetic::{render_face, Options, Preset, Shadow, ShadowShape};

use crate::png_out::encode_rgba;
use crate::{read_input, InputArgs};

/// The seed when none is given, so that two runs write the same files.
const DEFAULT_SEED: i32 = 1;

#[derive(Args)]
pub struct PhotoArgs {
    #[command(flatten)]
    input: InputArgs,
    /// Lighting: clean, scatter, gradient, shadow, dim-warm, glare, noise or bad-camera
    #[arg(long, default_value = "clean")]
    preset: Preset,
    /// Random seed; the same cube, preset, seed and --pixels always give the same files
    #[arg(long, default_value_t = DEFAULT_SEED, allow_negative_numbers = true)]
    seed: i32,
    /// Side of one face photo in pixels (--size is the cube size)
    #[arg(long = "pixels", default_value_t = 256, value_parser = clap::value_parser!(u32).range(16..=4096))]
    pixels: u32,
    /// Write one contact sheet (sheet.png, the six faces as a net) instead of six files
    #[arg(long)]
    sheet: bool,
    /// Directory for the PNG files, created when missing
    #[arg(long, default_value = ".")]
    out: PathBuf,
    /// Scale of the color scatter between stickers (default 1 for the scatter preset, else 0)
    #[arg(long)]
    scatter: Option<f64>,
    /// Gaussian noise on top of the preset, sigma in 0 to 255 levels
    #[arg(long)]
    noise: Option<f64>,
    /// Box blur on top of the preset, radius in pixels
    #[arg(long)]
    blur: Option<f64>,
    /// Factor for the light on every pixel, 0.5 is half as bright
    #[arg(long)]
    brightness: Option<f64>,
    /// How much light a shadow takes, 0 to 1 (default 0.15); any shadow option adds a shadow to the preset
    #[arg(long)]
    shadow_strength: Option<f64>,
    /// Shadow outline: band, half, spot or corner (default band)
    #[arg(long)]
    shadow_shape: Option<ShadowShape>,
    /// Width of the shadow edge as a share of the photo (default 0.03)
    #[arg(long)]
    shadow_softness: Option<f64>,
}

/// Where the face at this index of URFDLB sits in the net of 4 x 3 faces.
fn net_position(face: Face) -> (usize, usize) {
    match face {
        Face::U => (1, 0),
        Face::L => (0, 1),
        Face::F => (1, 1),
        Face::R => (2, 1),
        Face::B => (3, 1),
        Face::D => (1, 2),
    }
}

/// Puts the face photos into one image, the net of the unfolded cube.
fn sheet(photos: &[(Face, Vec<u8>)], size: usize) -> Vec<u8> {
    let width = size * 4;
    let mut out = [20u8, 20, 24, 255].repeat(width * size * 3);
    for (face, data) in photos {
        let (cx, cy) = net_position(*face);
        for y in 0..size {
            let from = y * size * 4;
            let to = ((cy * size + y) * width + cx * size) * 4;
            out[to..to + size * 4].copy_from_slice(&data[from..from + size * 4]);
        }
    }
    out
}

pub fn run(args: PhotoArgs) -> Result<(), String> {
    let (cube, detected) = read_input(&args.input)?;
    let scheme = args
        .input
        .scheme
        .or(detected)
        .unwrap_or(ColorScheme::WESTERN_BOY);
    let n = cube.kind().size();
    let size = args.pixels as usize;
    let shadow = (args.shadow_strength.is_some()
        || args.shadow_shape.is_some()
        || args.shadow_softness.is_some())
    .then_some(Shadow {
        strength: args.shadow_strength,
        shape: args.shadow_shape,
        softness: args.shadow_softness,
    });
    let options = Options {
        scatter: args.scatter,
        noise: args.noise,
        blur: args.blur,
        brightness: args.brightness,
        shadow,
        ..Options::default()
    };
    let mut photos = Vec::new();
    for (index, (face, stickers)) in Face::ALL
        .into_iter()
        .zip(cube.stickers().chunks(n * n))
        .enumerate()
    {
        let grid: Vec<Vec<char>> = stickers
            .chunks(n)
            .map(|row| row.iter().map(|&s| scheme.letter_of(s)).collect())
            .collect();
        // Each face gets its own seed, so the faces do not share one noise.
        let seed = args.seed.wrapping_add(index as i32);
        let data = render_face(&grid, args.preset, seed, size, &options)
            .map_err(|error| error.to_string())?;
        photos.push((face, data));
    }
    std::fs::create_dir_all(&args.out)
        .map_err(|error| format!("cannot create {}: {error}", args.out.display()))?;
    let write = |name: String, width: usize, height: usize, rgba: &[u8]| {
        let path = args.out.join(name);
        std::fs::write(&path, encode_rgba(width as u32, height as u32, rgba))
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        println!("{}", path.display());
        Ok::<(), String>(())
    };
    if args.sheet {
        write(
            "sheet.png".to_owned(),
            size * 4,
            size * 3,
            &sheet(&photos, size),
        )?;
    } else {
        for (face, data) in &photos {
            write(format!("{}.png", face.letter()), size, size, data)?;
        }
    }
    Ok(())
}
