mod icons;
mod improve;
mod moves_cmd;
mod recover_cmd;
mod recover_icons;
mod rows;
mod steps;
mod terminal;
mod views;

use std::io::Read;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::generate;
use cubacle::notation::orbit64;
use cubacle::notation::{colour_scheme_of, parse_cube, parse_expression, ColourScheme, Notation};
use cubacle::phases::{from_stages, phases};
use cubacle::puzzle::{
    reaches, relative_state, Facelets, Failure, PuzzleKind, SolveOptions, Solver,
};
use cubacle::ranges::{compressed_text, Optimise};

/// Rubik's cube solver and generator.
#[derive(Parser)]
#[command(name = "cubacle", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Solve a cube given as facelets or as a scramble
    Solve(SolveArgs),
    /// Explain a mistake made while following a solution and the way back
    Recover(recover_cmd::RecoverArgs),
    /// Print the plain move sequence an expression stands for
    Moves(moves_cmd::MovesArgs),
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
    /// Translate any supported notation into URFDLB facelets, colours or an Orbit64 token
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
    /// An Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colours, or (when the
    /// text is no cube) a move expression such as "(R U)*3" for a cube of
    /// --size; spaces are allowed and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., required_unless_present = "scramble", conflicts_with = "scramble")]
    cube: Vec<String>,
    /// Solve the cube produced by this scramble instead
    #[arg(short, long)]
    scramble: Option<String>,
    /// Cube size for --scramble or a move expression (a cube given as text has
    /// its size already)
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// Colour scheme: boy, jp or six letters in URFDLB order (default for
    /// reading colours: from the centres, boy without centres; for `convert
    /// --to color`: the scheme the colours were read in, else boy)
    #[arg(long)]
    scheme: Option<ColourScheme>,
}

/// What `convert` writes.
#[derive(Clone, Copy, ValueEnum)]
enum Output {
    /// Canonical URFDLB facelets
    Facelets,
    /// An Orbit64 state token
    Token,
    /// W/R/G/Y/O/B sticker letters in facelet order, written with --scheme
    /// (default: the scheme of colour input, else boy)
    #[value(name = "color", alias = "colour")]
    Colour,
}

#[derive(Args)]
struct ConvertArgs {
    #[command(flatten)]
    input: InputArgs,
    /// What to print
    #[arg(long, value_enum, default_value_t = Output::Facelets)]
    to: Output,
}

