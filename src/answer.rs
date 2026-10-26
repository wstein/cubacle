//! The solution `solve` prints: the replay check, the counts on standard
//! error and the text or pictures on standard output.

use cubacle::puzzle::{reaches, solves_by_color, Facelets, MoveSequence, PuzzleKind, SolveStage};
use cubacle::ranges::Optimize;

use crate::{metrics_out, phase_dots, stage_phases, steps, SolveArgs};

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
}

/// Checks that the moves reach the target, then prints them as `args` ask.
pub(crate) fn print(solved: &Solved<'_>) -> Result<(), String> {
    let Solved {
        args,
        start,
        facelets,
        solution,
        stages,
        ..
    } = *solved;
    let target = solved.target.clone();
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
    // The engine's own stages where it reports them, else the state-based cut.
    let found =
        if args.verbose || args.pictures.icons || phase_dots::Grouping::of(&args.group).phases {
            stage_phases::of(stages, facelets, solution)
        } else {
            Vec::new()
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
    let grouping = phase_dots::Grouping::of(&args.group);
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
