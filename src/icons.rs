//! The `icon` subcommand and `solve --icons`: move icons on the terminal.
//!
//! This is the only place that looks at the terminal and the environment; the
//! drawing itself lives in `cubacle::render`.

use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use clap::{Args, ValueEnum};
use cubacle::notation::ColourScheme;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use cubacle::render::{icon, Action, Icon, KittyPlacement, Palette, Rgb, Shading, Style};

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
    /// Letters; coloured when the terminal supports it
    Text,
}

#[derive(Args)]
pub struct IconArgs {
    /// Moves or whole-cube rotations, e.g. R U' Rw2 2R x
    #[arg(num_args = 1.., required = true)]
    moves: Vec<String>,
    /// Icon height in pixels (default: 32 per cube layer, at least 96)
    #[arg(long)]
    size: Option<u32>,
    /// Cube size, 2 to 7
    #[arg(long, default_value_t = 3)]
    cube: usize,
    /// Output format
    #[arg(long, value_enum, default_value = "auto")]
    format: Format,
    /// Write to this file instead of standard output
    #[arg(long)]
    out: Option<PathBuf>,
    /// Colour scheme: boy, jp or six letters in URFDLB order
    #[arg(long)]
    scheme: Option<ColourScheme>,
    /// Start from the cube this scramble produces
    #[arg(long)]
    from: Option<String>,
    /// Show the moving layer grey instead of darkening the rest
    #[arg(long)]
    grey: bool,
    /// Keep the background transparent in Kitty graphics (default: opaque
    /// dark, so text under the image cannot show through)
    #[arg(long)]
    transparent: bool,
    /// Draw each move on the cube left by the moves before it (default: each
    /// move on its own start cube, solved or from --from)
    #[arg(long)]
    cumulative: bool,
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

fn detect_stdout() -> Graphics {
    detect(std::io::stdout().is_terminal(), &|name| {
        std::env::var(name).ok().filter(|v| !v.is_empty())
    })
}

/// The backdrop of terminal graphics: opaque, so text below cannot show through.
const TERMINAL_BACKGROUND: Rgb = Rgb(0x20, 0x24, 0x2b);

/// The icon height that keeps stickers readable: 32 pixels per cube layer, at least 96.
fn default_height(cube: usize) -> u32 {
    (32 * u32::try_from(cube).unwrap_or(7)).max(96)
}

fn style(scheme: Option<ColourScheme>, size: u32, grey: bool) -> Style {
    Style {
        palette: Palette::from_letters(scheme.unwrap_or(ColourScheme::WESTERN_BOY).letters()),
        shading: if grey {
            Shading::GreyMoving
        } else {
            Shading::DimRest
        },
        size,
        ..Style::default()
    }
}

/// One icon in the form the terminal can show, with a trailing newline.
fn show(icon: &Icon, graphics: Graphics, size: u32) -> String {
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

pub fn run(args: IconArgs) -> Result<(), String> {
    let kind = PuzzleKind::of_size(args.cube)
        .ok_or_else(|| format!("cube size {} is not between 2 and 7", args.cube))?;
    let mut start = Facelets::solved(kind);
    if let Some(from) = &args.from {
        let moves: MoveSequence = from
            .parse()
            .map_err(|error| format!("--from is not valid move notation ({error})"))?;
        start = start.apply(&moves).map_err(|error| error.to_string())?;
    }
    let mut state = start.clone();
    let actions: Vec<Action> = args
        .moves
        .iter()
        .map(|token| {
            token
                .parse()
                .map_err(|_| format!("{token:?} is not a move or rotation"))
        })
        .collect::<Result<_, _>>()?;
    let height = args.size.unwrap_or_else(|| default_height(args.cube));
    let mut style = style(args.scheme, height, args.grey);
    let to_stdout = args.out.is_none();
    let terminal = std::io::stdout().is_terminal();
    let graphics = if to_stdout {
        detect_stdout()
    } else {
        Graphics::Plain
    };
    let format = match (args.format, &args.out) {
        (Format::Auto, Some(_)) => Format::Svg,
        (Format::Auto, None) if graphics == Graphics::Kitty => Format::Kitty,
        (Format::Auto, None) => Format::Text,
        (other, _) => other,
    };
    if format == Format::Kitty && !args.transparent {
        style.background = Some(TERMINAL_BACKGROUND);
    }
    if matches!(format, Format::Svg | Format::Rgba) && actions.len() != 1 {
        return Err("svg and rgba hold one icon; give one move or use --format text".into());
    }
    let refused = match format {
        Format::Kitty => (!terminal).then_some("graphics escapes are only written to a terminal"),
        Format::Rgba => terminal.then_some("raw pixels would garble the terminal"),
        _ => None,
    };
    if let (true, Some(why)) = (to_stdout, refused) {
        return Err(format!(
            "refusing to write {}: {why}; use --out FILE",
            name(format)
        ));
    }
    let mut bytes: Vec<u8> = Vec::new();
    for action in &actions {
        let drawn = icon(&state, action, &style).map_err(|error| error.to_string())?;
        match format {
            Format::Svg => bytes.extend(drawn.to_svg().into_bytes()),
            Format::Rgba => bytes.extend(drawn.to_rgba(height).data),
            Format::Kitty => bytes.extend(show(&drawn, Graphics::Kitty, height).into_bytes()),
            Format::Text | Format::Auto => {
                let how = if graphics == Graphics::Kitty {
                    Graphics::Ansi
                } else {
                    graphics
                };
                bytes.extend(show(&drawn, how, height).into_bytes());
                bytes.push(b'\n');
            }
        }
        if args.cumulative {
            state = action.apply(&state).map_err(|error| error.to_string())?;
        }
    }
    match &args.out {
        Some(path) => {
            std::fs::write(path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))
        }
        None => std::io::stdout()
            .lock()
            .write_all(&bytes)
            .map_err(|e| format!("cannot write to standard output: {e}")),
    }
}

fn name(format: Format) -> &'static str {
    match format {
        Format::Kitty => "Kitty graphics",
        _ => "RGBA pixels",
    }
}

/// `solve --icons`: each step of the solution with the picture of the cube before it.
pub fn print_solution(
    start: &Facelets,
    solution: &MoveSequence,
    scheme: Option<ColourScheme>,
) -> Result<(), String> {
    let graphics = detect_stdout();
    let mut style = style(scheme, default_height(start.kind().size()), false);
    if graphics == Graphics::Kitty {
        style.background = Some(TERMINAL_BACKGROUND);
    }
    let mut state = start.clone();
    let total = solution.len();
    let mut out = String::new();
    for (index, mv) in solution.moves().iter().enumerate() {
        let action = Action::Layer(*mv);
        let drawn = icon(&state, &action, &style).map_err(|error| error.to_string())?;
        out.push_str(&format!("step {} of {total}: {mv}\n", index + 1));
        out.push_str(&show(&drawn, graphics, style.size));
        if graphics != Graphics::Kitty {
            out.push('\n');
        }
        state = action.apply(&state).map_err(|error| error.to_string())?;
    }
    out.push_str(&format!("{solution}\n"));
    std::io::stdout()
        .lock()
        .write_all(out.as_bytes())
        .map_err(|e| format!("cannot write to standard output: {e}"))
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
    fn a_pipe_never_gets_graphics_or_colour() {
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
