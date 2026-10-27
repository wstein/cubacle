mod answer;
mod canon_cmd;
mod enumerate_cmd;
mod fetch;
mod icons;
mod improve;
mod inverse;
mod metrics_out;
mod moves_cmd;
mod optimal_cmd;
mod order_cmd;
mod partial_cmd;
mod pattern_aware_cmd;
mod patterns_cmd;
mod patterns_config;
mod patterns_import_cmd;
mod patterns_update;
mod phase_dots;
mod photo_cmd;
mod png_out;
mod ram;
mod recover_cmd;
mod recover_icons;
mod restricted_cmd;
mod rows;
mod solved_start;
mod stage_phases;
mod state_check;
mod steps;
mod template_cmd;
mod terminal;
mod views;
mod within_cmd;

use std::io::Read;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cubacle::centers;
use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::pattern::PatternShortcut;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::generate;
use cubacle::notation::orbit64;
use cubacle::notation::{
    color_scheme_of, named_subset, named_target, parse_cube, parse_expression, parse_partial,
    ColorScheme, Notation,
};
use cubacle::puzzle::{
    oriented_by_centers, relative_state, Facelets, Failure, PuzzleKind, SolveOptions, Solver,
    Target,
};
use cubacle::ranges::Optimize;

/// Rubik's cube solver and generator.
#[derive(Parser)]
#[command(name = "cubacle", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Solve a cube given as facelets or as a scramble, or with --now explain a mistake and the way back
    Solve(Box<SolveArgs>),
    /// Print the plain move sequence an expression stands for
    Moves(moves_cmd::MovesArgs),
    /// Print how many repetitions of a move sequence return the cube (its order)
    Order(order_cmd::OrderArgs),
    /// Print the canonical Orbit64 token of a cube: one identifier for the cube and its turned, mirrored and recolored images
    Canon(canon_cmd::CanonArgs),
    /// Write synthetic photos of the six faces of a cube as PNG files (for testing color recognition)
    Photo(photo_cmd::PhotoArgs),
    /// List the pretty patterns (checkerboard, cube in a cube, ...) or show one with its generator
    Patterns(patterns_cmd::PatternsArgs),
    /// Work with .cubacle template files: check that a schema and guard parse
    Template(template_cmd::TemplateArgs),
    /// Print a random scramble
    Scramble {
        /// Number of moves
        #[arg(short = 'n', long, default_value_t = 25)]
        moves: usize,
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
        /// How to write the moves: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
        #[arg(long, default_value = "wca")]
        notation: Notation,
    },
    /// Print a random cube state as facelets
    Random {
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
    },
    /// Translate any supported notation into URFDLB facelets, colors or an Orbit64 token
    Convert(ConvertArgs),
    /// Print the facelets reached by applying a scramble to a solved cube
    Facelets {
        /// Moves in WCA, SiGN or SSE notation, e.g. "R U R' U'"
        scramble: String,
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
    },
}

/// How a cube is written on the command line.
#[derive(Args)]
struct InputArgs {
    /// An Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colors, or (when the
    /// text is no cube) a move expression such as "(R U)*3" for a cube of
    /// --size; spaces are allowed and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., conflicts_with = "scramble")]
    cube: Vec<String>,
    /// Solve the cube produced by this scramble instead
    #[arg(short, long)]
    scramble: Option<String>,
    /// Cube size for --scramble or a move expression (a cube given as text has
    /// its size already)
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// Color scheme: boy, jp or six letters in URFDLB order (default for
    /// reading colors: from the centers, boy without centers; for `convert
    /// --to color`: the scheme the colors were read in, else boy)
    #[arg(long)]
    scheme: Option<ColorScheme>,
}

/// What `convert` writes.
#[derive(Clone, Copy, ValueEnum)]
enum Output {
    /// Canonical URFDLB facelets
    Facelets,
    /// An Orbit64 state token
    Token,
    /// W/R/G/Y/O/B sticker letters in facelet order, written with --scheme
    /// (default: the scheme of color input, else boy)
    #[value(name = "color")]
    Color,
}

