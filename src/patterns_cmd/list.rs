//! `patterns` without a subcommand: the flat list and the tree.

use cubacle::patterns::{self, Pattern, PatternKind, Span};

use cubacle::puzzle::PuzzleKind;

use super::icons::Gallery;
use super::ListArgs;
use crate::steps::SolutionIcons;

/// How many patterns `--icons` draws unless `--limit` says otherwise.
const ICON_LIMIT: usize = 24;

/// How many patterns are listed: `--limit`, else all, else (with `--icons`) 24.
fn effective_limit(limit: Option<u32>, icons: bool) -> usize {
    match (limit, icons) {
        (Some(n), _) => n as usize,
        (None, true) => ICON_LIMIT,
        (None, false) => usize::MAX,
    }
}

/// The size a pattern is drawn at: the size asked for, else the 3x3x3, else
/// the smallest size the pattern has.
fn picture_size(pattern: &Pattern, size: Option<usize>) -> Option<usize> {
    let sizes = sizes_of(pattern, size);
    let wanted = size.unwrap_or(3);
    sizes
        .contains(&wanted)
        .then_some(wanted)
        .or_else(|| sizes.first().copied())
}

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
    icons: Option<&SolutionIcons>,
) -> Result<(), String> {
    let ListArgs {
        tree,
        sort,
        min_patternness: min,
        kind,
        limit,
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
        if icons.is_some() {
            eprintln!("note: --tree is text only, so --icons and --limit draw and cut nothing");
        }
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
    let total = items.len();
    let limit = effective_limit(limit, icons.is_some());
    items.truncate(limit);
    let mut gallery = icons.map(Gallery::new);
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
        let Some(gallery) = gallery.as_mut() else {
            continue;
        };
        let state = picture_size(pattern, size)
            .and_then(PuzzleKind::of_size)
            .and_then(|kind| pattern.state(kind));
        match state {
            Some(state) => print!("{}", gallery.state(&label(&state), &state)),
            None => eprintln!("{}: no picture, the state is not defined", pattern.id),
        }
    }
    if total > limit {
        eprintln!(
            "showing the first {limit} of {total} patterns; --limit {total} shows all of them"
        );
    }
    Ok(())
}

/// The line above a picture: the size it is drawn at (the row above names the pattern).
fn label(state: &cubacle::puzzle::Facelets) -> String {
    let n = state.kind().size();
    format!("{n}x{n}x{n}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_cap_the_list_unless_a_limit_is_given() {
        assert_eq!(effective_limit(None, false), usize::MAX);
        assert_eq!(effective_limit(None, true), 24);
        assert_eq!(effective_limit(Some(5), true), 5);
        assert_eq!(effective_limit(Some(500), false), 500);
    }
}
