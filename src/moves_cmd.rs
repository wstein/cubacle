//! The `moves` subcommand: the plain move sequence an expression stands for,
//! or (`--icons`) the moves drawn as pictures.

use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;

use clap::Args;
use cubacle::notation::{
    axis_units, parse_expression, parse_grouped, to_wca_token, ColorScheme, Item, Notation,
};
use cubacle::puzzle::{Facelets, Move, MoveSequence, PuzzleKind, Turn};
use cubacle::ranges::{compress, compressed_text, Optimize};
use cubacle::render::{icon, Action};

use crate::icons::{
    default_height, detect_stdout, numbered, show, style, Background, Format, Graphics, Numbers,
};
use crate::metrics_out;
use crate::steps::{end_line, Layout, PictureArgs, Pictures};

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
    /// Print all seven move counts (htm qtm stm etm obtm btm atm) on stderr
    /// instead of the usual headline ones
    #[arg(long)]
    metrics: bool,
    /// Rewrite the moves for a metric: quarter writes every half turn as two
    /// quarter turns (R2 becomes R R), the form to count in QTM
    #[arg(long, value_enum, conflicts_with_all = ["icons", "optimize"])]
    expand: Option<Expand>,
    /// Write adjacent turns of one axis as layer ranges where that is shorter
    /// (`ranges`, as in `solve`)
    #[arg(long, value_name = "WHAT", conflicts_with = "icons")]
    optimize: Option<Optimize>,
    #[command(flatten)]
    pictures: PictureArgs,
    /// Hand units: dots (default) draws the groups marked by "·" and "( )",
    /// none draws one icon per move (both with --icons), axis ignores the marks
    /// and groups turns of one axis with disjoint layers (U D', R L); without
    /// --icons it writes the moves with " · " between those units
    #[arg(long, value_enum, default_value_t)]
    group: GroupMode,
    /// With --icons, the output format
    #[arg(long, requires = "icons", value_enum, default_value = "auto")]
    format: Format,
    /// With --icons, write to this file instead of standard output
    #[arg(long, requires = "icons")]
    out: Option<PathBuf>,
    /// With --icons, the color scheme: boy, jp or six letters in URFDLB order
    #[arg(long, requires = "icons")]
    scheme: Option<ColorScheme>,
    /// With --icons, start from the cube this scramble produces
    #[arg(long, requires = "icons")]
    from: Option<String>,
    /// With --icons, show the moving layer grey instead of darkening the rest
    #[arg(long, requires = "icons")]
    grey: bool,
    /// With --icons, paint a solid background of the --background color behind
    /// SVG, RGBA and Kitty graphics (default: transparent)
    #[arg(long, requires = "icons")]
    opaque: bool,
    /// With --icons and an explicit --format or --out, draw each move on the
    /// cube left by the moves before it (default: each move on its own start cube)
    #[arg(long, requires = "icons")]
    cumulative: bool,
}

/// What `moves --expand` rewrites.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Expand {
    /// Every half turn as two quarter turns
    Quarter,
}

/// The same moves with each half turn written as two quarter turns.
fn quarter_turns(moves: &MoveSequence) -> MoveSequence {
    moves
        .moves()
        .iter()
        .flat_map(|mv| {
            let count = if mv.turn == Turn::Half { 2 } else { 1 };
            let turn = if mv.turn == Turn::Half {
                Turn::Clockwise
            } else {
                mv.turn
            };
            std::iter::repeat_n(Move::new(mv.face, mv.depth, mv.wide, turn), count)
        })
        .collect()
}

/// How `moves --icons` treats `·` and `( )` in the input.
#[derive(Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum GroupMode {
    /// Draw the groups with brackets and numbers
    #[default]
    Dots,
    /// One icon per move, as without the dots
    None,
    /// Turns about one axis on disjoint layers are one unit, as in the ATM metric
    Axis,
}

