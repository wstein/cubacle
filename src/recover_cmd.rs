//! The `recover` command: a person followed a printed solution and went wrong.

use std::thread;
use std::time::{Duration, Instant};

use clap::Args;
use cubacle::notation::{parse_cube, parse_expression, Notation};
use cubacle::phases::{from_stages, phases};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use cubacle::recover::{diagnose_staged, locate, Limits, PhaseInfo, Plan, Scope};

use crate::{default_options, failure_message, read_cube, solver_for, InputArgs};

#[derive(Args)]
pub(crate) struct RecoverArgs {
    /// The cube the solution was made for, as `solve` takes it
    #[command(flatten)]
    input: InputArgs,
    /// The cube as it is now: an Orbit64 token, URFDLB facelets or colours
    /// (quote it when it has spaces)
    #[arg(long, value_name = "CUBE")]
    now: String,
    /// The solution the person followed (default: the one `solve` prints for
    /// the original cube)
    #[arg(long, value_name = "MOVES")]
    solution: Option<String>,
    /// The move you stopped at: you did move N (counting from 1) and noticed
    /// the cube no longer matches. The mistake is at move N or before; later
    /// moves are ignored
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..))]
    at_move: Option<u32>,
    /// The move you are sure the cube was still right after (a hint, not a
    /// fact: the search widens past it when nothing is found). Needs --at-move
    #[arg(long, value_name = "M", requires = "at_move")]
    sure_until: Option<u32>,
    /// Skip the search for the mistake and only solve the current cube
    #[arg(long)]
    fresh: bool,
    /// Moves the search for a way back onto the path may use, 0 to 5 (default 3)
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..=5))]
    depth: u8,
    /// Stray moves after a mistake the search allows, 0 to 3 (default 2)
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(0..=3))]
    strays: u8,
    /// Look for one mistake only (default: also two)
    #[arg(long)]
    one_mistake: bool,
    /// For two mistakes try every move and every extra move: slow and large on
    /// the big cubes
    #[arg(long, conflicts_with = "one_mistake")]
    thorough: bool,
    /// Time limit of the search in milliseconds, 0 for none (default 8000)
    #[arg(long, default_value_t = 8000, value_name = "MS")]
    max_time: u64,
    /// Print timings to standard error
    #[arg(short, long)]
    verbose: bool,
    /// How to write the moves: wca (Rw, 3Rw) or sign (r, 3r)
    #[arg(long, default_value = "wca")]
    notation: Notation,
}

pub(crate) fn run(args: RecoverArgs) -> Result<(), String> {
    let notation = args.notation;
    let original = read_cube(&args.input)?;
    let current = parse_cube(&args.now, args.input.scheme).map_err(|e| format!("--now: {e}"))?;
    if current.kind() != original.kind() {
        return Err(format!(
            "the original cube is a {0}x{0}x{0} and the current one a {1}x{1}x{1}",
            original.kind().size(),
            current.kind().size()
        ));
    }
    let solver = solver_for(original.kind());
    let options = default_options(original.kind());
    let started = Instant::now();
    let solve_staged = |cube: &Facelets| {
        let result = solver.solve(cube, &options);
        let stages = result.stages;
        result
            .outcome
            .map(|moves| (moves, stages))
            .map_err(|failure| failure_message(failure, result.message))
    };
    let solve = |cube: &Facelets| solve_staged(cube).map(|(moves, _)| moves);
    if args.fresh {
        println!("{}", notation.write(&solve(&current)?));
        return Ok(());
    }
    let (solution, stages): (MoveSequence, Vec<_>) = match &args.solution {
        Some(text) => (
            parse_expression(text, original.kind())
                .map_err(|e| format!("--solution is not valid move notation ({e})"))?,
            Vec::new(),
        ),
        None => solve_staged(&original)?,
    };
    // The engine's own stages where it reports them, else the state-based cut.
    let found = from_stages(&stages, solution.len())
        .or_else(|| phases(&original, &solution).ok())
        .unwrap_or_default();
    let infos: Vec<PhaseInfo> = found
        .iter()
        .map(|p| PhaseInfo {
            number: p.number,
            name: p.name.to_owned(),
            start: p.moves.start,
        })
        .collect();
    if args.verbose {
        eprintln!(
            "solution of {} moves after {:?}",
            solution.len(),
            started.elapsed()
        );
    }
    let limits = Limits {
        resume_depth: usize::from(args.depth),
        strays: usize::from(args.strays),
        two_mistakes: !args.one_mistake,
        thorough: args.thorough,
        max_time: (args.max_time > 0).then(|| Duration::from_millis(args.max_time)),
    };
    let hint = args.at_move.map(|n| Scope {
        at_move: n as usize,
        sure_until: args.sure_until.map(|m| m as usize),
        phase_starts: infos.iter().map(|p| p.start).collect(),
    });
    // Exactly on the path: nothing to compare with.
    let on_path = locate(&original, &current, &solution).map_err(|e| e.to_string())?;
    let (diagnosis, fresh) = if on_path.is_some() {
        let staged = diagnose_staged(&original, &current, &solution, &limits, hint.as_ref());
        (staged.map_err(|e| e.to_string())?.diagnosis, None)
    } else {
        // The search and the fresh solve run side by side.
        thread::scope(|scope| {
            let search = scope.spawn(|| {
                let t = Instant::now();
                let d = diagnose_staged(&original, &current, &solution, &limits, hint.as_ref());
                (d, t.elapsed())
            });
            let t = Instant::now();
            let fresh = solve(&current);
            if args.verbose {
                eprintln!("fresh solve {:?}", t.elapsed());
            }
            let (staged, took) = search.join().expect("the search does not panic");
            if args.verbose {
                eprintln!("search {took:?}");
                if let Ok(staged) = &staged {
                    let tried: Vec<&str> = staged.tried.iter().map(|w| w.describe()).collect();
                    eprintln!(
                        "stages tried: {}; answered by {}",
                        tried.join(", "),
                        staged.stage.describe()
                    );
                }
            }
            let diagnosis = staged.map(|s| s.diagnosis);
            // Every fresh solution is replayed, as `solve` does (not on the 2x2x2).
            let fresh = fresh.and_then(|moves| {
                let solves = current.kind() == PuzzleKind::Cube2
                    || current.apply(&moves).is_ok_and(|end| end.is_solved());
                solves
                    .then_some(moves)
                    .ok_or_else(|| "internal error: it does not solve the cube".to_owned())
            });
            if let Err(reason) = &fresh {
                eprintln!("warning: no fresh solution ({reason})");
            }
            diagnosis
                .map_err(|e| e.to_string())
                .map(|d| (d, fresh.ok()))
        })?
    };
    let plan = Plan::new(diagnosis, solution, fresh).with_phases(infos);
    print!("{}", plan.with(&|mv| notation.write_move(mv)));
    Ok(())
}
