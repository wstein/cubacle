//! The `moves` subcommand: the plain move sequence an expression stands for.

use std::io::Read;

use clap::Args;
use cubacle::notation::{parse_expression, Notation};
use cubacle::puzzle::{MoveSequence, PuzzleKind};

#[derive(Args)]
pub struct MovesArgs {
    /// A move expression such as "(R2 F2 L2)*2" or "[R: U]"; spaces are allowed
    /// and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., required = true)]
    expression: Vec<String>,
    /// Cube size, 2 to 7
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// How to write the moves: wca (Rw, 3Rw) or sign (r, 3r)
    #[arg(long, default_value = "wca")]
    notation: Notation,
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

/// The moves of an expression on a cube of `size`, checked for that size.
fn expand(text: &str, size: usize) -> Result<MoveSequence, String> {
    let kind = PuzzleKind::of_size(size)
        .ok_or_else(|| format!("size must be between 2 and 7, not {size}"))?;
    parse_expression(text, kind).map_err(|error| error.to_string())
}

pub fn run(args: MovesArgs) -> Result<(), String> {
    let text = read_text(&args.expression)?;
    let moves = expand(&text, args.size)?;
    println!("{}", args.notation.write(&moves));
    Ok(())
}
