//! The presets and options of a synthetic face.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use super::color::Rgb;

/// The lighting of a photo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    /// Plain light, no scatter.
    Clean,
    /// Clean light with the sticker-to-sticker color scatter of real captures.
    Scatter,
    /// Red rises and blue falls from left to right.
    Gradient,
    /// A soft-edged diagonal band, dimmer than its surroundings.
    Shadow,
    /// Little light, and a warm one, with noise.
    DimWarm,
    /// Four soft highlights toward white.
    Glare,
    /// Gaussian sensor noise.
    Noise,
    /// A soft lens and noise.
    BadCamera,
}

impl Preset {
    /// All presets, in documentation order.
    pub const ALL: [Preset; 8] = [
        Preset::Clean,
        Preset::Scatter,
        Preset::Gradient,
        Preset::Shadow,
        Preset::DimWarm,
        Preset::Glare,
        Preset::Noise,
        Preset::BadCamera,
    ];

    /// The name used on the command line and in the label.
    pub fn name(self) -> &'static str {
        match self {
            Preset::Clean => "clean",
            Preset::Scatter => "scatter",
            Preset::Gradient => "gradient",
            Preset::Shadow => "shadow",
            Preset::DimWarm => "dim-warm",
            Preset::Glare => "glare",
            Preset::Noise => "noise",
            Preset::BadCamera => "bad-camera",
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Preset {
    type Err = SyntheticError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Preset::ALL
            .into_iter()
            .find(|preset| preset.name() == text)
            .ok_or_else(|| SyntheticError::UnknownPreset(text.to_owned()))
    }
}

/// Why a photo cannot be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntheticError {
    /// The preset name is not one of the eight.
    UnknownPreset(String),
    /// The grid is empty or not square.
    BadGrid,
    /// The photo side is zero.
    ZeroSize,
}

impl fmt::Display for SyntheticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntheticError::UnknownPreset(name) => {
                write!(f, "Unknown synthetic preset: {name}")
            }
            SyntheticError::BadGrid => f.write_str("the grid must be square and not empty"),
            SyntheticError::ZeroSize => f.write_str("the photo size must be at least 1"),
        }
    }
}

impl std::error::Error for SyntheticError {}

/// A round patch of a palette color on one sticker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Logo {
    /// Row of the sticker, from the top.
    pub row: usize,
    /// Column of the sticker, from the left.
    pub col: usize,
    /// Sticker letter whose palette color the patch takes.
    pub color: char,
    /// Radius as a share of the sticker's half width (default 0.5).
    pub size: Option<f64>,
}

/// The shape of the face as shares: the margin of the photo, the gap between
/// stickers and their corner radius (both of a sticker).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geometry {
    /// Margin, share of the photo side (default 0.035).
    pub margin: Option<f64>,
    /// Gap, share of a sticker (default 0.07).
    pub gap: Option<f64>,
    /// Corner radius, share of a sticker (default 0.14).
    pub radius: Option<f64>,
}

/// Everything beyond grid, preset, seed and size; all fields are optional.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Options {
    /// Scale of the hue and chroma scatter (default 1 for `Scatter`, else 0).
    pub scatter: Option<f64>,
    /// Scale of the brightness jitter between stickers (default 1).
    pub lightness: Option<f64>,
    /// Sticker colors by letter, merged over the default palette.
    pub palette: BTreeMap<char, Rgb>,
    /// A logo patch.
    pub logo: Option<Logo>,
    /// Gaussian noise on top of the preset, sigma in 0..255 levels.
    pub noise: Option<f64>,
    /// Box blur radius in pixels on top of the preset (used when at least 1).
    pub blur: Option<f64>,
    /// Factor for the light on every pixel.
    pub brightness: Option<f64>,
    /// Face shape.
    pub geometry: Geometry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for preset in Preset::ALL {
            assert_eq!(preset.name().parse::<Preset>().unwrap(), preset);
        }
    }

    #[test]
    fn unknown_preset_is_an_error() {
        let error = "real".parse::<Preset>().unwrap_err();
        assert_eq!(error.to_string(), "Unknown synthetic preset: real");
    }
}
