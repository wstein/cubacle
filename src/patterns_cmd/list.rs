//! `patterns` without a subcommand: the flat list and the tree.

use cubacle::patterns::{self, Pattern, PatternKind, Span};

use super::ListArgs;

fn lengths(pattern: &Pattern, size: usize) -> String {
    let of = |metric| {
        pattern
            .generator(size, Some(metric))
            .map_or("-".to_owned(), |g| g.length.to_string())
    };
    format!("{}/{}", of(Span::Outer), of(Span::Range))
}

fn sizes_of(pattern: &Pattern, size: Option<usize>) -> Vec<usize> {
    let mut sizes: Vec<usize> = pattern
        .generators()
        .iter()
        .map(|g| g.size)
        .filter(|s| size.is_none_or(|wanted| wanted == *s))
        .collect();
    sizes.sort_unstable();
    sizes.dedup();
    sizes
}

pub(super) fn print(
    library: &crate::patterns_config::Library,
    size: Option<usize>,
    args: &ListArgs,
) -> Result<(), String> {
    let ListArgs {
        tree,
        sort,
        min_patternness: min,
        kind,
    } = *args;
    let scored = sort.is_some() || min.is_some();
    let mut items: Vec<(&Pattern, u8)> = library
        .all()
        .filter(|p| !sizes_of(p, size).is_empty())
        .map(|p| (p, shown_patternness(p, size)))
        .filter(|(_, score)| min.is_none_or(|m| *score >= m))
        .collect();
    if let Some(wanted) = kind {
        items.retain(|(p, _)| shown_kind(p, size) == Some(wanted));
    }
    if sort.is_some() {
        items.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    }
    if tree {
        let items: Vec<(&Pattern, Option<u8>)> = items
            .into_iter()
            .map(|(p, score)| (p, scored.then_some(score)))
            .collect();
        print_tree(&items, 0, size);
        return Ok(());
    }
    println!(
        "id\tname\tsizes (outer/range moves){}{}",
        if scored { "\tpatternness" } else { "" },
        if kind.is_some() { "\tkind" } else { "" }
    );
    for (pattern, score) in items {
        let sizes = sizes_of(pattern, size);
        let text: Vec<String> = sizes
            .iter()
            .map(|s| format!("{s}:{}", lengths(pattern, *s)))
            .collect();
        print!("{}\t{}\t{}", pattern.id, pattern.name, text.join(" "));
        if scored {
            print!("\t{score} %");
        }
        if let Some(kind) = kind {
            print!("\t{kind}");
        }
        match &pattern.origin {
            Some(origin) => println!("\timported from {origin}"),
            None => println!(),
        }
    }
    Ok(())
}

/// The highest patternness of a pattern over the sizes shown.
fn shown_patternness(pattern: &Pattern, size: Option<usize>) -> u8 {
    sizes_of(pattern, size)
        .into_iter()
        .filter_map(|s| pattern.patternness(s))
        .max()
        .unwrap_or(0)
}

/// The kind of a pattern at the size shown with the highest patternness
/// (the smallest on a tie).
fn shown_kind(pattern: &Pattern, size: Option<usize>) -> Option<PatternKind> {
    let best = sizes_of(pattern, size)
        .into_iter()
        .filter_map(|s| Some((pattern.patternness(s)?, std::cmp::Reverse(s))))
        .max()?;
    let state = pattern.state(cubacle::puzzle::PuzzleKind::of_size(best.1 .0)?)?;
    Some(patterns::kind(&state))
}

/// The group path shown in the tree: an imported table's last level is the
/// pattern's own name, so it is the leaf and not a heading.
fn tree_path(pattern: &Pattern) -> &[patterns::Group] {
    match pattern.group.last() {
        Some(last)
            if pattern.origin.is_some()
                && !last.en.is_empty()
                && pattern.name.ends_with(&last.en) =>
        {
            &pattern.group[..pattern.group.len() - 1]
        }
        _ => &pattern.group,
    }
}

/// Patterns under one heading, then the headings below it, in first-seen order.
fn print_tree(items: &[(&Pattern, Option<u8>)], depth: usize, size: Option<usize>) {
    let pad = "  ".repeat(depth);
    for (pattern, score) in items.iter().filter(|(p, _)| tree_path(p).len() == depth) {
        let sizes: Vec<String> = sizes_of(pattern, size)
            .iter()
            .map(|s| {
                let shortest = pattern
                    .generators()
                    .iter()
                    .filter(|g| g.size == *s)
                    .map(|g| g.length)
                    .min()
                    .unwrap_or(0);
                format!("{s}:{shortest}")
            })
            .collect();
        let count = pattern
            .generators()
            .iter()
            .filter(|g| size.is_none_or(|wanted| wanted == g.size))
            .count();
        let score = score.map_or(String::new(), |s| format!("\t{s} %"));
        println!(
            "{pad}{}\t{}\t{count} {}\t{}{score}",
            pattern.catalogue,
            pattern.name,
            if count == 1 {
                "generator"
            } else {
                "generators"
            },
            sizes.join(" ")
        );
    }
    let mut seen: Vec<&str> = Vec::new();
    for (pattern, _) in items {
        let path = tree_path(pattern);
        if path.len() <= depth || seen.contains(&path[depth].en.as_str()) {
            continue;
        }
        let heading = &path[depth].en;
        seen.push(heading);
        let below: Vec<(&Pattern, Option<u8>)> = items
            .iter()
            .copied()
            .filter(|(p, _)| tree_path(p).get(depth).is_some_and(|g| g.en == *heading))
            .collect();
        println!("{pad}{heading}");
        print_tree(&below, depth + 1, size);
    }
}
