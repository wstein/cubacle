//! The `canon` subcommand: the canonical Orbit64 token of a cube state.

use clap::{Args, ValueEnum};
use cubacle::canonical::canonical;

use crate::{read_unchecked, InputArgs};

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

/// The name of the slot convention the token is written in.
fn schema(size: usize) -> String {
    format!("orbit64-{size}x{size}-draft@1")
}

pub fn run(args: CanonArgs) -> Result<(), String> {
    let (cube, _) = read_unchecked(&args.input)?;
    let token = canonical(&cube).map_err(|error| error.to_string())?;
    let size = cube.kind().size();
    match (args.format, args.verbose) {
        (Format::Json, _) => println!(
            "{{\"canonical\":{{\"token\":\"{token}\",\"schema\":\"{}\"}},\"size\":{size}}}",
            schema(size)
        ),
        (Format::Text, true) => {
            println!("canonical: {token}");
            println!("schema: {}", schema(size));
            println!("size: {size}x{size}x{size}");
        }
        (Format::Text, false) => println!("{token}"),
    }
    Ok(())
}