#[derive(Args)]
struct ConvertArgs {
    #[command(flatten)]
    input: InputArgs,
    /// What to print
    #[arg(long, value_enum, default_value_t = Output::Facelets)]
    to: Output,
    /// When the input is a move expression, also print all seven move counts
    /// (htm qtm stm etm obtm btm atm) on stderr
    #[arg(long)]
    metrics: bool,
}

#[derive(Args)]
struct SolveArgs {
    #[command(flatten)]
    input: InputArgs,
    /// The cube as it is now, after following the solution of the cube above
    /// and going wrong: says where the mistake was and the way back (an
    /// Orbit64 token, URFDLB facelets or colors; quote it when it has spaces).
    /// With --to the way back leads to the target, not to the solved cube
    #[arg(long, value_name = "CUBE", conflicts_with_all = [
        "max_moves", "budget_ms", "probes", "improve", "parallel", "fast", "optimize",
        "slices", "moves", "all", "count", "raw", "optimal", "nodes", "max_memory", "group", "every",
        "pattern_aware",
    ])]
    now: Option<String>,
    #[command(flatten)]
    way: recover_cmd::RecoverOptions,
    /// Longest solution to accept (default: 21; none for the 4x4x4 and 5x5x5; a limit applies to the whole solution)
    #[arg(short = 'm', long)]
    max_moves: Option<u8>,
    /// Hard time limit in milliseconds
    #[arg(short, long)]
    budget_ms: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: make at least this many probes in the 3x3x3 search (4x4x4 and 5x5x5: default 500; deterministic, the same answer on every machine)
    #[arg(long, value_name = "N", conflicts_with = "improve")]
    probes: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: spend the time budget (-b, default 1000 ms) on a shorter solution, then print the best one
    #[arg(long)]
    improve: bool,
    /// 3x3x3 only: with --improve, search the six orientations concurrently (needs -b)
    #[arg(long)]
    parallel: bool,
    /// 3x3x3: return the first solution found, as before the counted default (plain solve makes 3000 probes for a shorter one)
    #[arg(long, conflicts_with_all = ["improve", "parallel", "probes"])]
    fast: bool,
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    verbose: bool,
    /// How to write the solution: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
    /// Post-solve compression (no search): `ranges` writes adjacent turns of one axis as
    /// layer ranges (2-3r in sign, 2-3Rw in wca) and reports both move counts; with
    /// --icons each range is one picture with one arrow
    #[arg(long, value_name = "WHAT")]
    optimize: Option<Optimize>,
    /// Allow single inner layers (2R) in 4x4x4 and 5x5x5 answers where they make them shorter
    #[arg(long)]
    slices: bool,
    /// Use only these moves: a list in any notation ("R U R2 U'", taken
    /// literally), generators ("<R,U>": every turn of R and U), or a name: htm
    /// (no restriction, the default), qtm, half-turns, quarter-slices. On the
    /// 3x3x3 the answer is shortest in the set, except qtm (the default
    /// answer in quarter turns, not proven shortest); a cube the set cannot
    /// solve is reported at once
    #[arg(long, value_name = "SET", conflicts_with_all = ["improve", "parallel", "probes", "slices"])]
    moves: Option<String>,
    /// 3x3x3 only: list every solution of the shortest length (with --moves, within the set), one order of commuting moves
    #[arg(long, conflicts_with_all = ["improve", "parallel", "probes", "slices", "optimize", "icons", "count"])]
    all: bool,
    /// 3x3x3 only: list this many solutions, shortest first, then stop
    #[arg(long, value_name = "N", conflicts_with_all = ["improve", "parallel", "probes", "slices", "optimize", "icons"])]
    count: Option<usize>,
    /// With --all or --count: also list the solutions up to this many moves,
    /// grouped by length. With --now: moves the search for a way back onto the
    /// path may use, 0 to 5 (default 3)
    #[arg(long, value_name = "D")]
    depth: Option<u8>,
    /// With --all or --count: list every order of commuting moves
    #[arg(long)]
    raw: bool,
    /// 3x3x3 only: find a provably shortest solution (half-turn metric). The default
    /// engine's answer comes first and stays the fallback; a large pruning table (cached in
    /// the cache directory) drives the search. Limited by -b (default 60000 ms) and --nodes;
    /// when the limit ends the search, the default answer is printed and stderr says "not
    /// proven shortest"
    #[arg(long, conflicts_with_all = ["fast", "probes", "improve", "parallel", "moves", "to", "all", "count"])]
    optimal: bool,
    /// With --optimal: stop after this many search nodes (the same on every machine)
    #[arg(long, value_name = "N", requires = "optimal")]
    nodes: Option<u64>,
    /// Hard memory cap in megabytes for the tables of --moves (default 1500) and
    /// --optimal (default: half of the available memory of this machine, at most
    /// 16000)
    #[arg(long, value_name = "MB")]
    max_memory: Option<usize>,
    /// For a cube that is only slightly scrambled: find an optimal route of at
    /// most K moves (1 to 7; write --within=5, plain --within means 6), searched
    /// from both ends, to the solved cube or to any --to state, in the alphabet
    /// the normal solver uses (--slices adds inner layers, --blocks every block
    /// of up to three layers). Limited by --max-time (default 60000 ms) and
    /// --max-memory (default 1000). Exit code 2: no route within K moves; 3: a
    /// limit ended the search
    #[arg(long, value_name = "K", num_args = 0..=1, require_equals = true, default_missing_value = "6", value_parser = clap::value_parser!(u8).range(1..=7))]
    within: Option<u8>,
    /// With --within: also allow the blocks of two and three layers from either
    /// side and the layers 2-3 (more routes, a larger search)
    #[arg(long)]
    blocks: bool,
    /// With --within: when no route of at most K moves exists, solve with the
    /// normal solver (the same options as without --within) and print its answer
    #[arg(long)]
    then_solve: bool,
    /// Solve to this state instead of to the solved cube: an Orbit64 token,
    /// URFDLB facelets, W/R/G/Y/O/B colors or a move expression applied to the
    /// solved cube of the start's size (one argument: quote it), or one of the
    /// names superflip, checkerboard, cube-in-cube, or the subset cross (3x3x3:
    /// the down cross; cross-u, cross-r, cross-f, cross-l, cross-b pick the
    /// face, the rest is free). An X (or ?) in facelets or
    /// colors marks a sticker that may show any color; the moves printed
    /// reach any such state. The moves printed take the start to the target.
    #[arg(long, value_name = "TARGET")]
    to: Option<String>,
    /// Mark hand units in the answer with " · ": phases (the solver phases; a
    /// solution of one phase, the 2x2x2, has no dot), axis (turns about one
    /// axis on disjoint layers, as in ATM) or both, "phases,axis" (a unit
    /// never crosses a phase border); none: a plain list
    #[arg(long, value_enum, value_delimiter = ',')]
    group: Vec<phase_dots::SolveGroup>,
    /// Cut the answer after every N moves, whatever the solver phases are: a
    /// " · " between the parts, or a heading above the pictures with --icons
    /// (for example 10 or 20). Replaces the solver phases
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u16).range(1..))]
    every: Option<u16>,
    /// Print the solution inverted: the moves that build the start cube from the
    /// solved cube (from the --to target when given). Phases, --every, --icons
    /// and --optimize ranges follow the inverted order; the counts stay those
    /// of the solution
    #[arg(long, conflicts_with_all = ["now", "all", "count"])]
    inverse: bool,
    /// Also look the cube up in the pattern library (built-in and the files of
    /// the config) and answer with the library route when it is strictly shorter
    /// than the solver's answer; the solver still runs, so the answer is never
    /// longer. 2x2x2: no effect. The default answer does not change without it
    #[arg(long, conflicts_with_all = ["moves", "optimal", "all", "count"])]
    pattern_aware: bool,
    /// With --pattern-aware: config file of the pattern sources (default:
    /// $CUBACLE_CONFIG, ./cubacle.toml, ~/.config/cubacle/config.toml)
    #[arg(long, value_name = "FILE", requires = "pattern_aware")]
    config: Option<std::path::PathBuf>,
    #[command(flatten)]
    pictures: steps::PictureArgs,
}

