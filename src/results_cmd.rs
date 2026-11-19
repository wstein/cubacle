//! `solve --results N`: several distinct solutions instead of one.
//!
//! The engine lists them ([`Solver::solve_many`]); the first is the solution a
//! plain solve prints, the others follow in the engine's order, none is sorted.
//! The text has one line per solution, `rank TAB counts TAB moves`, the counts
//! being all seven metrics as `moves --metrics` prints them; `--json` writes an
//! array of the solve reports; `--inverse` writes each solution inverted. A
//! summary goes to standard error. With `N = 1` none of this applies: the
//! command solves as always.

use std::time::Duration;

use cubacle::metrics;
use cubacle::puzzle::{
    reaches, solves_by_color, Facelets, Failure, PuzzleKind, SolveOptions, SolveResult, Solver,
};
use cubacle::report::SolveReport;

use crate::cli::SolveArgs;
use crate::failure::{failure_message, limit_message, note_limit};
use crate::{inverse, metrics_out};

/// The most solutions one run lists. Measured: see docs/cli/solve.md.
pub(crate) const MAX_RESULTS: usize = 10_000;

/// The number of solutions asked for when it is more than one.
pub(crate) fn wanted(args: &SolveArgs) -> Option<usize> {
    args.results.filter(|&n| n > 1)
}

/// Rejects a count out of range and the options a list does not combine with.
/// Runs before anything else: the options named here have paths of their own.
pub(crate) fn check(args: &SolveArgs) -> Result<(), String> {
    let Some(n) = args.results else {
        return Ok(());
    };
    if !(1..=MAX_RESULTS).contains(&n) {
        return Err(format!(
            "--results expects a number from 1 to {MAX_RESULTS}, not {n}"
        ));
    }
    if n == 1 {
        return Ok(());
    }
    let refused = [
        (args.pictures.icons, "--icons"),
        (args.optimize.is_some(), "--optimize"),
        (!args.group.is_empty(), "--group"),
        (args.every.is_some(), "--every"),
        (args.now.is_some(), "--now"),
        (args.all, "--all"),
        (args.count.is_some(), "--count"),
        (args.moves.is_some(), "--moves"),
        (args.optimal, "--optimal"),
        (args.improve, "--improve"),
        (args.parallel, "--parallel"),
        (args.fast, "--fast"),
        (args.within.is_some(), "--within"),
        (args.upright_centers, "--upright-centers"),
        (args.center_twist.is_some(), "--center-twist"),
        (args.twist_target.is_some(), "--twist-target"),
        (args.pattern_aware, "--pattern-aware"),
    ];
    for (set, name) in refused {
        if set {
            return Err(format!(
                "--results {n} cannot be combined with {name}{}",
                match name {
                    "--optimize" | "--group" | "--every" =>
                        " (a follow-up: the rewriting of a list of solutions is not built yet)",
                    "--icons" => " (pictures of several solutions are not supported)",
                    _ => "",
                }
            ));
        }
    }
    Ok(())
}

/// Rejects a cube size whose engine cannot list solutions.
pub(crate) fn check_kind(args: &SolveArgs, kind: PuzzleKind) -> Result<(), String> {
    match wanted(args) {
        Some(n) if kind.size() > 5 => Err(format!(
            "--results {n} is not supported for the {0}x{0}x{0}: its stages keep one route, no list of finished solutions (a follow-up)",
            kind.size()
        )),
        _ => Ok(()),
    }
}

/// What a list is solved from and checked against.
pub(crate) struct Request<'a> {
    pub args: &'a SolveArgs,
    pub n: usize,
    pub start: &'a Facelets,
    pub facelets: &'a Facelets,
    /// The state the moves lead to; `None` for the solved cube.
    pub target: Option<&'a Facelets>,
    /// The cube was solved with its colors named after its centers.
    pub oriented: bool,
}

