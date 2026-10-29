//! The command line: clap argument structs and the command enum.

use clap::{Args, Parser, Subcommand, ValueEnum};
use cubacle::notation::{ColorScheme, Notation};
use cubacle::ranges::Optimize;

use crate::{
    canon_cmd, moves_cmd, order_cmd, patterns_cmd, phase_dots, photo_cmd, recover_cmd, steps,
    template_cmd,
};

/// Rubik's cube solver and generator.
#[derive(Parser)]
#[command(name = "cubacle", version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
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
pub(crate) struct InputArgs {
    /// An Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colors, or (when the
    /// text is no cube) a move expression such as "(R U)*3" for a cube of
    /// --size; spaces are allowed and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., conflicts_with = "scramble")]
    pub(crate) cube: Vec<String>,
    /// Solve the cube produced by this scramble instead
    #[arg(short, long)]
    pub(crate) scramble: Option<String>,
    /// Cube size for --scramble or a move expression (a cube given as text has
    /// its size already)
    #[arg(long, default_value_t = 3)]
    pub(crate) size: usize,
    /// Color scheme: boy, jp or six letters in URFDLB order (default for
    /// reading colors: from the centers, boy without centers; for `convert
    /// --to color`: the scheme the colors were read in, else boy)
    #[arg(long)]
    pub(crate) scheme: Option<ColorScheme>,
}

/// What `convert` writes.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Output {
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
pub(crate) struct ConvertArgs {
    #[command(flatten)]
    pub(crate) input: InputArgs,
    /// What to print
    #[arg(long, value_enum, default_value_t = Output::Facelets)]
    pub(crate) to: Output,
    /// When the input is a move expression, also print all seven move counts
    /// (htm qtm stm etm obtm btm atm) on stderr
    #[arg(long)]
    pub(crate) metrics: bool,
}

