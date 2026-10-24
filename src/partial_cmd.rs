//! `solve --to` with a target that leaves stickers open.
//!
//! A target with no open sticker never comes here: it is a state, and the
//! relative-state path in `main` solves it as before. A partial target goes to
//! the engine's `solve_to`; what the free stickers ended in is not printed.

use std::time::Duration;

use cubacle::puzzle::{Facelets, Failure, SolveOptions, Target};

use crate::{default_options, enumerate_cmd, metrics_out, phase_dots, solver_for, SolveArgs};

/// Refuses the options that do not combine with a partial target (yet).
fn check(args: &SolveArgs) -> Result<(), String> {
    let refuse = |option: &str| {
        Err(format!(
            "{option} does not work with a --to target that has open stickers (X) yet"
        ))
    };
    if args.moves.is_some() {
        return refuse("--moves");
    }
    if enumerate_cmd::wanted(args) {
        return refuse("--all and --count");
    }
    if args.pictures.icons {
        return refuse("--icons");
    }
    if args.improve || args.parallel {
        return refuse("--improve and --parallel");
    }
    Ok(())
}

/// Solves `start` to `target` and prints the moves.
pub fn run(args: &SolveArgs, start: &Facelets, target: &Target) -> Result<(), String> {
    check(args)?;
    let kind = start.kind();
    // The reduction engines have no goal stages yet; say so before their tables are built.
    if kind.size() >= 4 {
        return Err(format!(
            "unsupported: the {0}x{0}x{0} engine cannot solve targets with open stickers yet (the 2x2x2 and 3x3x3 can)",
            kind.size()
        ));
    }
    let solver = solver_for(kind);
    let mut options: SolveOptions = default_options(kind);
    if let Some(moves) = args.max_moves {
        options = options.with_max_moves(moves);
    }
    if let Some(millis) = args.budget_ms {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    options = options.with_slices(args.slices);
    // Targets that need the two-phase search take its counted improvement when asked.
    if let Some(probes) = args.probes {
        options = options.with_min_probes(probes);
    }
    let result = solver.solve_to(start, target, &options);
    if args.verbose {
        eprintln!("elapsed {:?}", result.metrics.elapsed);
    }
    match result.outcome {
        Ok(solution) => {
            if !target.reached_by(start, &solution) {
                return Err("internal error: solution does not reach the target".into());
            }
            eprintln!("moves {}", solution.len());
            eprintln!("{}", metrics_out::line(kind, &solution));
            if let Some(message) = &result.message {
                eprintln!("{message}");
            }
            let (text, _) = phase_dots::write(args.notation, kind, &solution, &[], false, false);
            println!("{text}");
            Ok(())
        }
        Err(failure) => Err(result.message.unwrap_or_else(|| {
            match failure {
                Failure::InvalidInput => "the start or the target is impossible",
                Failure::BudgetExhausted => "the time budget ran out",
                Failure::Exhausted => "no solution within the move limit",
                Failure::Internal => "internal error",
            }
            .into()
        })),
    }
}
