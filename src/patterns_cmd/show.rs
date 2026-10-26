//! `patterns show`: one pattern with its generators.

use cubacle::patterns::{Generator, Pattern, Span};

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

/// `name` or `name#n`.
fn split_number(text: &str) -> (&str, Option<usize>) {
    match text.rsplit_once('#') {
        Some((name, n)) if n.parse::<usize>().is_ok() => (name.trim(), n.parse().ok()),
        _ => (text, None),
    }
}

/// Imported patterns and a `#n` request list generators by number.
fn show_numbered(
    pattern: &Pattern,
    size: Option<usize>,
    number: Option<usize>,
) -> Result<(), String> {
    println!("{} ({})", pattern.name, pattern.catalogue);
    if !pattern.aliases.is_empty() {
        println!("also: {}", pattern.aliases.join(", "));
    }
    println!("{}", pattern.description);
    if !pattern.group.is_empty() {
        let names: Vec<&str> = pattern.group.iter().map(|g| g.en.as_str()).collect();
        println!("group: {}", names.join(" > "));
    }
    if !pattern.provenance.is_empty() {
        println!("by: {}", pattern.provenance);
    }
    if let Some(origin) = &pattern.origin {
        println!("imported from {origin}");
    }
    let mut shown = 0;
    let mut scored: Vec<usize> = Vec::new();
    for g in pattern.generators() {
        if size.is_some_and(|s| s != g.size) || number.is_some_and(|n| n != g.number) {
            continue;
        }
        if !scored.contains(&g.size) {
            scored.push(g.size);
            if let Some(score) = pattern.patternness(g.size) {
                println!("{0}x{0}x{0} patternness: {score} %", g.size);
            }
        }
        shown += 1;
        println!("{0}x{0}x{0} #{1}: {2} moves", g.size, g.number, g.length);
        println!("    {}", g.moves);
    }
    if shown == 0 {
        return Err(format!("{} has no such generator", pattern.catalogue));
    }
    Ok(())
}

pub(super) fn show(
    library: &crate::patterns_config::Library,
    text: &str,
    size: Option<usize>,
) -> Result<(), String> {
    let (name, number) = split_number(text);
    let pattern = library
        .find(name)
        .ok_or_else(|| format!("no pattern named {name:?}"))?;
    if number.is_some() || pattern.origin.is_some() {
        return show_numbered(pattern, size, number);
    }
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
        if let Some(score) = pattern.patternness(s) {
            println!("  patternness: {score} %");
        }
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