/// Exit status of a failed command (success is 0).
const EXIT_ERROR: u8 = 1;
/// Exit status of a solve that reached its time limit.
const EXIT_LIMIT: u8 = 3;

/// Set by a command whose error is a limit, not a fault of the input.
static LIMIT_REACHED: AtomicBool = AtomicBool::new(false);

/// The longest a solve of the 4x4x4 to 7x7x7 may run without `--max-time`.
/// Measured on a loaded machine the slowest of three random 7x7x7 solves took
/// 18 s (the 4x4x4 and 5x5x5 under 5 s, the 6x6x6 under 7 s), so this is
/// about seven times the worst.
const DEFAULT_MAX_TIME_MS: u64 = 120_000;

/// The text of a face center error, as `solve` prints it after `error: `.
pub(crate) const CENTERS_OFF_FACE: &str = "the face centers are not on their own faces (a middle-slice turn moved them); this solver needs the centers of each face in place";

fn main() -> ExitCode {
    let matches = <Cli as clap::CommandFactory>::command().get_matches();
    let cli = <Cli as clap::FromArgMatches>::from_arg_matches(&matches)
        .unwrap_or_else(|error| error.exit());
    solved_start::note_size_given(&matches);
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(if LIMIT_REACHED.load(Ordering::Relaxed) {
                EXIT_LIMIT
            } else {
                EXIT_ERROR
            })
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Solve(args) => solve(*args),
        Command::Moves(args) => moves_cmd::run(args),
        Command::Order(args) => order_cmd::run(args),
        Command::Canon(args) => canon_cmd::run(args),
        Command::Photo(args) => photo_cmd::run(args),
        Command::Patterns(args) => patterns_cmd::run(args),
        Command::Template(args) => template_cmd::run(args),
        Command::Convert(args) => {
            let (cube, detected) = read_input(&args.input)?;
            if args.metrics {
                print_metrics(&args.input)?;
            }
            match args.to {
                Output::Facelets => println!("{cube}"),
                Output::Token => println!("{}", orbit64::encode(&cube).map_err(|e| e.to_string())?),
                Output::Color => {
                    let scheme = args
                        .input
                        .scheme
                        .or(detected)
                        .unwrap_or(ColorScheme::WESTERN_BOY);
                    println!("{}", scheme.paint(&cube));
                }
            }
            Ok(())
        }
        Command::Scramble {
            moves,
            size,
            notation,
        } => {
            let kind = kind_of(size)?;
            println!("{}", notation.write(kind, &generate::scramble(kind, moves)));
            Ok(())
        }
        Command::Random { size } => {
            let kind = kind_of(size)?;
            let cube = generate::random_state(kind)
                .ok_or_else(|| format!("no random state generator for the {size}x{size}x{size}"))?;
            println!("{cube}");
            Ok(())
        }
        Command::Facelets { scramble, size } => {
            println!("{}", scrambled(&scramble, kind_of(size)?)?);
            Ok(())
        }
    }
}

