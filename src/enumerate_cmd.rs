//! `solve --all`, `--count N` and `--depth D`: list solutions instead of one.
//!
//! The solutions stream to standard output as they are found, one per line,
//! shortest first; the length of the group that follows and a summary go to
//! standard error.

use std::io::Write;
use std::time::Duration;

use cubacle::engines::restricted::{
    EnumerateEnd, EnumerateRequest, SubsetSolver, DEFAULT_TABLE_ENTRIES,
};
use cubacle::notation::Notation;
use cubacle::puzzle::{reaches, Alphabet, Facelets, PuzzleKind, SolveOptions};

use crate::restricted_cmd::failure_text;
use crate::SolveArgs;

/// Whether the command asks for a list.
pub fn wanted(args: &SolveArgs) -> bool {
    args.all || args.count.is_some()
}

/// Rejects the options that only make sense with a list.
pub fn check(args: &SolveArgs, kind: PuzzleKind) -> Result<(), String> {
    if !wanted(args) {
        if args.depth.is_some() {
            return Err("--depth needs --all or --count".into());
        }
        if args.raw {
            return Err("--raw needs --all or --count".into());
        }
        return Ok(());
    }
    if kind != PuzzleKind::Cube3 {
        return Err(format!(
            "--all and --count list solutions of the 3x3x3, not of the {0}x{0}x{0}",
            kind.size()
        ));
    }
    if args.count == Some(0) {
        return Err("--count needs at least 1".into());
    }
    Ok(())
}

/// Lists the solutions of `facelets` (the cube left after the target, which
/// is `start` without one) within `alphabet`, the face turns when `None`.
pub fn run(
    args: &SolveArgs,
    start: &Facelets,
    facelets: &Facelets,
    target: Option<&Facelets>,
    alphabet: Option<Alphabet>,
) -> Result<(), String> {
    let kind = facelets.kind();
    let alphabet = alphabet.unwrap_or_else(|| Alphabet::face_turns(kind));
    let entries = (args.max_memory.saturating_mul(1 << 20) / 70).min(DEFAULT_TABLE_ENTRIES);
    let solver = SubsetSolver::new(alphabet)
        .map_err(|error| format!("--moves: {error}"))?
        .with_table_entries(entries);
    let mut options = SolveOptions::default();
    if let Some(moves) = args.max_moves {
        options = options.with_max_moves(moves);
    }
    if let Some(millis) = args.budget_ms {
        options = options.with_time_budget(Duration::from_millis(millis));
    }
    let request = EnumerateRequest {
        longest: args.depth,
        count: args.count,
        raw: args.raw,
    };
    let wanted_state = target
        .cloned()
        .unwrap_or_else(|| Facelets::solved(start.kind()));
    let notation: Notation = args.notation;
    let stdout = std::io::stdout();
    let mut length = None;
    let mut spread = crate::metrics_out::Spread::new(kind);
    let mut written = std::io::Result::Ok(());
    let listing = solver
        .enumerate(
            facelets,
            &request,
            &options,
            &mut |solution| {
                if !reaches(start, solution, &wanted_state) {
                    written = Err(std::io::Error::other(
                        "a listed solution does not solve the cube",
                    ));
                    return false;
                }
                if length != Some(solution.len()) {
                    length = Some(solution.len());
                    eprintln!("moves {}", solution.len());
                }
                spread.add(solution);
                let mut out = stdout.lock();
                written =
                    writeln!(out, "{}", notation.write(kind, solution)).and_then(|()| out.flush());
                written.is_ok()
            },
            &|| false,
        )
        .map_err(|failed| match failed.outcome {
            Err(failure) => failure_text(failure, failed.message),
            Ok(_) => "internal error".to_owned(),
        })?;
    written.map_err(|error| format!("cannot write the solutions: {error}"))?;
    if let Some(line) = spread.line() {
        eprintln!("{line}");
    }
    eprintln!(
        "solutions {} (shortest {} moves; {} nodes)",
        listing.found, listing.shortest, listing.nodes
    );
    match listing.end {
        EnumerateEnd::Complete => Ok(()),
        EnumerateEnd::CountReached => {
            eprintln!("stopped after {} solutions", listing.found);
            Ok(())
        }
        EnumerateEnd::TimedOut => Err(format!(
            "the time budget ran out after {} solutions: the list is incomplete",
            listing.found
        )),
        EnumerateEnd::Cancelled => Err("stopped".to_owned()),
    }
}
