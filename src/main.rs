mod icons;
mod improve;

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
use cubacle::notation::{parse_cube, ColourScheme};
use cubacle::puzzle::{Facelets, Failure, MoveSequence, PuzzleKind, SolveOptions, Solver};

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
    /// Draw moves as small pictures (SVG, pixels, Kitty graphics or text)
    Icon(icons::IconArgs),
    /// Print a random scramble
    Scramble {
        /// Number of moves
        #[arg(short = 'n', long, default_value_t = 25)]
        moves: usize,
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
    },
    /// Print a random cube state as facelets
    Random {
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
    },
    /// Translate any supported notation into URFDLB facelets or an Orbit64 token
    Convert(ConvertArgs),
    /// Print the facelets reached by applying a scramble to a solved cube
    Facelets {
        /// Moves in standard notation, e.g. "R U R' U'"
        scramble: String,
        /// Cube size
        #[arg(long, default_value_t = 3)]
        size: usize,
    },
}

/// How a cube is written on the command line.
#[derive(Args)]
struct InputArgs {
    /// An Orbit64 token, URFDLB facelets or W/R/G/Y/O/B colours; spaces are
    /// allowed and need no quoting. "-" reads stdin.
    #[arg(num_args = 1.., required_unless_present = "scramble", conflicts_with = "scramble")]
    cube: Vec<String>,
    /// Solve the cube produced by this scramble instead
    #[arg(short, long)]
    scramble: Option<String>,
    /// Cube size for --scramble (a cube given as text has its size already)
    #[arg(long, default_value_t = 3)]
    size: usize,
    /// Colour scheme: boy, jp or six letters in URFDLB order
    /// (default: read from the centres, boy without centres)
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
    /// 3x3x3 only: keep improving until this many probes were made (deterministic: the same answer on every machine)
    #[arg(long, value_name = "N", conflicts_with = "improve")]
    probes: Option<u64>,
    /// 3x3x3 only: keep improving until the time budget (-b, default 1000 ms) is spent, then print the best solution
    #[arg(long)]
    improve: bool,
    /// 3x3x3 only: with --improve, search the six orientations concurrently (needs -b)
    #[arg(long)]
    parallel: bool,
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    verbose: bool,
    /// Allow single inner layers (2R) in 4x4x4 and 5x5x5 answers where they make them shorter
    #[arg(long)]
    slices: bool,
    /// Show each step with a picture of the cube (graphics on a capable terminal)
    #[arg(long)]
    icons: bool,
    /// Icon height in pixels for --icons (default: 32 per cube layer, at least 96)
    #[arg(long, requires = "icons", value_name = "PIXELS")]
    icon_size: Option<u32>,
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
        Command::Icon(args) => icons::run(args),
        Command::Convert(args) => {
            let cube = read_cube(&args.input)?;
            match args.to {
                Output::Facelets => println!("{cube}"),
                Output::Token => println!("{}", orbit64::encode(&cube).map_err(|e| e.to_string())?),
            }
            Ok(())
        }
        Command::Scramble { moves, size } => {
            println!("{}", generate::scramble(kind_of(size)?, moves));
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

/// The solved cube of the given kind with a scramble applied.
fn scrambled(text: &str, kind: PuzzleKind) -> Result<Facelets, String> {
    let moves: MoveSequence = text
        .parse()
        .map_err(|error| format!("scramble is not valid move notation ({error})"))?;
    Facelets::solved(kind)
        .apply(&moves)
        .map_err(|error| error.to_string())
}

fn read_cube(input: &InputArgs) -> Result<Facelets, String> {
    if let Some(scramble) = &input.scramble {
        return scrambled(scramble, kind_of(input.size)?);
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
    parse_cube(&text, input.scheme).map_err(|error| error.to_string())
}

fn solve(args: SolveArgs) -> Result<(), String> {
    let facelets = read_cube(&args.input)?;
    improve::check(&args, facelets.kind())?;
    let solver: Box<dyn Solver> = match facelets.kind() {
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
        PuzzleKind::Cube6 => {
            let three = Arc::new(Min2PhaseSolver::new());
            let four = FourSolver::new(three.clone());
            let five = FiveSolver::new(three.clone());
            eprintln!("preparing the 4x4x4, 5x5x5 and 6x6x6 tables (built now on first use)");
            four.prepare();
            five.prepare();
            Box::new(DwaltonSolver::new(
                PuzzleKind::Cube6,
                three,
                Arc::new(four),
                Arc::new(five),
            ))
        }
        kind => {
            let size = kind.size();
            return Err(format!(
                "no engine for the {size}x{size}x{size} yet; only the 2x2x2 to 6x6x6 are solvable"
            ));
        }
    };

    let mut options = SolveOptions::default();
    match (args.max_moves, facelets.kind()) {
        (Some(moves), _) => options = options.with_max_moves(moves),
        (None, PuzzleKind::Cube2 | PuzzleKind::Cube3) => options = options.with_max_moves(21),
        (None, _) => {}
    }
    if let Some(millis) = args.budget_ms {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    options = options.with_slices(args.slices);
    let result = match improve::mode(&args) {
        Some(mode) => improve::solve(&facelets, options, mode, args.verbose),
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
            if facelets.kind() != PuzzleKind::Cube2
                && !facelets.apply(&solution).is_ok_and(|end| end.is_solved())
            {
                return Err("internal error: solution does not solve the cube".into());
            }
            eprintln!("moves {}", solution.len());
            if args.icons {
                return icons::print_solution(
                    &facelets,
                    &solution,
                    args.input.scheme,
                    args.icon_size,
                );
            }
            println!("{solution}");
            Ok(())
        }
        Err(failure) => {
            let reason = match failure {
                Failure::InvalidInput => "the cube state is impossible",
                Failure::BudgetExhausted => "the time budget ran out",
                Failure::Exhausted => "no solution within the move limit",
                Failure::Internal => "internal error",
            };
            Err(format!(
                "{reason}{}",
                result
                    .message
                    .map_or_else(String::new, |m| format!(" ({m})"))
            ))
        }
    }
}