/// `convert --metrics`: all seven counts of the moves the input stands for.
fn print_metrics(input: &InputArgs) -> Result<(), String> {
    let text = match (&input.scramble, input.cube.as_slice()) {
        (Some(scramble), _) => scramble.clone(),
        (None, [single]) if single == "-" => {
            return Err("--metrics cannot read the moves from stdin".into())
        }
        (None, words) => words.join(" "),
    };
    if parse_cube(&text, input.scheme).is_ok() {
        return Err("--metrics needs a move expression, not a cube".into());
    }
    let kind = kind_of(input.size)?;
    let moves = parse_expression(&text, kind).map_err(|error| error.to_string())?;
    eprintln!("{}", metrics_out::text_line(&text, kind, &moves, true));
    Ok(())
}

fn kind_of(size: usize) -> Result<PuzzleKind, String> {
    PuzzleKind::of_size(size).ok_or_else(|| format!("size must be between 2 and 7, not {size}"))
}

/// The solved cube of the given kind with a move expression applied; errors
/// are the bare reason.
fn expression_cube(text: &str, kind: PuzzleKind) -> Result<Facelets, String> {
    let moves = parse_expression(text, kind).map_err(|error| error.to_string())?;
    Facelets::solved(kind)
        .apply(&moves)
        .map_err(|error| error.to_string())
}

