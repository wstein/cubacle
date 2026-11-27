//! The state of a cube written in one of the formats a command can print.
//!
//! One formatter serves `convert` and the `state:` line of the other commands:
//! the Orbit64 V2 color token, URFDLB facelets and sticker colors. `orbit64`
//! and `orbit64v2` both write the color framed entry. A state that a format
//! cannot show is an `Err` with the reason; the caller decides whether that
//! is an error or a note.

use std::fmt;
use std::str::FromStr;

use super::orbit64;
use super::orbit64::v2::{self, Model, Pose};
use super::scheme::ColorScheme;
use crate::puzzle::Facelets;

/// How the state of a cube is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateFormat {
    /// An Orbit64 V2 token, color model, framed pose (the whole-cube
    /// orientation is part of the token).
    Orbit64V2,
    /// The color framed V2 token, under the historical `orbit64` name.
    /// Same bytes as [`StateFormat::Orbit64V2`].
    Orbit64,
    /// Canonical URFDLB facelets.
    Facelets,
    /// W/R/G/Y/O/B sticker letters (boy scheme unless one is given).
    Color,
    /// Nothing: the line is left out.
    None,
}

impl StateFormat {
    /// The name the command line uses.
    pub fn name(self) -> &'static str {
        match self {
            StateFormat::Orbit64V2 => "orbit64v2",
            StateFormat::Orbit64 => "orbit64",
            StateFormat::Facelets => "facelets",
            StateFormat::Color => "color",
            StateFormat::None => "none",
        }
    }
}

impl fmt::Display for StateFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for StateFormat {
    type Err = String;

    fn from_str(text: &str) -> Result<StateFormat, String> {
        [
            StateFormat::Orbit64V2,
            StateFormat::Orbit64,
            StateFormat::Facelets,
            StateFormat::Color,
            StateFormat::None,
        ]
        .into_iter()
        .find(|format| format.name() == text)
        .ok_or_else(|| {
            format!(
                "unknown state format {text:?}; use orbit64v2, orbit64, facelets, color or none"
            )
        })
    }
}

/// The V2 token of a cube state, for the registry entry of its size, `model`
/// and `pose`.
///
/// The color model reads the facelets as they are. The picture model needs the
/// identity and turn of every sticker, which colors do not carry: only the
/// 3x3x3 has pieces that colors tell apart, and its six fixed centers are
/// taken as upright (turn 0), which the turn parity of a picture cube allows
/// for states with an even corner permutation only. Any other size, and a
/// state with an odd corner permutation, is an error that names the reason.
pub fn token_v2(cube: &Facelets, model: Model, pose: Pose) -> Result<String, String> {
    let n = cube.kind().size();
    let entry = v2::entries()
        .iter()
        .find(|entry| entry.n == n && entry.model == model && entry.pose == pose)
        .ok_or_else(|| match (n, model) {
            (2, Model::Picture) => {
                "the 2x2x2 has no picture entry: its picture model is the color model".to_string()
            }
            _ => format!("Orbit64 V2 has no entry for the {n}x{n}x{n}"),
        })?;
    let text = cube.to_string();
    match model {
        Model::Color => v2::facelets::encode(entry.id, &text).map_err(|e| e.to_string()),
        Model::Picture => {
            if n != 3 {
                return Err(format!(
                    "a color cube does not tell the stickers of a {n}x{n}x{n} picture cube apart \
                     (wings, movable centers, center turns); only the 3x3x3 has a picture token \
                     from colors"
                ));
            }
            let mut state = v2::facelets::from_facelets(n, Model::Color, pose, &text)
                .map_err(|e| e.to_string())?;
            state.rotations = vec![0; 6];
            v2::encode(entry.id, &state).map_err(|_| {
                "the center turns of a picture cube are not known from colors, and turn 0 on all \
                 six centers does not fit this state (odd corner permutation)"
                    .to_string()
            })
        }
    }
}

/// The state of `cube` in `format`; `Ok(None)` for [`StateFormat::None`].
/// `scheme` colors the `color` format (boy when `None`).
pub fn state_text(
    cube: &Facelets,
    format: StateFormat,
    scheme: Option<ColorScheme>,
) -> Result<Option<String>, String> {
    Ok(Some(match format {
        StateFormat::None => return Ok(None),
        StateFormat::Orbit64V2 => token_v2(cube, Model::Color, Pose::Framed)?,
        StateFormat::Orbit64 => orbit64::encode(cube).map_err(|e| e.to_string())?,
        StateFormat::Facelets => cube.to_string(),
        StateFormat::Color => scheme.unwrap_or(ColorScheme::WESTERN_BOY).paint(cube),
    }))
}
