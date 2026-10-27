//! `solve --to TARGET` without a cube: the start is the solved cube.

use std::sync::atomic::{AtomicBool, Ordering};

use clap::{parser::ValueSource, ArgMatches};
use cubacle::notation::{parse_cube, parse_partial};
use cubacle::puzzle::{Facelets, PuzzleKind};

use crate::cli::SolveArgs;
use crate::input::kind_of;

/// The message for a command that got neither a cube nor a scramble.
pub(crate) const NO_CUBE: &str = "no cube given; give a cube state or --scramble (or use --to TARGET to start from the solved cube)";

static SIZE_GIVEN: AtomicBool = AtomicBool::new(false);

/// Remembers whether `--size` was typed (its default is 3 either way).
pub(crate) fn note_size_given(matches: &ArgMatches) {
    let given = matches
        .subcommand_matches("solve")
        .is_some_and(|solve| solve.value_source("size") == Some(ValueSource::CommandLine));
    SIZE_GIVEN.store(given, Ordering::Relaxed);
}

/// The size the target text stands for, when it is a cube state.
fn target_kind(text: &str, args: &SolveArgs) -> Option<PuzzleKind> {
    let scheme = args.input.scheme;
    if let Ok(Some(partial)) = parse_partial(text, scheme) {
        return Some(partial.kind());
    }
    parse_cube(text, scheme).ok().map(|cube| cube.kind())
}

/// The solved cube to start from, when `--to` is given with no cube and no
/// scramble; `None` when the start is given. Prints the note on stderr.
pub(crate) fn of_target(args: &SolveArgs) -> Result<Option<Facelets>, String> {
    let Some(text) = args.to.as_deref() else {
        return Ok(None);
    };
    if args.input.scramble.is_some() || !args.input.cube.is_empty() {
        return Ok(None);
    }
    let size = args.input.size;
    let kind = match target_kind(text, args) {
        Some(kind) => {
            if SIZE_GIVEN.load(Ordering::Relaxed) && kind.size() != size {
                return Err(format!(
                    "--to: the target is a {0}x{0}x{0} but --size is {1}",
                    kind.size(),
                    size
                ));
            }
            kind
        }
        None => kind_of(size)?,
    };
    eprintln!("start: the solved cube (no cube given; routing to the target)");
    Ok(Some(Facelets::solved(kind)))
}
