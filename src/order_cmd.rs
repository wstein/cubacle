//! The `order` subcommand: how many repetitions of a move sequence return
//! the cube, from the cycles of its stickers.

use clap::Args;
use cubacle::puzzle::{sticker_cycles, OrbitCycles};

use crate::moves_cmd::{expand, kind_of, read_text};

#[derive(Args)]
pub struct OrderArgs {
    /// A move expression such as "R U" or "[R: U]"; spaces are allowed and
    /// need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., required = true)]
    expression: Vec<String>,
    /// Cube size, 2 to 7
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// Also print the cycle lengths of the stickers of each orbit (corners,
    /// edges, wings, centers), whose least common multiple is the order
    #[arg(short, long)]
    verbose: bool,
}

/// The cycle lengths with their counts, e.g. `3x2 5` for two 3-cycles and a 5-cycle.
fn lengths_text(orbit: &OrbitCycles) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut rest = orbit.lengths.as_slice();
    while let Some(&len) = rest.first() {
        let count = rest.iter().take_while(|&&l| l == len).count();
        parts.push(if count == 1 {
            len.to_string()
        } else {
            format!("{len}x{count}")
        });
        rest = &rest[count..];
    }
    if parts.is_empty() {
        "in place".to_owned()
    } else {
        parts.join(" ")
    }
}

pub fn run(args: OrderArgs) -> Result<(), String> {
    let kind = kind_of(args.size)?;
    let text = read_text(&args.expression)?;
    let moves = expand(&text, kind)?;
    let cycles = sticker_cycles(kind, &moves).map_err(|error| error.to_string())?;
    println!("order {}", cycles.order());
    if args.verbose {
        for orbit in &cycles.orbits {
            println!(
                "{}: {} (order {})",
                orbit.orbit.name(),
                lengths_text(orbit),
                orbit.order()
            );
        }
    }
    Ok(())
}
