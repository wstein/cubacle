//! The `moves` subcommand: the plain move sequence an expression stands for,
//! or (`--icons`) the moves drawn as pictures.

use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;

use clap::Args;
use cubacle::notation::{parse_expression, to_wca_token, ColourScheme, Notation};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use cubacle::render::{icon, Action};

use crate::icons::{default_height, detect_stdout, show, style, Background, Format, Graphics};
use crate::steps::{PictureArgs, Pictures};

#[derive(Args)]
pub struct MovesArgs {
    /// A move expression such as "(R2 F2 L2)*2" or "[R: U]"; spaces are allowed
    /// and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., required = true)]
    expression: Vec<String>,
    /// Cube size, 2 to 7
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// How to write the moves: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
    #[command(flatten)]
    pictures: PictureArgs,
    /// With --icons, the output format
    #[arg(long, requires = "icons", value_enum, default_value = "auto")]
    format: Format,
    /// With --icons, write to this file instead of standard output
    #[arg(long, requires = "icons")]
    out: Option<PathBuf>,
    /// With --icons, the colour scheme: boy, jp or six letters in URFDLB order
    #[arg(long, requires = "icons")]
    scheme: Option<ColourScheme>,
    /// With --icons, start from the cube this scramble produces
    #[arg(long, requires = "icons")]
    from: Option<String>,
    /// With --icons, show the moving layer grey instead of darkening the rest
    #[arg(long, requires = "icons")]
    grey: bool,
    /// With --icons, paint a solid background of the --background colour behind
    /// SVG, RGBA and Kitty graphics (default: transparent)
    #[arg(long, requires = "icons")]
    opaque: bool,
    /// With --icons and an explicit --format or --out, draw each move on the
    /// cube left by the moves before it (default: each move on its own start cube)
    #[arg(long, requires = "icons")]
    cumulative: bool,
}

/// The expression words joined, or standard input for a lone "-".
fn read_text(words: &[String]) -> Result<String, String> {
    match words {
        [single] if single == "-" => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| format!("cannot read stdin: {error}"))?;
            Ok(text)
        }
        parts => Ok(parts.join(" ")),
    }
}

fn kind_of(size: usize) -> Result<PuzzleKind, String> {
    PuzzleKind::of_size(size).ok_or_else(|| format!("size must be between 2 and 7, not {size}"))
}

/// The moves of an expression on a cube of `kind`, checked for that size.
fn expand(text: &str, kind: PuzzleKind) -> Result<MoveSequence, String> {
    parse_expression(text, kind).map_err(|error| error.to_string())
}

pub fn run(args: MovesArgs) -> Result<(), String> {
    let kind = kind_of(args.size)?;
    let text = read_text(&args.expression)?;
    let moves = expand(&text, kind)?;
    if args.pictures.icons {
        return draw(&args, kind, &text, &moves);
    }
    println!("{}", args.notation.write(kind, &moves));
    Ok(())
}

/// The icons the text stands for: when every word is a move or rotation (WCA
/// or SiGN), one icon each, so rotations keep their own picture; otherwise
/// one per move of the expanded expression.
fn actions_of(text: &str, moves: &MoveSequence) -> Vec<Action> {
    let words: Option<Vec<Action>> = text
        .split_whitespace()
        .map(|word| {
            to_wca_token(word)
                .ok()
                .and_then(|wca| wca.parse::<Action>().ok())
        })
        .collect();
    words.unwrap_or_else(|| moves.moves().iter().map(|mv| Action::Layer(*mv)).collect())
}

fn draw(
    args: &MovesArgs,
    kind: PuzzleKind,
    text: &str,
    moves: &MoveSequence,
) -> Result<(), String> {
    let mut start = Facelets::solved(kind);
    if let Some(from) = &args.from {
        let scramble = expand(from, kind)
            .map_err(|error| format!("--from is not valid move notation ({error})"))?;
        start = start.apply(&scramble).map_err(|error| error.to_string())?;
    }
    let pictures = &args.pictures;
    let to_stdout = args.out.is_none();
    if args.format == Format::Auto && to_stdout {
        // Step pictures, as `solve --icons` shows them.
        let options = pictures.options(args.scheme, args.notation);
        let steps = Pictures::new(args.size, &options).steps(&start, moves, &[])?;
        return std::io::stdout()
            .lock()
            .write_all(steps.as_bytes())
            .map_err(|e| format!("cannot write to standard output: {e}"));
    }
    let actions = actions_of(text, moves);
    let height = pictures
        .icon_size
        .unwrap_or_else(|| default_height(args.size));
    let background = pictures.background.unwrap_or(Background::Dark);
    let mut style = style(args.scheme, height, args.grey, background);
    let terminal = std::io::stdout().is_terminal();
    let graphics = if to_stdout {
        detect_stdout(pictures.graphics)
    } else {
        Graphics::Plain
    };
    let format = match (args.format, &args.out) {
        (Format::Auto, _) => Format::Svg,
        (other, _) => other,
    };
    if args.opaque && matches!(format, Format::Kitty | Format::Svg | Format::Rgba) {
        style.background = Some(background.opaque());
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
    let mut state = start;
    let mut bytes: Vec<u8> = Vec::new();
    for action in &actions {
        let drawn = icon(&state, action, &style).map_err(|error| error.to_string())?;
        match format {
            Format::Svg | Format::Auto => bytes.extend(drawn.to_svg().into_bytes()),
            Format::Rgba => bytes.extend(drawn.to_rgba(height).data),
            Format::Kitty => bytes.extend(show(&drawn, Graphics::Kitty, height).into_bytes()),
            Format::Text => {
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
