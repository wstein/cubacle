//! The `patterns` subcommand: list the pattern library, or show one pattern
//! with its generators.

use clap::{Args, Subcommand};
use cubacle::patterns::{self, Generator, Metric, Pattern};

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
}

pub fn run(args: PatternsArgs) -> Result<(), String> {
    match args.action {
        None => list(args.size),
        Some(Action::Show { name }) => show(&name.join(" "), args.size),
    }
}

/// The shortest generator of a size in both metrics, as `outer/range` lengths.
fn lengths(pattern: &Pattern, size: usize) -> String {
    let of = |metric| {
        pattern
            .generator(size, Some(metric))
            .map_or("-".to_owned(), |g| g.length.to_string())
    };
    format!("{}/{}", of(Metric::Outer), of(Metric::Range))
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
        for metric in [Metric::Outer, Metric::Range] {
            let Some(generator) = pattern.generator(s, Some(metric)) else {
                continue;
            };
            // A range answer that is an outer generator was printed already.
            if metric == Metric::Outer || generator.metric == Metric::Range {
                print_generator(generator);
            }
        }
    }
    Ok(())
}
