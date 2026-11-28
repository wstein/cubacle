//! The `canon` subcommand: the canonical Orbit64 token of a cube state.

use clap::{Args, ValueEnum};
use cubacle::canonical::{canonical, canonical_text, is_class00_identifier, schema_name};

use crate::cli::InputArgs;
use crate::input::read_unchecked;

/// How `canon` writes its answer.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    /// The token alone, or with --verbose labeled lines
    Text,
    /// One JSON object with the token, the schema and the size
    Json,
}

#[derive(Args)]
pub struct CanonArgs {
    #[command(flatten)]
    input: InputArgs,
    /// Also print the schema name and the size (labeled lines)
    #[arg(short, long)]
    verbose: bool,
    /// How to write the answer
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

pub fn run(args: CanonArgs) -> Result<(), String> {
    // A 43-character class-`00` token is no longer a state, but it is still
    // the 5×5×5 canonical identifier, so `canon` reads the identifier it writes.
    let (token, size) = if let Some(text) = class00_identifier(&args.input) {
        let token = canonical_text(&text).map_err(|error| error.to_string())?;
        let size = token.kind().size();
        (token.to_string(), size)
    } else {
        let (cube, _) = read_unchecked(&args.input)?;
        let token = canonical(&cube).map_err(|error| error.to_string())?;
        let size = cube.kind().size();
        (token.to_string(), size)
    };
    match (args.format, args.verbose) {
        (Format::Json, _) => println!(
            "{{\"canonical\":{{\"token\":\"{token}\",\"schema\":\"{}\"}},\"size\":{size}}}",
            schema_name(size)
        ),
        (Format::Text, true) => {
            println!("canonical: {token}");
            println!("schema: {}", schema_name(size));
            println!("size: {size}x{size}x{size}");
        }
        (Format::Text, false) => println!("{token}"),
    }
    Ok(())
}

/// The one class-`00` form `canon` still reads: a 43-character 5×5×5 identifier
/// that [`is_class00_identifier`] accepts. Anything else, including a compact
/// move list of that length, is read by the ordinary path.
fn class00_identifier(input: &InputArgs) -> Option<String> {
    if input.scramble.is_some() || input.cube.len() != 1 {
        return None;
    }
    let text = &input.cube[0];
    is_class00_identifier(text).then(|| text.split_whitespace().collect())
}