#[derive(Args)]
pub(crate) struct SolveArgs {
    #[command(flatten)]
    pub(crate) input: InputArgs,
    /// The cube as it is now, after following the solution of the cube above
    /// and going wrong: says where the mistake was and the way back (an
    /// Orbit64 token, URFDLB facelets or colors; quote it when it has spaces).
    /// With --to the way back leads to the target, not to the solved cube
    #[arg(long, value_name = "CUBE", conflicts_with_all = [
        "max_moves", "budget_ms", "probes", "improve", "parallel", "fast", "optimize",
        "slices", "moves", "all", "count", "raw", "optimal", "nodes", "max_memory", "group", "every",
        "pattern_aware",
    ])]
    pub(crate) now: Option<String>,
    #[command(flatten)]
    pub(crate) way: recover_cmd::RecoverOptions,
    /// Longest solution to accept (default: 21; none for the 4x4x4 and 5x5x5; a limit applies to the whole solution)
    #[arg(short = 'm', long)]
    pub(crate) max_moves: Option<u8>,
    /// Hard time limit in milliseconds
    #[arg(short, long)]
    pub(crate) budget_ms: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: make at least this many probes in the 3x3x3 search (4x4x4 and 5x5x5: default 500; deterministic, the same answer on every machine)
    #[arg(long, value_name = "N", conflicts_with = "improve")]
    pub(crate) probes: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: spend the time budget (-b, default 1000 ms) on a shorter solution, then print the best one
    #[arg(long)]
    pub(crate) improve: bool,
    /// 3x3x3 only: with --improve, search the six orientations concurrently (needs -b)
    #[arg(long)]
    pub(crate) parallel: bool,
    /// 3x3x3: return the first solution found, as before the counted default (plain solve makes 3000 probes for a shorter one)
    #[arg(long, conflicts_with_all = ["improve", "parallel", "probes"])]
    pub(crate) fast: bool,
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    pub(crate) verbose: bool,
    /// How to write the solution: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    pub(crate) notation: Notation,
    /// Post-solve compression (no search): `ranges` writes adjacent turns of one axis as
    /// layer ranges (2-3r in sign, 2-3Rw in wca) and reports both move counts; with
    /// --icons each range is one picture with one arrow
    #[arg(long, value_name = "WHAT")]
    pub(crate) optimize: Option<Optimize>,
    /// Allow single inner layers (2R) in 4x4x4 and 5x5x5 answers where they make them shorter
    #[arg(long)]
    pub(crate) slices: bool,
    /// Use only these moves: a list in any notation ("R U R2 U'", taken
    /// literally), generators ("<R,U>": every turn of R and U), or a name: htm
    /// (no restriction, the default), qtm, half-turns, quarter-slices. On the
    /// 3x3x3 the answer is shortest in the set, except qtm (the default
    /// answer in quarter turns, not proven shortest); a cube the set cannot
    /// solve is reported at once
    #[arg(long, value_name = "SET", conflicts_with_all = ["improve", "parallel", "probes", "slices"])]
    pub(crate) moves: Option<String>,
    /// 3x3x3 only: list every solution of the shortest length (with --moves, within the set), one order of commuting moves
    #[arg(long, conflicts_with_all = ["improve", "parallel", "probes", "slices", "optimize", "icons", "count"])]
    pub(crate) all: bool,
    /// 3x3x3 only: list this many solutions, shortest first, then stop
    #[arg(long, value_name = "N", conflicts_with_all = ["improve", "parallel", "probes", "slices", "optimize", "icons"])]
    pub(crate) count: Option<usize>,
    /// With --all or --count: also list the solutions up to this many moves,
    /// grouped by length. With --now: moves the search for a way back onto the
    /// path may use, 0 to 5 (default 3)
    #[arg(long, value_name = "D")]
    pub(crate) depth: Option<u8>,
    /// With --all or --count: list every order of commuting moves
    #[arg(long)]
    pub(crate) raw: bool,
    /// 3x3x3 only: find a provably shortest solution (half-turn metric). The default
    /// engine's answer comes first and stays the fallback; a large pruning table (cached in
    /// the cache directory) drives the search. Limited by -b (default 60000 ms) and --nodes;
    /// when the limit ends the search, the default answer is printed and stderr says "not
    /// proven shortest"
    #[arg(long, conflicts_with_all = ["fast", "probes", "improve", "parallel", "moves", "to", "all", "count"])]
    pub(crate) optimal: bool,
    /// With --optimal: stop after this many search nodes (the same on every machine)
    #[arg(long, value_name = "N", requires = "optimal")]
    pub(crate) nodes: Option<u64>,
    /// Hard memory cap in megabytes for the tables of --moves (default 1500) and
    /// --optimal (default: half of the available memory of this machine, at most
    /// 16000)
    #[arg(long, value_name = "MB")]
    pub(crate) max_memory: Option<usize>,
    /// For a cube that is only slightly scrambled: find an optimal route of at
    /// most K moves (1 to 7; write --within=5, plain --within means 6), searched
    /// from both ends, to the solved cube or to any --to state, in the alphabet
    /// the normal solver uses (--slices adds inner layers, --blocks every block
    /// of up to three layers). Limited by --max-time (default 60000 ms) and
    /// --max-memory (default 1000). Exit code 2: no route within K moves; 3: a
    /// limit ended the search
    #[arg(long, value_name = "K", num_args = 0..=1, require_equals = true, default_missing_value = "6", value_parser = clap::value_parser!(u8).range(1..=7))]
    pub(crate) within: Option<u8>,
    /// With --within: also allow the blocks of two and three layers from either
    /// side and the layers 2-3 (more routes, a larger search)
    #[arg(long)]
    pub(crate) blocks: bool,
    /// With --within: when no route of at most K moves exists, solve with the
    /// normal solver (the same options as without --within) and print its answer
    #[arg(long)]
    pub(crate) then_solve: bool,
    /// Solve to this state instead of to the solved cube: an Orbit64 token,
    /// URFDLB facelets, W/R/G/Y/O/B colors or a move expression applied to the
    /// solved cube of the start's size (one argument: quote it), or one of the
    /// names superflip, checkerboard, cube-in-cube, or the subset cross (3x3x3:
    /// the down cross; cross-u, cross-r, cross-f, cross-l, cross-b pick the
    /// face, the rest is free). An X (or ?) in facelets or
    /// colors marks a sticker that may show any color; the moves printed
    /// reach any such state. The moves printed take the start to the target.
    #[arg(long, value_name = "TARGET")]
    pub(crate) to: Option<String>,
    /// Mark hand units in the answer with " · ": phases (the solver phases; a
    /// solution of one phase, the 2x2x2, has no dot), axis (turns about one
    /// axis on disjoint layers, as in ATM) or both, "phases,axis" (a unit
    /// never crosses a phase border); none: a plain list
    #[arg(long, value_enum, value_delimiter = ',')]
    pub(crate) group: Vec<phase_dots::SolveGroup>,
    /// Cut the answer after every N moves, whatever the solver phases are: a
    /// " · " between the parts, or a heading above the pictures with --icons
    /// (for example 10 or 20). Replaces the solver phases
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u16).range(1..))]
    pub(crate) every: Option<u16>,
    /// Print the solution inverted: the moves that build the start cube from the
    /// solved cube (from the --to target when given). Phases, --every, --icons
    /// and --optimize ranges follow the inverted order; the counts stay those
    /// of the solution
    #[arg(long, conflicts_with_all = ["now", "all", "count"])]
    pub(crate) inverse: bool,
    /// Print the answer as the JSON object the REST server sends for /v1/solve
    /// (kind, solution, inverse, length, metrics, elapsedMicros) on stdout and
    /// nothing else there; the lines on stderr stay. The solution is the one
    /// of the equivalent request only (defaults differ from the server's)
    #[arg(long, conflicts_with_all = ["now", "all", "count", "icons", "group", "every"])]
    pub(crate) json: bool,
    /// Also look the cube up in the pattern library (built-in and the files of
    /// the config) and answer with the library route when it is strictly shorter
    /// than the solver's answer; the solver still runs, so the answer is never
    /// longer. 2x2x2: no effect. The default answer does not change without it
    #[arg(long, conflicts_with_all = ["moves", "optimal", "all", "count"])]
    pub(crate) pattern_aware: bool,
    /// With --pattern-aware: config file of the pattern sources (default:
    /// $CUBACLE_CONFIG, ./cubacle.toml, ~/.config/cubacle/config.toml)
    #[arg(long, value_name = "FILE", requires = "pattern_aware")]
    pub(crate) config: Option<std::path::PathBuf>,
    #[command(flatten)]
    pub(crate) pictures: steps::PictureArgs,
}
