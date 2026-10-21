use std::io::Read;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::notation::{parse_cube, ColourScheme};
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Solver};
use cubacle::{apply_moves, from_moves, random_cube, random_moves};

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
    /// Print a random scramble
    Scramble {
        /// Number of moves
        #[arg(short = 'n', long, default_value_t = 25)]
        moves: u16,
    },
    /// Print a uniformly random cube state as facelets
    Random,
    /// Translate any supported notation into canonical URFDLB facelets
    Convert(InputArgs),
    /// Print the facelets reached by applying a scramble to a solved cube
    Facelets {
        /// Moves in standard notation, e.g. "R U R' U'"
        scramble: String,
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
    /// Colour scheme: western-boy, japanese-bl-wt or six letters in URFDLB order
    /// (default: read from the centres, western-boy without centres)
    #[arg(long)]
    scheme: Option<ColourScheme>,
}

#[derive(Args)]
struct SolveArgs {
    #[command(flatten)]
    input: InputArgs,
    /// Longest solution to accept
    #[arg(short = 'm', long, default_value_t = 21)]
    max_moves: u8,
    /// Hard time limit in milliseconds
    #[arg(short, long)]
    budget_ms: Option<u64>,
    /// Also print timing and search statistics to stderr
    #[arg(short, long)]
    verbose: bool,
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
        Command::Convert(input) => {
            println!("{}", read_cube(&input)?);
            Ok(())
        }
        Command::Scramble { moves } => {
            println!("{}", random_moves(moves).trim());
            Ok(())
        }
        Command::Random => {
            println!("{}", random_cube());
            Ok(())
        }
        Command::Facelets { scramble } => {
            let cube = from_moves(&scramble).ok_or("scramble is not valid move notation")?;
            println!("{cube}");
            Ok(())
        }
    }
}

fn read_cube(input: &InputArgs) -> Result<Facelets, String> {
    if let Some(scramble) = &input.scramble {
        let text = from_moves(scramble).ok_or("scramble is not valid move notation")?;
        return Facelets::parse(&text).map_err(|error| error.to_string());
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
        kind => {
            let size = kind.size();
            return Err(format!(
                "no engine for the {size}x{size}x{size} yet; only the 2x2x2 and 3x3x3 are solvable"
            ));
        }
    };

    let mut options = SolveOptions::default().with_max_moves(args.max_moves);
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
            if facelets.kind() == PuzzleKind::Cube3 {
                let replay = apply_moves(&facelets.to_string(), &solution.to_string());
                let solved = Facelets::solved(PuzzleKind::Cube3).to_string();
                if replay.as_deref() != Some(solved.as_str()) {
                    return Err("internal error: solution does not solve the cube".into());
                }
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