/// The expression words joined, or standard input for a lone "-".
pub(crate) fn read_text(words: &[String]) -> Result<String, String> {
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

pub(crate) fn kind_of(size: usize) -> Result<PuzzleKind, String> {
    PuzzleKind::of_size(size).ok_or_else(|| format!("size must be between 2 and 7, not {size}"))
}

/// The moves of an expression on a cube of `kind`, checked for that size.
pub(crate) fn expand(text: &str, kind: PuzzleKind) -> Result<MoveSequence, String> {
    parse_expression(text, kind).map_err(|error| error.to_string())
}

pub fn run(args: MovesArgs) -> Result<(), String> {
    let kind = kind_of(args.size)?;
    let text = read_text(&args.expression)?;
    let moves = expand(&text, kind)?;
    if !args.pictures.icons && args.group != GroupMode::Axis && args.group != GroupMode::default() {
        return Err(
            "--group dots and --group none need --icons; without it only --group axis applies"
                .into(),
        );
    }
    if args.group == GroupMode::Axis && !args.pictures.icons {
        if args.expand.is_some() || args.optimize.is_some() {
            return Err("--group axis cannot be combined with --expand or --optimize".into());
        }
        let tree = axis_units(moves.moves(), kind, &[]);
        let units: Vec<String> = tree
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Group(unit) => Some(
                    args.notation
                        .write(kind, &MoveSequence::new(unit.flatten())),
                ),
                Item::Move(_) => None,
            })
            .collect();
        println!("{}", units.join(" · "));
        eprintln!(
            "{}",
            metrics_out::text_line(&text, kind, &moves, args.metrics)
        );
        return Ok(());
    }
    if args.pictures.icons {
        if args.metrics {
            eprintln!("{}", metrics_out::text_line(&text, kind, &moves, true));
        }
        return draw(&args, kind, &text, &moves);
    }
    if args.optimize == Some(Optimize::Ranges) {
        let (written, _) = compressed_text(args.notation, kind, &moves);
        println!("{written}");
        let packed = compress(kind, &moves);
        eprintln!(
            "{}",
            if args.metrics {
                metrics_out::compressed_all_line(kind, &packed)
            } else {
                metrics_out::compressed_line(kind, &packed)
            }
        );
        return Ok(());
    }
    if args.expand == Some(Expand::Quarter) {
        let quarters = quarter_turns(&moves);
        println!("{}", args.notation.write(kind, &quarters));
        eprintln!(
            "{}",
            if args.metrics {
                metrics_out::all_line(kind, &quarters)
            } else {
                metrics_out::line(kind, &quarters)
            }
        );
        return Ok(());
    }
    println!("{}", args.notation.write(kind, &moves));
    eprintln!(
        "{}",
        metrics_out::text_line(&text, kind, &moves, args.metrics)
    );
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
        let mut pictures = Pictures::new(args.size, &options);
        if args.group == GroupMode::Axis {
            let tree = axis_units(moves.moves(), kind, &[]);
            pictures = pictures.grouped(Layout::of(&tree));
        } else if args.group == GroupMode::Dots && (text.contains('·') || text.contains('(')) {
            let tree = parse_grouped(text, kind).map_err(|error| error.to_string())?;
            pictures = pictures.grouped(Layout::of(&tree));
        }
        let mut steps = pictures.steps(&start, moves, &[], false)?;
        // the cube after the last move, as `solve --icons` ends
        let end = start.apply(moves).map_err(|error| error.to_string())?;
        steps.push('\n');
        steps.push_str(&end_line(&end)?);
        steps.push_str(&pictures.start_state(&end));
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
    for (at, action) in actions.iter().enumerate() {
        let drawn = icon(&state, action, &style).map_err(|error| error.to_string())?;
        // the text pictures have no number; the others carry the typed index
        let numbers = match (pictures.numbers, format) {
            (Numbers::Auto, Format::Svg) => Numbers::Off,
            (numbers, _) => numbers,
        };
        let drawn = numbered(drawn, at + 1, numbers, style.backdrop);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_turns_become_two_quarter_turns() {
        let moves: MoveSequence = "R2 U' 2Rw2".parse().unwrap();
        assert_eq!(quarter_turns(&moves).to_string(), "R R U' Rw Rw");
    }
}