/// Solves, checks every solution by replay and prints the list.
pub(crate) fn run(
    solver: &dyn Solver,
    request: &Request<'_>,
    options: &SolveOptions,
) -> Result<(), String> {
    let Request {
        args,
        n,
        start,
        facelets,
        target,
        oriented,
    } = *request;
    let kind = facelets.kind();
    let results = solver.solve_many(facelets, options, n);
    let first = results.first().ok_or("internal error: no result")?;
    if let Err(failure) = first.outcome {
        return Err(failed(failure, first, options.time_budget, kind, target));
    }
    let wanted_state = target
        .cloned()
        .unwrap_or_else(|| Facelets::solved(start.kind()));
    let mut listed: Vec<&SolveResult> = Vec::new();
    for result in &results {
        let Some(solution) = result.solution() else {
            continue;
        };
        let replays = if oriented {
            solves_by_color(start, solution)
        } else {
            reaches(start, solution, &wanted_state)
        };
        if (start.kind() != PuzzleKind::Cube2 || target.is_some()) && !replays {
            return Err("internal error: a listed solution does not solve the cube".into());
        }
        listed.push(result);
    }
    let mut spread = metrics_out::Spread::new(kind);
    let mut shortest = usize::MAX;
    for result in &listed {
        let solution = result.solution().expect("a solved result");
        spread.add(solution);
        shortest = shortest.min(solution.len());
    }
    if let Some(line) = spread.line() {
        eprintln!("{line}");
    }
    eprintln!("solutions {} (shortest {shortest} moves)", listed.len());
    if listed.len() < n {
        // a time limit can end a list only where one is set (the default of the
        // 4x4x4 and bigger, or --max-time and -b)
        let clock = if options.time_budget.is_some() {
            ", which include --max-time"
        } else {
            ""
        };
        eprintln!(
            "found {} of {n} solutions: no more distinct ones turned up within the search limits{clock}",
            listed.len()
        );
    }
    if args.json {
        return print_json(args, start, &listed, kind);
    }
    for (rank, result) in listed.iter().enumerate() {
        let solution = result.solution().expect("a solved result");
        let shown = if args.inverse {
            inverse::moves(solution)
        } else {
            solution.clone()
        };
        println!(
            "{}\t{}\t{}",
            rank + 1,
            metrics::join(metrics::count_all(solution, kind)),
            args.notation.write(kind, &shown)
        );
    }
    Ok(())
}

/// `--json`: an array with the solve report of each solution, each followed by
/// the `state` of the start cube.
fn print_json(
    args: &SolveArgs,
    start: &Facelets,
    listed: &[&SolveResult],
    kind: PuzzleKind,
) -> Result<(), String> {
    let format = args
        .json_format
        .as_deref()
        .map(str::parse)
        .transpose()?
        .unwrap_or_default();
    let reports: Vec<String> = listed
        .iter()
        .map(|result| {
            let solution = result.solution().expect("a solved result");
            let json = SolveReport::of(kind, solution, args.notation, None, result.metrics.elapsed)
                .with_phases(&result.stages)
                .with_evidence(result.metrics.evidence)
                .to_json_format(format);
            // the start cube follows the keys of the report, as for a single solution
            let body = json.strip_suffix('}').expect("a JSON object");
            format!("{body},\"state\":{}}}", crate::state_line::json(start))
        })
        .collect();
    println!("[{}]", reports.join(","));
    Ok(())
}

/// The error text of a list whose first solution failed, as a plain solve says it.
fn failed(
    failure: Failure,
    result: &SolveResult,
    limit: Option<Duration>,
    kind: PuzzleKind,
    target: Option<&Facelets>,
) -> String {
    note_limit(failure);
    let message = result.message.clone();
    match (failure, limit) {
        (Failure::InvalidInput, _) if target.is_some() => format!(
            "the start or the target state is impossible{}",
            message.map_or_else(String::new, |m| format!(" ({m})"))
        ),
        (Failure::BudgetExhausted, Some(limit)) => {
            let detail = message
                .filter(|message| !message.contains("time limit"))
                .map_or_else(String::new, |message| format!(" ({message})"));
            format!("{}{detail}", limit_message(limit, kind))
        }
        _ => failure_message(failure, message),
    }
}
