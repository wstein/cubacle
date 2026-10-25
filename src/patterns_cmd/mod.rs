//! The `patterns` subcommand: list the pattern library, or show one pattern
//! with its generators.

use clap::{Args, Subcommand};
use cubacle::engines::pattern::DEFAULT_MAX_MEMORY_MB;
use cubacle::notation::{ColourScheme, Notation};
use cubacle::patterns::PatternKind;

use crate::steps::PictureArgs;

mod icons;

#[derive(Args)]
pub struct PatternsArgs {
    /// Cube size, 3 to 7: list only the patterns that exist for it (with
    /// `show`: print only that size)
    #[arg(long, global = true)]
    size: Option<usize>,
    /// Config file (default: $CUBACLE_CONFIG, ./cubacle.toml, ~/.config/cubacle/cubacle.toml)
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
    #[command(flatten)]
    list: ListArgs,
    #[command(flatten)]
    pictures: PictureArgs,
    /// Never use the network, not even for `update` or `auto_fetch` (also CUBACLE_OFFLINE=1)
    #[arg(long, global = true, env = "CUBACLE_OFFLINE", value_parser = clap::builder::BoolishValueParser::new())]
    offline: bool,
    /// More detail (`identify`: every symmetry; which sources are not downloaded)
    #[arg(short, long, global = true)]
    verbose: bool,
    #[command(subcommand)]
    action: Option<Action>,
}

/// What the list shows; given to `patterns` or `patterns list`.
#[derive(Args, Clone, Copy)]
pub struct ListArgs {
    /// List as a tree of categories with the number of generators and the
    /// shortest length per size, instead of the flat list
    #[arg(long, global = true)]
    tree: bool,
    /// Order the list: `patternness` puts the most pattern-like first (with a
    /// patternness column)
    #[arg(long, global = true, value_enum)]
    sort: Option<SortKey>,
    /// List only patterns whose patternness (the highest over the sizes shown) is at least N
    #[arg(long, global = true, value_name = "N", value_parser = clap::value_parser!(u8).range(0..=100))]
    min_patternness: Option<u8>,
    /// List only patterns of this kind (SOLVED, FEW, MOSTLY, SYM, FACE,
    /// IRREG), judged at the size with the highest patternness; adds a kind
    /// column. Library and imported patterns alike
    #[arg(long, global = true, value_name = "NAME")]
    kind: Option<PatternKind>,
    /// List at most N patterns (default: all; with --icons 24, and a hint on
    /// standard error says how many were cut)
    #[arg(long, global = true, value_name = "N", value_parser = clap::value_parser!(u32).range(1..))]
    limit: Option<u32>,
}

impl ListArgs {
    /// The first list option given, for commands that cannot use it.
    fn first_used(&self) -> Option<&'static str> {
        [
            (self.tree, "--tree"),
            (self.sort.is_some(), "--sort"),
            (self.min_patternness.is_some(), "--min-patternness"),
            (self.kind.is_some(), "--kind"),
            (self.limit.is_some(), "--limit"),
        ]
        .into_iter()
        .find_map(|(given, name)| given.then_some(name))
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum SortKey {
    Patternness,
}

#[derive(Subcommand)]
enum Action {
    /// List the pattern library, the same as `patterns` alone
    List,
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
    /// Download the http(s) sources of the config into the cache (only what the config lists)
    #[command(visible_alias = "download")]
    Update {
        /// Download even if the server says the file is unchanged
        #[arg(long)]
        force: bool,
    },
    /// Print the config file in use and what each pattern source gave
    Sources,
}

#[derive(Args)]
pub struct IdentifyCmd {
    #[command(flatten)]
    state: IdentifyArgs,
    #[command(flatten)]
    repair: repair::RepairArgs,
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
    /// How many of the closest patterns (built-in and imported) to list. The
    /// percentage counts equal stickers and is not a distance: a state a few
    /// block moves from a pattern can agree with it on far fewer stickers
    #[arg(long, default_value_t = 3, value_name = "N")]
    closest: usize,
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
    if !matches!(args.action, None | Some(Action::List)) {
        if let Some(name) = args.list.first_used() {
            return Err(format!(
                "{name} applies to the list only: use it with `patterns` or `patterns list`"
            ));
        }
    }
    if args.pictures.icons
        && !matches!(
            args.action,
            None | Some(Action::List | Action::Show { .. } | Action::Find(_) | Action::Identify(_))
        )
    {
        return Err("--icons applies to patterns list, show, find and identify".to_owned());
    }
    let config = crate::patterns_config::load(args.config.as_deref())?;
    let mode = crate::patterns_update::Mode {
        force: false,
        offline: args.offline,
    };
    if !matches!(
        args.action,
        Some(Action::Update { .. } | Action::Sources | Action::Find(_) | Action::Cycles(_))
            | Some(Action::Import(_))
    ) {
        crate::patterns_update::auto_fetch(&config, mode);
    }
    let verbose = args.verbose;
    let library = || {
        let library = config.library();
        for warning in &library.warnings {
            eprintln!("warning: {warning}");
        }
        if !library.missing.is_empty() {
            let n = library.missing.len();
            let (noun, verb) = if n == 1 {
                ("source", "is")
            } else {
                ("sources", "are")
            };
            eprintln!("{n} {noun} {verb} not downloaded; run `cubacle patterns update`");
            if verbose {
                for url in &library.missing {
                    eprintln!("  not downloaded: {url}");
                }
            }
        }
        library
    };
    match args.action {
        None | Some(Action::List) => {
            let icons = icons::options(&args.pictures, None, Notation::default());
            list::print(&library(), args.size, &args.list, icons.as_ref())
        }
        Some(Action::Show { name }) => {
            let icons = icons::options(&args.pictures, None, Notation::default());
            show::show(&library(), &name.join(" "), args.size, icons.as_ref())
        }
        Some(Action::Find(find)) => {
            let icons = icons::options(&args.pictures, find.scheme, find.notation);
            state::find_state(&find, args.size.unwrap_or(3), icons.as_ref())
        }
        Some(Action::Identify(identify)) => {
            let state = &identify.state;
            let icons = icons::options(&args.pictures, state.scheme, Notation::default());
            let size = args.size.unwrap_or(3);
            state::identify_state(
                state,
                &identify.repair,
                verbose,
                size,
                &library(),
                icons.as_ref(),
            )
        }
        Some(Action::Cycles(cycles)) => state::cycles_state(&cycles, args.size.unwrap_or(3)),
        Some(Action::Import(import)) => {
            let dir = config.data_dir()?;
            crate::patterns_import_cmd::run(import, &dir, args.size)
        }
        Some(Action::Update { force }) => {
            crate::patterns_update::update(&config, crate::patterns_update::Mode { force, ..mode })
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

mod list;
mod repair;
mod show;
mod state;
