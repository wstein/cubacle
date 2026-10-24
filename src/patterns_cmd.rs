//! The `patterns` subcommand: list the pattern library, or show one pattern
//! with its generators.

use std::time::Duration;

use clap::{Args, Subcommand};
use cubacle::engines::pattern::{find_generator_with, Cap, DEFAULT_MAX_MEMORY_MB};
use cubacle::notation::{parse_cube, parse_expression, ColourScheme, Notation};
use cubacle::patterns::{self, Generator, Pattern, Span};
use cubacle::puzzle::Move;

#[derive(Args)]
pub struct PatternsArgs {
    /// Cube size, 3 to 7: list only the patterns that exist for it (with
    /// `show`: print only that size)
    #[arg(long, global = true)]
    size: Option<usize>,
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// Describe one pattern and print its generators
    Show {
        /// Pattern id, name or alias; spaces are allowed and need no quoting
        #[arg(num_args = 1.., required = true)]
        name: Vec<String>,
    },
    /// Find the shortest known generator of a cube state: the moves that build it from the solved cube
    Find(FindArgs),
    /// Read the structure of a cube state: its symmetries and, as the detector
    /// grows, the pattern it is, its pieces and the subgroups it lies in
    Identify(IdentifyArgs),
    /// Print a state as piece cycles with orientation marks and the order of
    /// the permutation (moves give the exact order; colours only on 2x2x2 and 3x3x3)
    Cycles(IdentifyArgs),
}

#[derive(Args)]
pub struct IdentifyArgs {
    /// An Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colours or a move
    /// expression for a cube of --size (3 unless given); spaces are allowed and
    /// need no quoting
    #[arg(num_args = 1.., required_unless_present = "scramble", conflicts_with = "scramble")]
    cube: Vec<String>,
    /// Use the cube this scramble makes (on a cube of --size)
    #[arg(short, long)]
    scramble: Option<String>,
    /// Colour scheme of colour input: boy, jp or six letters in URFDLB order
    #[arg(long)]
    scheme: Option<ColourScheme>,
}

#[derive(Args)]
pub struct FindArgs {
    /// An Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colours or a move
    /// expression for a cube of --size (3 unless given); spaces are allowed and
    /// need no quoting
    #[arg(num_args = 1.., required_unless_present = "scramble", conflicts_with = "scramble")]
    cube: Vec<String>,
    /// Use the cube this scramble makes (on a cube of --size)
    #[arg(short, long)]
    scramble: Option<String>,
    /// Colour scheme of colour input: boy, jp or six letters in URFDLB order
    #[arg(long)]
    scheme: Option<ColourScheme>,
    /// Build the state with these moves only: a list, a named set or <R,U>
    /// generators (see `solve --moves`)
    #[arg(long, value_name = "SET")]
    moves: Option<String>,
    /// Hard memory cap of the searches in megabytes
    #[arg(long, value_name = "MB", default_value_t = DEFAULT_MAX_MEMORY_MB)]
    max_memory: usize,
    /// Hard time limit in milliseconds (default: none, the cap ends the search)
    #[arg(short, long)]
    budget_ms: Option<u64>,
    /// How to write the moves: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
}

pub fn run(args: PatternsArgs) -> Result<(), String> {
    match args.action {
        None => list(args.size),
        Some(Action::Show { name }) => show(&name.join(" "), args.size),
        Some(Action::Find(find)) => find_state(&find, args.size.unwrap_or(3)),
        Some(Action::Identify(identify)) => identify_state(&identify, args.size.unwrap_or(3)),
        Some(Action::Cycles(cycles)) => cycles_state(&cycles, args.size.unwrap_or(3)),
    }
}

/// `patterns cycles`: a move sequence is followed piece by piece; a cube given
/// as text is read from its colours.
fn cycles_state(args: &IdentifyArgs, size: usize) -> Result<(), String> {
    let kind = crate::kind_of(size)?;
    let text = match &args.scramble {
        Some(scramble) => scramble.clone(),
        None => args.cube.join(" "),
    };
    let cube = match &args.scramble {
        Some(_) => None,
        None => parse_cube(&text, args.scheme).ok(),
    };
    let cycles = match cube {
        Some(cube) => patterns::cycles_of_state(&cube)?,
        None => match parse_expression(&text, kind) {
            Ok(moves) => patterns::cycles_of_moves(kind, &moves)?,
            Err(_) => return crate::parse_text(&text, args.scheme, size).map(|_| ()),
        },
    };
    print!("{cycles}");
    Ok(())
}