/// The solved cube of the given kind with a scramble applied.
fn scrambled(text: &str, kind: PuzzleKind) -> Result<Facelets, String> {
    expression_cube(text, kind)
        .map_err(|error| format!("scramble is not valid move notation ({error})"))
}

fn read_cube(input: &InputArgs) -> Result<Facelets, String> {
    read_input(input).map(|(cube, _)| cube)
}

/// The cube, and the color scheme its colors were written in when it was
/// given as colors with centers to read it from.
fn read_input(input: &InputArgs) -> Result<(Facelets, Option<ColorScheme>), String> {
    let (cube, scheme) = read_unchecked(input)?;
    state_check::require(&cube)?;
    Ok((cube, scheme))
}

/// Like `read_input`, for commands that judge the state themselves (`canon`
/// accepts any coloring of the six colors, so its centers need not be a
/// rotation of the cube).
fn read_unchecked(input: &InputArgs) -> Result<(Facelets, Option<ColorScheme>), String> {
    if let Some(scramble) = &input.scramble {
        return Ok((scrambled(scramble, kind_of(input.size)?)?, None));
    }
    let text = match input.cube.as_slice() {
        [] => return Err(solved_start::NO_CUBE.into()),
        [single] if single == "-" => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| format!("cannot read stdin: {error}"))?;
            text
        }
        parts => parts.join(" "),
    };
    parse_text(&text, input.scheme, input.size)
}

/// Reads cube text: cube text always wins; only text that is no cube is tried
/// as a move expression for a cube of `size`.
fn parse_text(
    text: &str,
    scheme: Option<ColorScheme>,
    size: usize,
) -> Result<(Facelets, Option<ColorScheme>), String> {
    match parse_cube(text, scheme) {
        Ok(cube) => Ok((cube, scheme.or_else(|| color_scheme_of(text)))),
        Err(error) if text.trim().is_empty() => Err(error.to_string()),
        Err(error) => {
            let expression = kind_of(size).and_then(|kind| expression_cube(text, kind));
            match expression {
                Ok(cube) => Ok((cube, None)),
                Err(reason) => Err(format!(
                    "not a cube ({error}) and not a move expression for a {size}x{size}x{size} ({reason}){}",
                    state_check::count_hint(text).map_or_else(String::new, |hint| format!("\n{hint}"))
                )),
            }
        }
    }
}

/// The state `--to` names, read like the start; a move expression is applied
/// to the solved cube of the start's size. A text with `X` or `?` stickers
/// gives a partial target; every other form is a complete state.
fn read_target(text: &str, args: &SolveArgs, start: &Facelets) -> Result<Target, String> {
    // A pattern name is taken only when the whole text is that name.
    if let Some(named) = named_target(text, start.kind()) {
        return named
            .map(|state| Target::complete(&state))
            .map_err(|error| format!("--to: {error}"));
    }
    if let Some(subset) = named_subset(text, start.kind()) {
        return subset.map_err(|error| format!("--to: {error}"));
    }
    if let Some(partial) =
        parse_partial(text, args.input.scheme).map_err(|error| format!("--to: {error}"))?
    {
        if partial.kind() != start.kind() {
            return Err(format!(
                "--to: the start is a {0}x{0}x{0} but the target has {1} stickers, a {2}x{2}x{2}",
                start.kind().size(),
                partial.kind().facelet_count(),
                partial.kind().size()
            ));
        }
        return Ok(partial);
    }
    let (target, _) = parse_text(text, args.input.scheme, start.kind().size())
        .map_err(|error| format!("--to: {error}"))?;
    state_check::require(&target).map_err(|error| format!("--to: {error}"))?;
    Ok(Target::complete(&target))
}

