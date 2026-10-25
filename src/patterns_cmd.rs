//! The `patterns` subcommand: list the pattern library, or show one pattern
//! with its generators.

use std::time::Duration;

use clap::{Args, Subcommand};
use cubacle::engines::pattern::{find_generator_with, Cap, DEFAULT_MAX_MEMORY_MB};
use cubacle::notation::{parse_cube, parse_expression, ColourScheme, Notation};
use cubacle::patterns::{self, Generator, Pattern, PatternKind, Span};
use cubacle::puzzle::Move;

#[derive(Args)]
pub struct PatternsArgs {
    /// Cube size, 3 to 7: list only the patterns that exist for it (with
    /// `show`: print only that size)
    #[arg(long, global = true)]
    size: Option<usize>,
    /// Config file (default: $CUBACLE_CONFIG, ./cubacle.toml, ~/.config/cubacle/cubacle.toml)
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
    /// List as a tree of categories with the number of generators and the
    /// shortest length per size, instead of the flat list
    #[arg(long)]
    tree: bool,
    /// Order the list: `patternness` puts the most pattern-like first (with a
    /// patternness column)
    #[arg(long, value_enum)]
    sort: Option<SortKey>,
    /// List only patterns whose patternness (the highest over the sizes shown) is at least N
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u8).range(0..=100))]
    min_patternness: Option<u8>,
    /// List only patterns of this kind (SOLVED, FEW, MOSTLY, SYM, FACE, ORBIT,
    /// IRREG), judged at the size with the highest patternness; adds a kind
    /// column. Library and imported patterns alike
    #[arg(long, value_name = "NAME")]
    kind: Option<PatternKind>,
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum SortKey {
    Patternness,
}

#[derive(Subcommand)]
enum Action {
    /// Describe one pattern and print its generators
    Show {
        /// Pattern id, name, alias or catalogue id (`ct:A200.01`), with `#n` for one generator; spaces are allowed and need no quoting
        #[arg(num_args = 1.., required = true)]
        name: Vec<String>,
    },
    /// Find the shortest known generator of a cube state: the moves that build it from the solved cube
    Find(FindArgs),
    /// Read the structure of a cube state: its symmetries and, as the detector
    /// grows, the pattern it is, its pieces and the subgroups it lies in
    Identify(IdentifyCmd),
    /// Print a state as piece cycles with orientation marks and the order of
    /// the permutation (moves give the exact order; colours only on 2x2x2 and 3x3x3)
    Cycles(IdentifyArgs),
    /// Import a tab-separated pattern table (CubeTwister `PatternExportTab.txt` layout)
    Import(crate::patterns_import_cmd::ImportArgs),
    /// Print the config file in use and what each pattern source gave
    Sources,
}

#[derive(Args)]
pub struct IdentifyCmd {
    #[command(flatten)]
    state: IdentifyArgs,
    /// List every symmetry operation with the colour renaming it needs
    #[arg(short, long)]
    verbose: bool,
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
    let config = crate::patterns_config::load(args.config.as_deref())?;
    let library = || {
        let library = config.library();
        for warning in &library.warnings {
            eprintln!("warning: {warning}");
        }
        library
    };
    match args.action {
        None => list(
            &library(),
            args.size,
            args.tree,
            args.sort,
            args.min_patternness,
            args.kind,
        ),
        Some(Action::Show { name }) => show(&library(), &name.join(" "), args.size),
        Some(Action::Find(find)) => find_state(&find, args.size.unwrap_or(3)),
        Some(Action::Identify(identify)) => identify_state(
            &identify.state,
            identify.verbose,
            args.size.unwrap_or(3),
            &library(),
        ),
        Some(Action::Cycles(cycles)) => cycles_state(&cycles, args.size.unwrap_or(3)),
        Some(Action::Import(import)) => {
            let dir = config.data_dir()?;
            crate::patterns_import_cmd::run(import, &dir, args.size)
        }
        Some(Action::Sources) => sources(&config),
    }
}

/// `patterns sources`: the config and what each entry gave.
fn sources(config: &crate::patterns_config::Config) -> Result<(), String> {
    match &config.path {
        Some(path) => println!("config: {}", path.display()),
        None => println!("config: none (default: the built-in library only)"),
    }
    println!("data directory: {}", config.data_dir()?.display());
    for note in config.library().notes {
        println!("{note}");
    }
    Ok(())
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
    let (cycles, state_of_cycles) = match cube {
        Some(cube) => (patterns::cycles_of_state(&cube)?, Some(cube)),
        None => match parse_expression(&text, kind) {
            Ok(moves) => (
                patterns::cycles_of_moves(kind, &moves)?,
                cubacle::puzzle::Facelets::solved(kind).apply(&moves).ok(),
            ),
            Err(_) => return crate::parse_text(&text, args.scheme, size).map(|_| ()),
        },
    };
    print!("{cycles}");
    if let Some(state) = state_of_cycles {
        println!("patternness: {} %", patterns::patternness(&state));
    }
    Ok(())
}

/// `patterns identify`: what the structure of a state says about it.
fn identify_state(
    args: &IdentifyArgs,
    verbose: bool,
    size: usize,
    library: &crate::patterns_config::Library,
) -> Result<(), String> {
    let state = match &args.scramble {
        Some(scramble) => crate::scrambled(scramble, crate::kind_of(size)?)?,
        None => crate::parse_text(&args.cube.join(" "), args.scheme, size)?.0,
    };
    let report = patterns::detect(&state);
    if verbose {
        print!("{}", report.detailed());
    } else {
        print!("{report}");
    }
    print_kind(&patterns::measures(&state));
    for pattern in patterns::identify_imported(&library.extra, &state) {
        println!(
            "imported: {} {} (imported from {})",
            pattern.catalogue,
            pattern.name,
            pattern.origin.as_deref().unwrap_or("?")
        );
    }
    Ok(())
}

/// The kind of a state and the measures it was judged by.
fn print_kind(m: &patterns::Measures) {
    let kind = m.kind();
    println!("kind: {kind} ({})", kind.describe());
    println!(
        "measures: fixed {:.2}, defect symmetry {:.2}, moved {} of {} pieces, symmetry {:.2} (order {}), faces {:.2}, simple faces {:.2}, orbits {:.2}",
        m.fixed,
        m.mask_symmetry,
        m.moved,
        m.pieces,
        m.symmetry,
        m.symmetry_order,
        m.faces,
        m.simple_faces,
        m.orbit
    );
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
    eprintln!("patternness: {} %", patterns::patternness(&state));
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

fn list(
    library: &crate::patterns_config::Library,
    size: Option<usize>,
    tree: bool,
    sort: Option<SortKey>,
    min: Option<u8>,
    kind: Option<PatternKind>,
) -> Result<(), String> {
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

fn show(
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
