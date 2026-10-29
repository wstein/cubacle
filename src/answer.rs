//! The solution `solve` prints: the replay check, the counts on standard
//! error and the text or pictures on standard output.

use std::time::Duration;

use cubacle::puzzle::{
    reaches, solves_by_color, Facelets, MoveSequence, PuzzleKind, SolveEvidence, SolveStage,
};
use cubacle::ranges::Optimize;
use cubacle::report::SolveReport;

use crate::cli::SolveArgs;
use crate::{inverse, metrics_out, pattern_hint, phase_dots, stage_phases, steps};

/// A solution and what it was found for.
pub(crate) struct Solved<'a> {
    pub args: &'a SolveArgs,
    pub start: &'a Facelets,
    /// The cube the engine solved: the start, or what is left of it after the
    /// target is taken out.
    pub facelets: &'a Facelets,
    /// The state the moves lead to; `None` for the solved cube.
    pub target: Option<Facelets>,
    pub solution: &'a MoveSequence,
    /// The engine's own stages where it reports them.
    pub stages: &'a [SolveStage],
    /// The cube was solved with its colors named after its centers.
    pub oriented: bool,
    /// The line that says a known pattern was adopted.
    pub adopted: Option<String>,
    /// The time the search took, for `--json`.
    pub elapsed: Duration,
    pub evidence: SolveEvidence,
}

/// Checks that the moves reach the target, then prints them as `args` ask.
pub(crate) fn print(solved: &Solved<'_>) -> Result<(), String> {
    let Solved {
        args,
        start,
        facelets,
        solution,
        stages,
        elapsed,
        ..
    } = *solved;
    let mut target = solved.target.clone();
    // Every engine's answer is replayed here, whatever the size.
    let wanted = target
        .clone()
        .unwrap_or_else(|| Facelets::solved(start.kind()));
    let replays = if solved.oriented {
        solves_by_color(start, solution)
    } else {
        reaches(start, solution, &wanted)
    };
    if (start.kind() != PuzzleKind::Cube2 || target.is_some()) && !replays {
        return Err(match target {
            Some(_) => "internal error: solution does not reach the target".into(),
            None => "internal error: solution does not solve the cube".into(),
        });
    }
    eprintln!("moves {}", solution.len());
    eprintln!("{}", metrics_out::line(facelets.kind(), solution));
    if let Some(adopted) = &solved.adopted {
        eprintln!("{adopted}");
    }
    if args.json {
        return print_json(solved, elapsed);
    }
    // With `--inverse` everything below shows the moves that undo the solution,
    // starting from the cube the solution ends in; the counts above stay those
    // of the solution.
    let built;
    let back;
    let (start, solution) = if args.inverse {
        built = start.apply(solution).map_err(|e| e.to_string())?;
        back = inverse::moves(solution);
        (&built, &back)
    } else {
        (start, solution)
    };
    if args.inverse {
        target = Some(solved.start.clone());
    }
    // The engine's own stages where it reports them, else the state-based cut.
    let found = match args.every {
        Some(every) if args.optimize == Some(Optimize::Ranges) => {
            phase_dots::every_packed(facelets.kind(), solution, usize::from(every))
        }
        Some(every) => phase_dots::every(solution.len(), usize::from(every)),
        None if args.verbose
            || args.pictures.icons
            || phase_dots::Grouping::of(&args.group).phases =>
        {
            if args.inverse {
                let phases = stage_phases::of(stages, facelets, solved.solution);
                inverse::phases(&phases, solution.len())
            } else {
                stage_phases::of(stages, facelets, solution)
            }
        }
        None => Vec::new(),
    };
    // With `--icons` on a graphics terminal the phase headings say it.
    let headed = args.pictures.icons && steps::shows_phase_headings(args.pictures.graphics);
    if args.verbose && !headed {
        for phase in &found {
            eprintln!("{}", steps::phase_line(phase));
        }
    }
    if args.pictures.icons {
        let mut options = args.pictures.options(args.input.scheme, args.notation);
        options.target = target;
        options.show_empty = args.verbose;
        let ranges = args.optimize == Some(Optimize::Ranges);
        if ranges {
            // the steps the pictures show, as the text answer reports its own
            let packed = steps::compressed(start, solution, &found);
            eprintln!("compressed moves {}", packed.len());
            eprintln!(
                "compressed {}",
                metrics_out::compressed_line(facelets.kind(), &packed)
            );
        }
        return steps::print_solution(start, solution, &found, &options, ranges);
    }
    let mut grouping = phase_dots::Grouping::of(&args.group);
    grouping.phases |= args.every.is_some();
    let dots = grouping.dots();
    let ranges = args.optimize == Some(Optimize::Ranges);
    let (text, count) = phase_dots::write(
        args.notation,
        facelets.kind(),
        solution,
        &found,
        ranges,
        grouping,
    );
    if ranges {
        eprintln!("compressed moves {count}");
        if !dots {
            let packed = cubacle::ranges::compress(facelets.kind(), solution);
            eprintln!(
                "compressed {}",
                metrics_out::compressed_line(facelets.kind(), &packed)
            );
        }
    }
    println!("{text}");
    Ok(())
}

/// `--json`: the body the REST server sends for an equivalent request, alone on
/// standard output. The compressed counts stay on standard error as in the text.
fn print_json(solved: &Solved<'_>, elapsed: Duration) -> Result<(), String> {
    let args = solved.args;
    let kind = solved.facelets.kind();
    let hint = if args.verbose {
        pattern_hint::start_name(args.config.as_deref(), solved.start)
    } else {
        None
    };
    let report = SolveReport::of(kind, solved.solution, args.notation, args.optimize, elapsed)
        .with_phases(solved.stages)
        .with_pattern_hint(hint)
        .with_evidence(solved.evidence);
    if let Some(compressed) = &report.compressed {
        eprintln!("compressed moves {}", compressed.length);
        let packed = cubacle::ranges::compress(kind, solved.solution);
        eprintln!("compressed {}", metrics_out::compressed_line(kind, &packed));
    }
    println!(
        "{}",
        report.to_json_format(
            args.json_format
                .as_deref()
                .map(str::parse)
                .transpose()?
                .unwrap_or_default()
        )
    );
    Ok(())
}
