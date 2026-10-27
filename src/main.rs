mod answer;
mod canon_cmd;
mod cli;
mod engine;
mod enumerate_cmd;
mod failure;
mod fetch;
mod icons;
mod improve;
mod input;
mod inverse;
mod metrics_out;
mod moves_cmd;
mod optimal_cmd;
mod order_cmd;
mod partial_cmd;
mod pattern_aware_cmd;
mod pattern_hint;
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
mod solve_cmd;
mod solved_start;
mod stage_phases;
mod state_check;
mod steps;
mod template_cmd;
mod terminal;
mod views;
mod within_cmd;

use std::process::ExitCode;
use std::sync::atomic::Ordering;

use cubacle::generate;
use cubacle::notation::orbit64;
use cubacle::notation::{parse_cube, parse_expression, ColorScheme};

use crate::cli::{Cli, Command, InputArgs, Output};
use crate::failure::{EXIT_ERROR, EXIT_LIMIT, LIMIT_REACHED};
use crate::input::{kind_of, read_input, scrambled};

/// A closed pipe (`cubacle ... | head`) ends the program quietly, as other
/// command line tools do; every other panic reports as usual.
fn quiet_on_closed_pipe() {
    let report = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let text = if let Some(text) = info.payload().downcast_ref::<String>() {
            text.as_str()
        } else {
            info.payload().downcast_ref::<&str>().copied().unwrap_or("")
        };
        if text.contains("Broken pipe") {
            std::process::exit(0);
        }
        report(info);
    }));
}

fn main() -> ExitCode {
    quiet_on_closed_pipe();
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
        Command::Solve(args) => solve_cmd::solve(*args),
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
