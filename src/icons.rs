//! Move icons on the terminal for `moves --icons` and `solve --icons`.
//!
//! This is the only place that looks at the terminal and the environment; the
//! drawing itself lives in `cubacle::render`.

use std::io::IsTerminal;

use crate::terminal;
use clap::ValueEnum;
use cubacle::notation::ColorScheme;
use cubacle::render::{Backdrop, Icon, KittyPlacement, Palette, Rgb, Shading, Style};

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Kitty graphics on a capable terminal, otherwise text
    Auto,
    /// A scalable vector image (one move)
    Svg,
    /// Raw RGBA pixels, width x height x 4 bytes (one move)
    Rgba,
    /// Kitty graphics protocol escapes (terminal or --out only)
    Kitty,
    /// Letters; colored when the terminal supports it
    Text,
}

/// What the icon will be seen on.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Background {
    /// Dark backdrop: light notation, resting pieces dimmed to 25 %
    Dark,
    /// Light backdrop: dark notation, resting pieces dimmed to 40 %
    Light,
}

impl Background {
    pub(crate) fn backdrop(self) -> Backdrop {
        match self {
            Background::Dark => Backdrop::Dark,
            Background::Light => Backdrop::Light,
        }
    }

    /// The solid color `--opaque` paints behind the icon.
    pub(crate) fn opaque(self) -> Rgb {
        match self {
            Background::Dark => Rgb(0x20, 0x24, 0x2b),
            Background::Light => Rgb(0xff, 0xff, 0xff),
        }
    }
}

/// Whether the icons carry their move number.
#[derive(Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum Numbers {
    /// On, except in SVG files written by `moves`, which have none
    #[default]
    Auto,
    /// A small grey move number in the top-left corner of every icon
    On,
    /// No numbers
    Off,
}

/// The icon of a move, numbered when asked: `number` counts from 1 over the
/// whole move list, the same index that `solve --now --at-move` takes.
pub(crate) fn numbered(icon: Icon, number: usize, numbers: Numbers, backdrop: Backdrop) -> Icon {
    match numbers {
        Numbers::Auto | Numbers::On => icon.numbered(number, backdrop),
        Numbers::Off => icon,
    }
}

/// The icon of a move with a text label in the corner (`R1`), when numbers are on.
pub(crate) fn labeled(icon: Icon, label: &str, numbers: Numbers, backdrop: Backdrop) -> Icon {
    match numbers {
        Numbers::Auto | Numbers::On => icon.numbered_as(label, backdrop),
        Numbers::Off => icon,
    }
}

/// How standard output can show an icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Graphics {
    Kitty,
    Ansi,
    Plain,
}

/// Decides from the terminal state and the environment; never touches either itself.
pub fn detect(is_terminal: bool, env: &dyn Fn(&str) -> Option<String>) -> Graphics {
    if !is_terminal {
        return Graphics::Plain;
    }
    let term = env("TERM").unwrap_or_default();
    let program = env("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
    if env("KITTY_WINDOW_ID").is_some()
        || term == "xterm-kitty"
        || program == "ghostty"
        || program == "wezterm"
    {
        Graphics::Kitty
    } else if term == "dumb" || env("NO_COLOR").is_some() {
        Graphics::Plain
    } else {
        Graphics::Ansi
    }
}

/// What `--graphics` and `CUBACLE_GRAPHICS` can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum GraphicsMode {
    /// Decide from the environment, then ask the terminal
    #[default]
    Auto,
    /// Kitty graphics protocol images
    Kitty,
    /// Colored text pictures
    Ansi,
    /// Letters only
    Plain,
}

/// An explicit mode wins, also when standard output is not a terminal; `Auto` decides.
pub fn choose(
    mode: GraphicsMode,
    is_terminal: bool,
    env: &dyn Fn(&str) -> Option<String>,
) -> Graphics {
    match mode {
        GraphicsMode::Auto => detect(is_terminal, env),
        GraphicsMode::Kitty => Graphics::Kitty,
        GraphicsMode::Ansi => Graphics::Ansi,
        GraphicsMode::Plain => Graphics::Plain,
    }
}

