//! The `solve` command driver.

use std::sync::atomic::Ordering;
use std::time::Duration;

use cubacle::centers;
use cubacle::notation::parse_expression;
use cubacle::puzzle::{oriented_by_centers, relative_state, Failure, PuzzleKind, SolveOptions};

use crate::cli::SolveArgs;
use crate::engine::{default_options, solver_for};
use crate::failure::{failure_message, limit_message, DEFAULT_MAX_TIME_MS, LIMIT_REACHED};
use crate::input::{read_cube, read_target};
use crate::{
    answer, enumerate_cmd, improve, optimal_cmd, partial_cmd, pattern_aware_cmd, pattern_hint,
    recover_cmd, restricted_cmd, solved_start, within_cmd,
};

pub(crate) fn solve(args: SolveArgs) -> Result<(), String> {
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
    if args.verbose {
        pattern_hint::print(
            args.config.as_deref(),
            &start,
            target.as_ref().and_then(|t| t.as_state()).as_ref(),
            args.notation,
            args.slices,
        );
    }
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