/// `patterns identify`: what the structure of a state says about it.
fn identify_state(args: &IdentifyArgs, size: usize) -> Result<(), String> {
    let state = match &args.scramble {
        Some(scramble) => crate::scrambled(scramble, crate::kind_of(size)?)?,
        None => crate::parse_text(&args.cube.join(" "), args.scheme, size)?.0,
    };
    print!("{}", patterns::detect(&state));
    Ok(())
}

/// `patterns find`: the moves that build a state from the solved cube.
fn find_state(args: &FindArgs, size: usize) -> Result<(), String> {
    let state = match &args.scramble {
        Some(scramble) => crate::scrambled(scramble, crate::kind_of(size)?)?,
        None => crate::parse_text(&args.cube.join(" "), args.scheme, size)?.0,
    };
    let cap = Cap::megabytes(args.max_memory);
    let time = args.budget_ms.map(Duration::from_millis);
    let alphabet = match &args.moves {
        Some(text) => crate::restricted_cmd::read_set(text, state.kind())?,
        None => None,
    };
    let allow = alphabet.as_ref().map(|a| move |mv: &Move| a.contains(mv));
    let generator = find_generator_with(
        &state,
        cap,
        time,
        allow.as_ref().map(|f| f as &dyn Fn(&Move) -> bool),
    )
    .map_err(|error| error.to_string())?;
    eprintln!("moves {}", generator.moves.len());
    eprintln!(
        "{}",
        crate::metrics_out::line(state.kind(), &generator.moves)
    );
    match (&generator.note, generator.proven) {
        (_, true) => {
            eprintln!("shortest: no sequence of fewer moves exists (searched exhaustively)")
        }
        (Some(note), false) => eprintln!("best known, a shorter one may exist ({note})"),
        (None, false) => eprintln!("best known, a shorter one may exist"),
    }
    println!("{}", args.notation.write(state.kind(), &generator.moves));
    Ok(())
}

/// The shortest generator of a size in both metrics, as `outer/range` lengths.
fn lengths(pattern: &Pattern, size: usize) -> String {
    let of = |metric| {
        pattern
            .generator(size, Some(metric))
            .map_or("-".to_owned(), |g| g.length.to_string())
    };
    format!("{}/{}", of(Span::Outer), of(Span::Range))
}

fn list(size: Option<usize>) -> Result<(), String> {
    println!("id\tname\tsizes (outer/range moves)");
    for pattern in patterns::all() {
        let mut sizes: Vec<usize> = pattern
            .generators()
            .iter()
            .map(|g| g.size)
            .filter(|s| size.is_none_or(|wanted| wanted == *s))
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        if sizes.is_empty() {
            continue;
        }
        let text: Vec<String> = sizes
            .iter()
            .map(|s| format!("{s}:{}", lengths(pattern, *s)))
            .collect();
        println!("{}\t{}\t{}", pattern.id, pattern.name, text.join(" "));
    }
    Ok(())
}

fn print_generator(generator: &Generator) {
    println!(
        "  {} metric, {} moves, {}:",
        generator.metric.name(),
        generator.length,
        generator.status.name()
    );
    println!("    {}", generator.moves);
    if let Some(kind) = cubacle::puzzle::PuzzleKind::of_size(generator.size) {
        if let Some(line) = crate::metrics_out::words_line(&generator.moves, kind) {
            eprintln!("{line}");
        }
    }
}

fn show(name: &str, size: Option<usize>) -> Result<(), String> {
    let pattern = patterns::find(name).ok_or_else(|| format!("no pattern named {name:?}"))?;
    println!("{} ({})", pattern.name, pattern.id);
    if !pattern.aliases.is_empty() {
        println!("also: {}", pattern.aliases.join(", "));
    }
    println!("{}", pattern.description);
    let mut sizes: Vec<usize> = pattern.generators().iter().map(|g| g.size).collect();
    sizes.sort_unstable();
    sizes.dedup();
    if let Some(wanted) = size {
        if !sizes.contains(&wanted) {
            return Err(format!(
                "{} has no generator for the {wanted}x{wanted}x{wanted} (sizes: {})",
                pattern.id,
                sizes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        sizes.retain(|s| *s == wanted);
    }
    for s in sizes {
        println!("{s}x{s}x{s}");
        for metric in [Span::Outer, Span::Range] {
            let Some(generator) = pattern.generator(s, Some(metric)) else {
                continue;
            };
            // A range answer that is an outer generator was printed already.
            if metric == Span::Outer || generator.metric == Span::Range {
                print_generator(generator);
            }
        }
    }
    Ok(())
}