pub(crate) fn detect_stdout(mode: GraphicsMode) -> Graphics {
    let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let graphics = choose(mode, std::io::stdout().is_terminal(), &env);
    // Only the automatic choice asks, and only where the variables left doubt.
    if mode == GraphicsMode::Auto && graphics == Graphics::Ansi && terminal::supports_kitty() {
        Graphics::Kitty
    } else {
        graphics
    }
}

/// The icon height that keeps stickers readable: 32 pixels per cube layer, at least 96.
pub(crate) fn default_height(cube: usize) -> u32 {
    (32 * u32::try_from(cube).unwrap_or(7)).max(96)
}

pub(crate) fn style(
    scheme: Option<ColorScheme>,
    size: u32,
    grey: bool,
    background: Background,
) -> Style {
    Style {
        palette: Palette::from_letters(scheme.unwrap_or(ColorScheme::WESTERN_BOY).letters()),
        shading: if grey {
            Shading::GreyMoving
        } else {
            Shading::DimRest
        },
        size,
        backdrop: background.backdrop(),
        ..Style::default()
    }
}

/// One icon in the form the terminal can show, with a trailing newline.
pub(crate) fn show(icon: &Icon, graphics: Graphics, size: u32) -> String {
    match graphics {
        Graphics::Kitty => {
            // Erase the rows the image covers so older text cannot show
            // through, step back up, draw without moving the cursor, then step
            // down below the image. The terminal scales to `rows` and keeps
            // the aspect ratio; the 16 px row height is an estimate.
            let rows = size.div_ceil(16).max(2);
            let placement = KittyPlacement {
                rows: Some(rows),
                keep_cursor: true,
            };
            let mut text = "\x1b[2K\n".repeat(rows as usize);
            text.push_str(&format!("\x1b[{rows}A"));
            text.push_str(&icon.to_rgba(size).to_kitty(&placement));
            text.push_str(&format!("\x1b[{rows}B\r"));
            text
        }
        Graphics::Ansi => icon.to_text(true),
        Graphics::Plain => icon.to_text(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn an_explicit_mode_wins_over_the_environment_and_the_pipe() {
        let none = env(&[]);
        let kitty = env(&[("KITTY_WINDOW_ID", "1")]);
        assert_eq!(choose(GraphicsMode::Kitty, false, &none), Graphics::Kitty);
        assert_eq!(choose(GraphicsMode::Ansi, true, &kitty), Graphics::Ansi);
        assert_eq!(choose(GraphicsMode::Plain, true, &kitty), Graphics::Plain);
        assert_eq!(choose(GraphicsMode::Auto, true, &kitty), Graphics::Kitty);
        assert_eq!(choose(GraphicsMode::Auto, false, &kitty), Graphics::Plain);
    }

    #[test]
    fn a_pipe_never_gets_graphics_or_color() {
        let kitty = env(&[("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-kitty")]);
        assert_eq!(detect(false, &kitty), Graphics::Plain);
    }

    #[test]
    fn known_terminals_get_kitty_and_others_fall_back() {
        assert_eq!(
            detect(true, &env(&[("KITTY_WINDOW_ID", "3")])),
            Graphics::Kitty
        );
        assert_eq!(
            detect(true, &env(&[("TERM", "xterm-kitty")])),
            Graphics::Kitty
        );
        assert_eq!(
            detect(true, &env(&[("TERM_PROGRAM", "ghostty")])),
            Graphics::Kitty
        );
        assert_eq!(
            detect(true, &env(&[("TERM_PROGRAM", "WezTerm")])),
            Graphics::Kitty
        );
        assert_eq!(
            detect(true, &env(&[("TERM", "xterm-256color")])),
            Graphics::Ansi
        );
        assert_eq!(detect(true, &env(&[("TERM", "dumb")])), Graphics::Plain);
        assert_eq!(detect(true, &env(&[("NO_COLOR", "1")])), Graphics::Plain);
    }
}