/// The engine for a cube size, prepared; progress goes to standard error. From
/// the 4x4x4 up it also tries known patterns and answers with the shorter solution.
fn solver_for(kind: PuzzleKind) -> Box<dyn Solver> {
    let engine = engine_for(kind);
    if kind.size() >= 4 {
        Box::new(PatternShortcut::new(engine))
    } else {
        engine
    }
}

fn engine_for(kind: PuzzleKind) -> Box<dyn Solver> {
    match kind {
        PuzzleKind::Cube2 => Box::new(PocketSolver::new()),
        PuzzleKind::Cube3 => Box::new(Min2PhaseSolver::new()),
        PuzzleKind::Cube4 => {
            let four = FourSolver::new(Arc::new(Min2PhaseSolver::new()));
            eprintln!("preparing the 4x4x4 tables (built now, about 12 MB)");
            four.prepare();
            Box::new(four)
        }
        PuzzleKind::Cube5 => {
            let five = FiveSolver::new(Arc::new(Min2PhaseSolver::new()));
            match five.cache_dir() {
                Some(dir) => eprintln!(
                    "preparing the 5x5x5 tables (cache directory {})",
                    dir.display()
                ),
                None => eprintln!("preparing the 5x5x5 tables (no cache directory: built now)"),
            }
            five.prepare();
            if five.cache_write_failed() == Some(true) {
                eprintln!("warning: the tables could not be saved; they are built again next time");
            }
            Box::new(five)
        }
        PuzzleKind::Cube6 | PuzzleKind::Cube7 => {
            let three = Arc::new(Min2PhaseSolver::new());
            let four = FourSolver::new(three.clone());
            let five = FiveSolver::new(three.clone());
            eprintln!(
                "preparing the 4x4x4 and 5x5x5 tables for the wings (the center tables are built on first use)"
            );
            four.prepare();
            five.prepare();
            Box::new(DwaltonSolver::new(
                kind,
                three,
                Arc::new(four),
                Arc::new(five),
            ))
        }
    }
}

/// The limits `solve` applies without options: 21 moves for the 2x2x2 and 3x3x3.
fn default_options(kind: PuzzleKind) -> SolveOptions {
    match kind {
        PuzzleKind::Cube2 | PuzzleKind::Cube3 => SolveOptions::default().with_max_moves(21),
        _ => SolveOptions::default(),
    }
}

