mod icons;

use std::io::Read;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};
use cubacle::engines::five::FiveSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::generate;
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
    /// Translate any supported notation into canonical URFDLB facelets
    Convert(InputArgs),
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

#[derive(Args)]
struct SolveArgs {
    #[command(flatten)]
    input: InputArgs,
    /// Longest solution to accept (default: 21; none for the 5x5x5)
    #[arg(short = 'm', long)]
    max_moves: Option<u8>,
    /// Hard time limit in milliseconds
    #[arg(short, long)]
    budget_ms: Option<u64>,
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    verbose: bool,
    /// Show each step with a picture of the cube (graphics on a capable terminal)
    #[arg(long)]
    icons: bool,
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
        Command::Convert(input) => {
            println!("{}", read_cube(&input)?);
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
    let solver: Box<dyn Solver> = match facelets.kind() {
        PuzzleKind::Cube2 => Box::new(PocketSolver::new()),
        PuzzleKind::Cube3 => Box::new(Min2PhaseSolver::new()),
        PuzzleKind::Cube5 => Box::new(FiveSolver::new(Arc::new(Min2PhaseSolver::new()))),
        kind => {
            let size = kind.size();
            return Err(format!(
                "no engine for the {size}x{size}x{size} yet; only the 2x2x2, 3x3x3 and 5x5x5 are solvable"
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
    let result = solver.solve(&facelets, &options);

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
            if matches!(facelets.kind(), PuzzleKind::Cube3 | PuzzleKind::Cube5)
                && !facelets.apply(&solution).is_ok_and(|end| end.is_solved())
            {
                return Err("internal error: solution does not solve the cube".into());
            }
            if args.icons {
                return icons::print_solution(&facelets, &solution, args.input.scheme);
            }
            println!("{solution}");
            Ok(())
        }
        Err(failure) => {
            let reason = match failure {
                Failure::InvalidInput => "the cube state is impossible",
                Failure::BudgetExhausted => "the time budget ran out",
                Failure::Exhausted => "no solution within the move limit",
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
