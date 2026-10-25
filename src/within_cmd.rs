//! `solve --within K`: an optimal route of at most K moves for a cube that is
//! only slightly scrambled, to the solved cube or to any target state.
//!
//! Exit codes of `solve`: 0 ok, 1 input error, 2 no route (or an impossible
//! state), 3 a limit (time or memory) ended the search.

use std::process::exit;
use std::time::Duration;

use cubacle::puzzle::{Facelets, SolveStage, Target};
use cubacle::within::{self, Kit, Limit, Limits, Outcome};

use crate::answer::{self, Solved};
use crate::SolveArgs;

/// Exit code when no route exists within K moves.
const EXIT_NO_ROUTE: i32 = 2;

/// What `run` leaves to the caller.
pub(crate) enum Done {
    /// The route was printed.
    Printed,
    /// No route, and `--then-solve` asks for the normal solver.
    Fallback,
}

/// Refuses the options that make no sense with `--within`, naming each.
pub(crate) fn check(args: &SolveArgs) -> Result<(), String> {
    if args.within.is_none() {
        for (set, name) in [(args.then_solve, "--then-solve"), (args.blocks, "--blocks")] {
            if set {
                return Err(format!("{name} needs --within"));
            }
        }
        return Ok(());
    }
    let always = [
        (args.now.is_some(), "--now"),
        (args.optimal, "--optimal"),
        (args.moves.is_some(), "--moves"),
        (args.count.is_some(), "--count"),
        (args.all, "--all"),
    ];
    for (set, name) in always {
        if set {
            let why = if name == "--all" {
                "lists one route only"
            } else {
                "cannot be combined"
            };
            return Err(format!("--within and {name}: {why}"));
        }
    }
    // The options of the normal solver matter only for its fallback.
    if !args.then_solve {
        let solver = [
            (args.max_moves.is_some(), "-m/--max-moves"),
            (args.budget_ms.is_some(), "-b/--budget-ms (use --max-time)"),
            (args.probes.is_some(), "--probes"),
            (args.improve, "--improve"),
            (args.parallel, "--parallel"),
            (args.fast, "--fast"),
        ];
        for (set, name) in solver {
            if set {
                return Err(format!(
                    "--within and {name}: the option belongs to the normal solver; add --then-solve to use both"
                ));
            }
        }
    }
    Ok(())
}

fn kit_of(args: &SolveArgs) -> Kit {
    if args.blocks {
        Kit::Blocks
    } else if args.slices {
        Kit::Slices
    } else {
        Kit::Outer
    }
}

fn stop(code: i32, message: &str) -> ! {
    eprintln!("error: {message}");
    exit(code)
}

/// Searches, prints the route, or reports why not. Exits with code 2 or 3 where
/// the search ends without a route, except when `--then-solve` asks to go on.
pub(crate) fn run(
    args: &SolveArgs,
    k: u8,
    start: &Facelets,
    target: Option<&Target>,
) -> Result<Done, String> {
    let goal = match target {
        Some(target) => target.as_state().ok_or_else(|| {
            "--within needs a complete target state; a target with open stickers (X) is not supported"
                .to_owned()
        })?,
        None => Facelets::solved(start.kind()),
    };
    let kit = kit_of(args);
    let (alphabet, metric) = within::describe(kit, start.kind());
    let limits = Limits {
        max_moves: usize::from(k),
        max_memory_mb: args.max_memory.unwrap_or(within::DEFAULT_MEMORY_MB),
        max_time: match args.way.max_time {
            Some(0) => Duration::from_secs(86_400 * 365),
            Some(millis) => Duration::from_millis(millis),
            None => within::DEFAULT_TIME,
        },
    };
    let report = within::search(start, &goal, kit, &limits);
    if args.verbose {
        let s = &report.stats;
        eprintln!(
            "elapsed {:?}, bound {}, stored {} moves deep ({} states, {} MB)",
            s.elapsed,
            s.bound
                .map_or_else(|| "unreachable".to_owned(), |b| b.to_string()),
            s.stored_depth,
            s.table_states,
            s.table_mb
        );
    }
    match report.outcome {
        Outcome::Found(solution) => {
            let stages: [SolveStage; 0] = [];
            answer::print(&Solved {
                args,
                start,
                facelets: start,
                target: target.and_then(Target::as_state),
                solution: &solution,
                stages: &stages,
                oriented: false,
                adopted: None,
            })?;
            Ok(Done::Printed)
        }
        Outcome::Excluded | Outcome::NotFound if args.then_solve => {
            eprintln!("no route within {k} moves; solved with the normal solver");
            Ok(Done::Fallback)
        }
        Outcome::Excluded | Outcome::NotFound => stop(
            EXIT_NO_ROUTE,
            &format!("no route of at most {k} moves in the {alphabet} alphabet ({metric})"),
        ),
        Outcome::Limit(limit) => {
            crate::LIMIT_REACHED.store(true, std::sync::atomic::Ordering::Relaxed);
            Err(match limit {
                Limit::Time => "time limit",
                Limit::Memory => "memory limit",
            }
            .to_owned())
        }
    }
}