fn solve(args: SolveArgs) -> Result<(), String> {
    if args.pictures.icons && !args.group.is_empty() {
        return Err("--group cannot be combined with --icons: the pictures are already headed by the solver phases, and the dots (` · `) mark units in the written answer only".into());
    }
    within_cmd::check(&args)?;
    if let Some(now) = &args.now {
        let depth = args.depth.unwrap_or(3);
        if depth > 5 {
            return Err(format!("--depth must be 0 to 5 with --now, not {depth}"));
        }
        let target = match &args.to {
            Some(text) => {
                let start = read_cube(&args.input)?;
                let target = read_target(text, &args, &start)?;
                Some(target.as_state().ok_or_else(|| {
                    "--to: with --now the target must be a complete cube, without open stickers"
                        .to_owned()
                })?)
            }
            None => None,
        };
        let shared = recover_cmd::Shared {
            target: target.as_ref(),
            input: &args.input,
            verbose: args.verbose,
            notation: args.notation,
            pictures: &args.pictures,
        };
        return recover_cmd::run_now(&shared, now, &args.way, depth);
    }
    let start = match solved_start::of_target(&args)? {
        Some(solved) => solved,
        None => read_cube(&args.input)?,
    };
    let target = args
        .to
        .as_deref()
        .map(|text| read_target(text, &args, &start))
        .transpose()?;
    if let Some(k) = args.within {
        if let within_cmd::Done::Printed = within_cmd::run(&args, k, &start, target.as_ref())? {
            return Ok(());
        }
    }
    // A target with open stickers is a set of states, not one: the engine
    // searches for the set. Every other target is a state and keeps the
    // relative-state path below.
    if let Some(partial) = target.as_ref().filter(|t| t.as_state().is_none()) {
        return partial_cmd::run(&args, &start, partial);
    }
    let target = target.and_then(|t| t.as_state());
    // The cube to solve: the start itself, or what is left of it after the
    // target is taken out.
    // A target whose centers differ in color on one face is reached through
    // a center fix-up; its cube to solve is made once the engine is at hand.
    let mixed = target.as_ref().is_some_and(|t| !centers::uniform(t));
    let facelets = match &target {
        Some(target) if !mixed => {
            relative_state(&start, target).map_err(|error| error.to_string())?
        }
        _ => start.clone(),
    };
    // A 3x3x3, 5x5x5 or 7x7x7 whose centers a slice turn moved is solved as
    // the cube with its colors named after its centers; the moves are the
    // same, and they leave every face one color.
    let renamed = match target {
        None => oriented_by_centers(&facelets),
        Some(_) => None,
    };
    let oriented = renamed.is_some();
    let facelets = renamed.unwrap_or(facelets);
    if oriented {
        eprintln!(
            "note: the face centers are not on their own faces; the solution makes every face the color of its center"
        );
    }
    improve::check(&args, facelets.kind())?;
    optimal_cmd::check(&args, facelets.kind())?;
    let restricted = match args.moves.as_deref() {
        Some(text) => restricted_cmd::read_set(text, facelets.kind())?,
        None => None,
    };
    enumerate_cmd::check(&args, facelets.kind())?;
    if mixed && restricted.is_some() {
        return Err(
            "--moves cannot be combined with a target whose centers differ in color on a face: the center fix-up uses inner slices"
                .into(),
        );
    }
    if enumerate_cmd::wanted(&args) {
        return enumerate_cmd::run(&args, &start, &facelets, target.as_ref(), restricted);
    }
    let solver = if args.optimal {
        optimal_cmd::solver(args.max_memory, args.nodes)?
    } else {
        match &restricted {
            Some(alphabet) => {
                restricted_cmd::solver_for(alphabet.clone(), args.max_memory.unwrap_or(1500))?
            }
            None => solver_for(facelets.kind()),
        }
    };

    let restricted_set = restricted.is_some();
    // A restricted set has no default limit: its answers are longer than 21.
    let mut options = match restricted {
        Some(_) => SolveOptions::default(),
        None => default_options(facelets.kind()),
    };
    if let Some(moves) = args.max_moves {
        options = options.with_max_moves(moves);
    }
    if let Some(millis) = args.budget_ms {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    options = options.with_slices(args.slices);
    let plan = match (&target, mixed) {
        (Some(target), true) => {
            // the moves of the target, when it is given as moves, bound the fix-up
            let moves = args
                .to
                .as_deref()
                .and_then(|text| parse_expression(text, start.kind()).ok());
            centers::plan(solver.as_ref(), &start, target, &options, moves.as_ref())
                .map_err(|error| error.to_string())?
        }
        _ => None,
    };
    let facelets = match &plan {
        Some(plan) => relative_state(&start, plan.goal()).map_err(|error| error.to_string())?,
        None => facelets,
    };
    let mode = improve::mode(&args).or_else(|| {
        improve::default_mode(&args, facelets.kind(), target.is_some(), restricted_set)
            .filter(|_| !args.optimal)
    });
    // --max-time, else the default of the big cubes; --improve spends its
    // own budget (-b) and the optimal search has its own (-b, 60 s).
    let spends_budget = matches!(
        mode,
        Some(improve::Mode::Time | improve::Mode::TimeParallel)
    );
    if args.budget_ms.is_none() && !spends_budget && !args.optimal {
        let limit = args.way.max_time.or_else(|| {
            (facelets.kind().size() >= 4 && !restricted_set).then_some(DEFAULT_MAX_TIME_MS)
        });
        if let Some(ms) = limit.filter(|&ms| ms > 0) {
            options = options.with_time_budget(Duration::from_millis(ms));
        }
    }
    let limited = match &plan {
        Some(plan) => plan.limit(&options),
        None => Ok(options),
    };
    let result = match limited {
        Err(failed) => failed,
        Ok(options) => {
            let result = match mode {
                Some(mode) if facelets.kind() == PuzzleKind::Cube3 => {
                    improve::solve(&facelets, options, mode, args.verbose)
                }
                Some(mode) => improve::solve_reduction(solver.as_ref(), &facelets, options, mode),
                None => solver.solve(&facelets, &options),
            };
            match &plan {
                Some(plan) => plan.complete(result),
                None => result,
            }
        }
    };

    let (result, adopted) = if args.pattern_aware {
        pattern_aware_cmd::apply(
            &pattern_aware_cmd::Request {
                config: args.config.as_deref(),
                start: &start,
                target: target.as_ref(),
                cube: &facelets,
                max_moves: args.max_moves,
                slices: args.slices,
                verbose: args.verbose,
            },
            result,
        )
    } else {
        (result, None)
    };

    if args.verbose {
        eprintln!(
            "elapsed {:?}, probes {}",
            result.metrics.elapsed,
            result
                .metrics
                .probes
                .map_or_else(|| "-".into(), |p| p.to_string())
        );
    }
    if let (true, Some(plan)) = (args.verbose, &plan) {
        eprintln!(
            "center fix-up: {} moves ({})",
            plan.tail().len(),
            plan.method()
        );
    }
    if args.verbose {
        if let Some(message) = &result.message {
            eprintln!("{message}");
        }
    }
    match result.outcome {
        Ok(solution) => answer::print(&answer::Solved {
            args: &args,
            start: &start,
            facelets: &facelets,
            target,
            solution: &solution,
            stages: &result.stages,
            oriented,
            adopted: adopted.as_ref().map(|a| a.line()),
        }),
        Err(Failure::InvalidInput) if target.is_some() => Err(format!(
            "the start or the target state is impossible{}",
            result
                .message
                .map_or_else(String::new, |m| format!(" ({m})"))
        )),
        Err(failure) => {
            if failure == Failure::BudgetExhausted {
                LIMIT_REACHED.store(true, Ordering::Relaxed);
            }
            if restricted.is_some() {
                return Err(restricted_cmd::failure_text(failure, result.message));
            }
            match (failure, options.time_budget) {
                (Failure::BudgetExhausted, Some(limit)) => {
                    // The engines' own words say more only when they name a stage.
                    let detail = result
                        .message
                        .filter(|message| !message.contains("time limit"))
                        .map_or_else(String::new, |message| format!(" ({message})"));
                    Err(format!("{}{detail}", limit_message(limit, facelets.kind())))
                }
                _ => Err(failure_message(failure, result.message)),
            }
        }
    }
}

/// The error of a solve that reached its time limit.
fn limit_message(limit: Duration, kind: PuzzleKind) -> String {
    let millis = limit.as_millis();
    let seconds = if millis.is_multiple_of(1000) {
        (millis / 1000).to_string()
    } else {
        format!("{:.3}", millis as f64 / 1000.0)
            .trim_end_matches('0')
            .to_owned()
    };
    let size = kind.size();
    format!(
        "time limit of {seconds} s reached on the {size}x{size}x{size} (raise it with --max-time MS)"
    )
}

/// The text of a failed solve, as `solve` and `solve --now` print it.
fn failure_message(failure: Failure, message: Option<String>) -> String {
    let reason = match failure {
        Failure::CentersOffFace => {
            return format!(
            "{CENTERS_OFF_FACE}\nhint: turn the cube so the centers match, or undo the slice turn"
        )
        }
        Failure::InvalidInput => "the cube state is impossible",
        Failure::BudgetExhausted => "the time budget ran out",
        Failure::Exhausted => "no solution within the move limit",
        Failure::Internal => "internal error",
    };
    format!(
        "{reason}{}",
        message.map_or_else(String::new, |m| format!(" ({m})"))
    )
}