#[derive(Args)]
struct SolveArgs {
    #[command(flatten)]
    input: InputArgs,
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
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    verbose: bool,
    /// How to write the solution: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
    /// Post-solve compression (no search): `ranges` writes adjacent turns of one axis as
    /// layer ranges (2-3r in sign, 2-3Rw in wca) and reports both move counts
    #[arg(long, value_name = "WHAT", conflicts_with = "icons")]
    optimise: Option<Optimise>,
    /// Allow single inner layers (2R) in 4x4x4 and 5x5x5 answers where they make them shorter
    #[arg(long)]
    slices: bool,
    /// Solve to this state instead of to the solved cube: an Orbit64 token,
    /// URFDLB facelets, W/R/G/Y/O/B colours or a move expression applied to the
    /// solved cube of the start's size (one argument: quote it). The moves
    /// printed take the start to the target.
    #[arg(long, value_name = "TARGET")]
    to: Option<String>,
    #[command(flatten)]
    pictures: steps::PictureArgs,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Solve(args) => solve(args),
        Command::Recover(args) => recover_cmd::run(args),
        Command::Moves(args) => moves_cmd::run(args),
        Command::Convert(args) => {
            let (cube, detected) = read_input(&args.input)?;
            match args.to {
                Output::Facelets => println!("{cube}"),
                Output::Token => println!("{}", orbit64::encode(&cube).map_err(|e| e.to_string())?),
                Output::Colour => {
                    let scheme = args
                        .input
                        .scheme
                        .or(detected)
                        .unwrap_or(ColourScheme::WESTERN_BOY);
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

/// The cube, and the colour scheme its colours were written in when it was
/// given as colours with centres to read it from.
fn read_input(input: &InputArgs) -> Result<(Facelets, Option<ColourScheme>), String> {
    if let Some(scramble) = &input.scramble {
        return Ok((scrambled(scramble, kind_of(input.size)?)?, None));
    }
    let text = match input.cube.as_slice() {
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
    scheme: Option<ColourScheme>,
    size: usize,
) -> Result<(Facelets, Option<ColourScheme>), String> {
    match parse_cube(text, scheme) {
        Ok(cube) => Ok((cube, scheme.or_else(|| colour_scheme_of(text)))),
        Err(error) if text.trim().is_empty() => Err(error.to_string()),
        Err(error) => {
            let expression = kind_of(size).and_then(|kind| expression_cube(text, kind));
            match expression {
                Ok(cube) => Ok((cube, None)),
                Err(reason) => Err(format!(
                    "not a cube ({error}) and not a move expression for a {size}x{size}x{size} ({reason})"
                )),
            }
        }
    }
}

/// The state `--to` names, read like the start; a move expression is applied
/// to the solved cube of the start's size.
fn read_target(text: &str, args: &SolveArgs, start: &Facelets) -> Result<Facelets, String> {
    let (target, _) = parse_text(text, args.input.scheme, start.kind().size())
        .map_err(|error| format!("--to: {error}"))?;
    Ok(target)
}

/// The engine for a cube size, prepared; progress goes to standard error.
fn solver_for(kind: PuzzleKind) -> Box<dyn Solver> {
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
                "preparing the 4x4x4 and 5x5x5 tables for the wings (the centre tables are built on first use)"
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
    let start = read_cube(&args.input)?;
    let target = args
        .to
        .as_deref()
        .map(|text| read_target(text, &args, &start))
        .transpose()?;
    // The cube to solve: the start itself, or what is left of it after the
    // target is taken out.
    let facelets = match &target {
        Some(target) => relative_state(&start, target).map_err(|error| error.to_string())?,
        None => start.clone(),
    };
    improve::check(&args, facelets.kind())?;
    let solver = solver_for(facelets.kind());

    let mut options = default_options(facelets.kind());
    if let Some(moves) = args.max_moves {
        options = options.with_max_moves(moves);
    }
    if let Some(millis) = args.budget_ms {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    options = options.with_slices(args.slices);
    let result = match improve::mode(&args) {
        Some(mode) if facelets.kind() == PuzzleKind::Cube3 => {
            improve::solve(&facelets, options, mode, args.verbose)
        }
        Some(mode) => improve::solve_reduction(solver.as_ref(), &facelets, options, mode),
        None => solver.solve(&facelets, &options),
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
    match result.outcome {
        Ok(solution) => {
            // Every engine's answer is replayed here, whatever the size.
            let wanted = target
                .clone()
                .unwrap_or_else(|| Facelets::solved(start.kind()));
            if (start.kind() != PuzzleKind::Cube2 || target.is_some())
                && !reaches(&start, &solution, &wanted)
            {
                return Err(match target {
                    Some(_) => "internal error: solution does not reach the target".into(),
                    None => "internal error: solution does not solve the cube".into(),
                });
            }
            eprintln!("moves {}", solution.len());
            // The engine's own stages where it reports them, else the state-based cut.
            let found = if args.verbose || args.pictures.icons {
                from_stages(&result.stages, solution.len())
                    .or_else(|| phases(&facelets, &solution).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            if args.verbose {
                for phase in &found {
                    eprintln!("{}", steps::phase_line(phase));
                }
            }
            if args.pictures.icons {
                let mut options = args.pictures.options(args.input.scheme, args.notation);
                options.target = target;
                return steps::print_solution(&start, &solution, &found, &options);
            }
            if args.optimise == Some(Optimise::Ranges) {
                let (text, count) = compressed_text(args.notation, facelets.kind(), &solution);
                eprintln!("compressed moves {count}");
                println!("{text}");
                return Ok(());
            }
            println!("{}", args.notation.write(facelets.kind(), &solution));
            Ok(())
        }
        Err(Failure::InvalidInput) if target.is_some() => Err(format!(
            "the start or the target state is impossible{}",
            result
                .message
                .map_or_else(String::new, |m| format!(" ({m})"))
        )),
        Err(failure) => Err(failure_message(failure, result.message)),
    }
}

/// The text of a failed solve, as `solve` and `recover` print it.
fn failure_message(failure: Failure, message: Option<String>) -> String {
    let reason = match failure {
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
