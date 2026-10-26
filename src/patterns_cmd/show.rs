//! `patterns show`: one pattern with its generators.

use cubacle::notation::orbit64;
use cubacle::patterns::{self, Generator, Pattern, Span};
use cubacle::puzzle::PuzzleKind;

use super::icons::Gallery;
use crate::steps::SolutionIcons;

/// The picture of the pattern at a size, when pictures are on.
fn draw_pattern(gallery: &mut Option<Gallery<'_>>, pattern: &Pattern, size: usize) {
    let (Some(gallery), Some(kind)) = (gallery.as_mut(), PuzzleKind::of_size(size)) else {
        return;
    };
    match pattern.state(kind) {
        Some(state) => print!(
            "{}",
            gallery.state(&format!("{size}x{size}x{size} pattern"), &state)
        ),
        None => eprintln!("{}: no picture at {size}x{size}x{size}", pattern.id),
    }
}

/// The move icons of a generator, when pictures are on.
fn draw_moves(gallery: &mut Option<Gallery<'_>>, generator: &Generator) -> Result<(), String> {
    if let Some(gallery) = gallery.as_mut() {
        print!("{}", gallery.moves(generator)?);
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
    icons: Option<&SolutionIcons>,
) -> Result<(), String> {
    let mut gallery = icons.map(Gallery::new);
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
            draw_pattern(&mut gallery, pattern, g.size);
        }
        shown += 1;
        println!("{0}x{0}x{0} #{1}: {2} moves", g.size, g.number, g.length);
        println!("    {}", g.moves);
        draw_moves(&mut gallery, g)?;
    }
    if shown == 0 {
        return Err(format!("{} has no such generator", pattern.catalogue));
    }
    Ok(())
}

/// The pattern a name, alias or id stands for; or, for an Orbit64 token, the library
/// pattern the cube is (the first match; the others are named on standard error).
fn find<'a>(
    library: &'a crate::patterns_config::Library,
    name: &str,
) -> Result<&'a Pattern, String> {
    if let Some(pattern) = library.find(name) {
        return Ok(pattern);
    }
    let compact: String = name.split_whitespace().collect();
    let Ok(state) = orbit64::decode(&compact) else {
        return Err(format!("no pattern named {name:?}"));
    };
    let size = state.kind().size();
    let report = patterns::detect_with(&state, &library.extra, 0);
    let mut found = report
        .matches
        .iter()
        .filter_map(|m| library.find(&m.id).map(|p| (p, m)));
    let Some((pattern, first)) = found.next() else {
        return Err(format!(
            "the token is a {size}x{size}x{size} cube that is no library pattern (`cubacle patterns identify` lists the closest)"
        ));
    };
    eprintln!(
        "the token is {} ({}), {}",
        pattern.name, pattern.id, first.relation
    );
    for (other, m) in found {
        eprintln!("also {} ({}), {}", other.name, other.id, m.relation);
    }
    Ok(pattern)
}

pub(super) fn show(
    library: &crate::patterns_config::Library,
    text: &str,
    size: Option<usize>,
    icons: Option<&SolutionIcons>,
) -> Result<(), String> {
    let (name, number) = split_number(text);
    let pattern = find(library, name)?;
    if number.is_some() || pattern.origin.is_some() {
        return show_numbered(pattern, size, number, icons);
    }
    let mut gallery = icons.map(Gallery::new);
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
        draw_pattern(&mut gallery, pattern, s);
        for metric in [Span::Outer, Span::Range] {
            let Some(generator) = pattern.generator(s, Some(metric)) else {
                continue;
            };
            // A range answer that is an outer generator was printed already.
            if metric == Span::Outer || generator.metric == Span::Range {
                print_generator(generator);
                draw_moves(&mut gallery, generator)?;
            }
        }
    }
    Ok(())
}
