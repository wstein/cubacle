//! The state line: `state: <format> <value>` on standard error for the cube a
//! command starts from, `target: ...` for the cube it aims at.
//!
//! Standard output is not touched, so pipes (`scramble | solve -`) keep working.
//! A state the format cannot show gives `state: unavailable: <reason>`; the
//! command goes on and its exit code is unchanged.

use clap::Args;
use cubacle::notation::{state_text, ColorScheme, StateFormat};
use cubacle::puzzle::Facelets;

/// The `--state-format` option, shared by the commands that print a state line.
#[derive(Args, Clone, Copy)]
pub(crate) struct StateOpts {
    /// Format of the state line on stderr (the cube the command starts from,
    /// and with --to the target): orbit64v2 (Orbit64 V2 token, color model,
    /// framed pose), orbit64 (V1 token), facelets, color, or none for no line
    #[arg(long, value_name = "FORMAT", default_value = "orbit64v2")]
    pub(crate) state_format: StateFormat,
}

impl StateOpts {
    /// The line for the cube the command starts from.
    pub(crate) fn start(&self, cube: &Facelets, scheme: Option<ColorScheme>) {
        self.line("state", cube, scheme);
    }

    /// The line for the cube the command aims at.
    pub(crate) fn target(&self, cube: &Facelets, scheme: Option<ColorScheme>) {
        self.line("target", cube, scheme);
    }

    /// A line under another label (`now` for the cube of `solve --now`).
    pub(crate) fn line(&self, label: &str, cube: &Facelets, scheme: Option<ColorScheme>) {
        if let Some(text) = self.text(label, cube, scheme) {
            eprintln!("{text}");
        }
    }

    /// A line for a state that is not one cube, with the reason.
    pub(crate) fn unavailable(&self, label: &str, reason: &str) {
        if self.state_format != StateFormat::None {
            eprintln!("{label}: unavailable: {reason}");
        }
    }

    /// The text of the line; `None` for `--state-format none`.
    pub(crate) fn text(
        &self,
        label: &str,
        cube: &Facelets,
        scheme: Option<ColorScheme>,
    ) -> Option<String> {
        match state_text(cube, self.state_format, scheme) {
            Ok(Some(value)) => Some(format!("{label}: {} {value}", self.state_format)),
            Ok(None) => None,
            Err(reason) => Some(format!("{label}: unavailable: {reason}")),
        }
    }
}

/// The state in every format as one JSON object, keys in the order
/// `orbit64v2`, `orbit64`, `facelets`, `color`; a format that cannot show the
/// state is `null`.
pub(crate) fn json(cube: &Facelets) -> String {
    let mut out = String::new();
    let mut object = cubacle::report::Object::open(&mut out);
    for format in [
        StateFormat::Orbit64V2,
        StateFormat::Orbit64,
        StateFormat::Facelets,
        StateFormat::Color,
    ] {
        match state_text(cube, format, None) {
            Ok(Some(text)) => object.string(format.name(), &text),
            _ => object.value(format.name(), |out| out.push_str("null")),
        }
    }
    object.close();
    out
}
