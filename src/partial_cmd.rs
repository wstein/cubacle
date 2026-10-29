//! `solve --to` with a target that leaves stickers open.
//!
//! A target with no open sticker never comes here: it is a state, and the
//! relative-state path in `main` solves it as before. A partial target goes to
//! the engine's `solve_to`; what the free stickers ended in is not printed. On a
//! 4x4x4 to 7x7x7 the only partial target is the centers (`centers`, or `X`
//! text with the centers fully specified), solved by `centers::solve_centers_only`.

use std::time::Duration;

use cubacle::centers::{self, CenterGoal};
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Target};

use crate::cli::SolveArgs;
use crate::engine::{default_options, solver_for};
use crate::{enumerate_cmd, metrics_out, phase_dots};

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
    if args.json {
        return refuse("--json");
    }
    if args.improve || args.parallel {
        return refuse("--improve and --parallel");
    }
    if args.nodes.is_some() {
        return refuse("--nodes");
    }
    Ok(())
}

/// Solves `start` to `target` and prints the moves.
pub fn run(args: &SolveArgs, start: &Facelets, target: &Target) -> Result<(), String> {
    check(args)?;
    let kind = start.kind();
    let exact = args.optimal || args.any_orientation;
    if exact && kind != PuzzleKind::Cube3 {
        return Err(format!(
            "{} with a --to target works on the 3x3x3 only, not the {1}x{1}x{1}",
            if args.optimal {
                "--optimal"
            } else {
                "--any-orientation"
            },
            kind.size()
        ));
    }
    // The reduction engines solve one subset: the centers. Say so before their
    // tables are built when the target is another one.
    let centers = kind.size() >= 4;
    if centers {
        CenterGoal::read(start, target)?;
    }
    let solver = solver_for(kind);
    let mut options: SolveOptions = default_options(kind);
    if let Some(moves) = args.max_moves {
        options = options.with_max_moves(moves);
    }
    if let Some(millis) = args.budget_ms.or(args.way.max_time.filter(|&ms| ms > 0)) {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    options = options.with_slices(args.slices);
    if exact {
        options = options
            .with_exact(true)
            .with_free_orientation(args.any_orientation);
    }
    if let Some(megabytes) = args.max_memory {
        options = options.with_max_memory(megabytes);
    }
    // Targets that need the two-phase search take its counted improvement when asked.
    if let Some(probes) = args.probes {
        options = options.with_min_probes(probes);
    }
    let result = if centers {
        centers::solve_centers_only(solver.as_ref(), start, target, &options)
    } else {
        solver.solve_to(start, target, &options)
    };
    if args.verbose {
        eprintln!("elapsed {:?}", result.metrics.elapsed);
    }
    match result.outcome {
        Ok(solution) => {
            let reached = if args.any_orientation {
                target.reached_by_any_rotation(start, &solution)
            } else {
                target.reached_by(start, &solution)
            };
            if !reached {
                return Err("internal error: solution does not reach the target".into());
            }
            eprintln!("moves {}", solution.len());
            eprintln!("{}", metrics_out::line(kind, &solution));
            if let Some(message) = &result.message {
                eprintln!("{message}");
            }
            let solution = if args.inverse {
                crate::inverse::moves(&solution)
            } else {
                solution
            };
            let (text, _) = phase_dots::write(
                args.notation,
                kind,
                &solution,
                &[],
                false,
                phase_dots::Grouping {
                    phases: false,
                    axis: false,
                },
            );
            println!("{text}");
            Ok(())
        }
        Err(failure) => Err(result.message.unwrap_or_else(|| {
            match failure {
                Failure::InvalidInput => "the start or the target is impossible",
                Failure::CentersOffFace => crate::failure::CENTERS_OFF_FACE,
                Failure::BudgetExhausted => "the time budget ran out",
                Failure::Exhausted => "no solution within the move limit",
                Failure::Internal => "internal error",
            }
            .into()
        })),
    }
}
