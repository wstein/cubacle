//! The `recover` command: a person followed a printed solution and went wrong.

use std::thread;
use std::time::{Duration, Instant};

use clap::Args;
use cubacle::notation::{parse_cube, parse_expression, Notation};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use cubacle::recover::{diagnose_staged, locate, Advice, Limits, PhaseInfo, Plan, Scope};

use crate::metrics_out;
use crate::recover_icons;
use crate::stage_phases;
use crate::steps::PictureArgs;
use crate::{default_options, failure_message, read_cube, solver_for, InputArgs};

/// The options only the way back needs: shared by every command that takes
/// a cube as it is now.
#[derive(Args)]
pub(crate) struct RecoverOptions {
    /// The solution the person followed (default: the one `solve` prints for
    /// the original cube)
    #[arg(long, value_name = "MOVES", requires = "now")]
    solution: Option<String>,
    /// The move you stopped at: you did move N (counting from 1) and noticed
    /// the cube no longer matches. The mistake is at move N or before; later
    /// moves are ignored
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..), requires = "now")]
    at_move: Option<u32>,
    /// The move you are sure the cube was still right after (a hint, not a
    /// fact: the search widens past it when nothing is found). Needs --at-move
    #[arg(long, value_name = "M", requires_all = ["at_move", "now"])]
    sure_until: Option<u32>,
    /// Skip the search for the mistake and only solve the current cube
    #[arg(long, requires = "now")]
    fresh: bool,
    /// Stray moves after a mistake the search allows, 0 to 3 (default 2)
    #[arg(long, default_value_t = 2, requires = "now", value_parser = clap::value_parser!(u8).range(0..=3))]
    strays: u8,
    /// Look for one mistake only (default: also two)
    #[arg(long, requires = "now")]
    one_mistake: bool,
    /// For two mistakes try every move and every extra move: slow and large on
    /// the big cubes
    #[arg(long, conflicts_with = "one_mistake", requires = "now")]
    thorough: bool,
    /// Time limit of the search in milliseconds, 0 for none (default 8000)
    #[arg(long, default_value_t = 8000, requires = "now", value_name = "MS")]
    max_time: u64,
}

#[derive(Args)]
pub(crate) struct RecoverArgs {
    /// The cube the solution was made for, as `solve` takes it
    #[command(flatten)]
    input: InputArgs,
    /// The cube as it is now: an Orbit64 token, URFDLB facelets or colours
    /// (quote it when it has spaces)
    #[arg(long, value_name = "CUBE")]
    now: String,
    #[command(flatten)]
    options: RecoverOptions,
    /// Moves the search for a way back onto the path may use, 0 to 5 (default 3)
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..=5))]
    depth: u8,
    /// Print timings to standard error
    #[arg(short, long)]
    verbose: bool,
    /// How to write the moves: wca (Rw, 3Rw), sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
    #[command(flatten)]
    pictures: PictureArgs,
}

/// What the way back shares with the command that asks for it.
pub(crate) struct Shared<'a> {
    pub(crate) input: &'a InputArgs,
    pub(crate) verbose: bool,
    pub(crate) notation: Notation,
    pub(crate) pictures: &'a PictureArgs,
}

pub(crate) fn run(args: RecoverArgs) -> Result<(), String> {
    let shared = Shared {
        input: &args.input,
        verbose: args.verbose,
        notation: args.notation,
        pictures: &args.pictures,
    };
    run_now(&shared, &args.now, &args.options, args.depth)
}

/// Explains where the cube `now` went wrong against the solution of the
/// original cube, and the way back.
pub(crate) fn run_now(
    shared: &Shared,
    now: &str,
    way: &RecoverOptions,
    depth: u8,
) -> Result<(), String> {
    let notation = shared.notation;
    let input = shared.input;
    let original = read_cube(input)?;
    let current = parse_cube(now, input.scheme).map_err(|e| format!("--now: {e}"))?;
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
    if way.fresh {
        let (moves, stages) = solve_staged(&current)?;
        println!("{}", notation.write(current.kind(), &moves));
        eprintln!("{}", metrics_out::line(current.kind(), &moves));
        if shared.pictures.icons {
            let options = shared.pictures.options(input.scheme, notation);
            print!(
                "{}",
                recover_icons::render_fresh(&options, &current, &moves, &stages)?
            );
        }
        return Ok(());
    }
    let (solution, stages): (MoveSequence, Vec<_>) = match &way.solution {
        Some(text) => (
            parse_expression(text, original.kind())
                .map_err(|e| format!("--solution is not valid move notation ({e})"))?,
            Vec::new(),
        ),
        None => solve_staged(&original)?,
    };
    // The engine's own stages where it reports them, else the state-based cut.
    let found = stage_phases::of(&stages, &original, &solution);
    let infos: Vec<PhaseInfo> = found
        .iter()
        .map(|p| PhaseInfo {
            number: p.number,
            name: p.name.to_owned(),
            start: p.moves.start,
        })
        .collect();
    if shared.verbose {
        eprintln!(
            "solution of {} moves after {:?}",
            solution.len(),
            started.elapsed()
        );
    }
    let limits = Limits {
        resume_depth: usize::from(depth),
        strays: usize::from(way.strays),
        two_mistakes: !way.one_mistake,
        thorough: way.thorough,
        max_time: (way.max_time > 0).then(|| Duration::from_millis(way.max_time)),
    };
    let hint = way.at_move.map(|n| Scope {
        at_move: n as usize,
        sure_until: way.sure_until.map(|m| m as usize),
        phase_starts: infos.iter().map(|p| p.start).collect(),
    });
    // Exactly on the path: nothing to compare with.
    let on_path = locate(&original, &current, &solution).map_err(|e| e.to_string())?;
    let mut fresh_stages = Vec::new();
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
            let fresh = solve_staged(&current);
            if shared.verbose {
                eprintln!("fresh solve {:?}", t.elapsed());
            }
            let (staged, took) = search.join().expect("the search does not panic");
            if shared.verbose {
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
            let fresh = fresh.and_then(|(moves, stages)| {
                fresh_stages = stages;
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
    print!(
        "{}",
        plan.with(&|mv| notation.write_move(original.kind(), mv))
    );
    if let Advice::Recover(route) = &plan.advice {
        eprintln!(
            "recover route {}",
            metrics_out::line(original.kind(), &route.moves)
        );
    }
    if let Some(fresh) = &plan.fresh {
        eprintln!("fresh {}", metrics_out::line(original.kind(), fresh));
    }
    if shared.pictures.icons {
        let options = shared.pictures.options(input.scheme, notation);
        print!(
            "{}",
            recover_icons::render(&options, &original, &current, &plan, &found, &fresh_stages)?
        );
    }
    Ok(())
}
