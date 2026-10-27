//! Reading cubes, scrambles and targets from the command line.

use std::io::Read;

use cubacle::notation::{
    color_scheme_of, named_subset, named_target, parse_cube, parse_expression, parse_partial,
    ColorScheme,
};
use cubacle::puzzle::{Facelets, PuzzleKind, Target};

use crate::cli::{InputArgs, SolveArgs};
use crate::{solved_start, state_check};

pub(crate) fn kind_of(size: usize) -> Result<PuzzleKind, String> {
    PuzzleKind::of_size(size).ok_or_else(|| format!("size must be between 2 and 7, not {size}"))
}

/// The solved cube of the given kind with a move expression applied; errors
/// are the bare reason.
pub(crate) fn expression_cube(text: &str, kind: PuzzleKind) -> Result<Facelets, String> {
    let moves = parse_expression(text, kind).map_err(|error| error.to_string())?;
    Facelets::solved(kind)
        .apply(&moves)
        .map_err(|error| error.to_string())
}

/// The solved cube of the given kind with a scramble applied.
pub(crate) fn scrambled(text: &str, kind: PuzzleKind) -> Result<Facelets, String> {
    expression_cube(text, kind)
        .map_err(|error| format!("scramble is not valid move notation ({error})"))
}

pub(crate) fn read_cube(input: &InputArgs) -> Result<Facelets, String> {
    read_input(input).map(|(cube, _)| cube)
}

/// The cube, and the color scheme its colors were written in when it was
/// given as colors with centers to read it from.
pub(crate) fn read_input(input: &InputArgs) -> Result<(Facelets, Option<ColorScheme>), String> {
    let (cube, scheme) = read_unchecked(input)?;
    state_check::require(&cube)?;
    Ok((cube, scheme))
}

/// Like `read_input`, for commands that judge the state themselves (`canon`
/// accepts any coloring of the six colors, so its centers need not be a
/// rotation of the cube).
pub(crate) fn read_unchecked(input: &InputArgs) -> Result<(Facelets, Option<ColorScheme>), String> {
    if let Some(scramble) = &input.scramble {
        return Ok((scrambled(scramble, kind_of(input.size)?)?, None));
    }
    let text = match input.cube.as_slice() {
        [] => return Err(solved_start::NO_CUBE.into()),
        [single] if single == "-" => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| format!("cannot read stdin: {error}"))?;
            text
        }
        parts => parts.join(" "),
    };
    parse_text(&text, input.scheme, input.size)
}

/// Reads cube text: cube text always wins; only text that is no cube is tried
/// as a move expression for a cube of `size`.
pub(crate) fn parse_text(
    text: &str,
    scheme: Option<ColorScheme>,
    size: usize,
) -> Result<(Facelets, Option<ColorScheme>), String> {
    match parse_cube(text, scheme) {
        Ok(cube) => Ok((cube, scheme.or_else(|| color_scheme_of(text)))),
        Err(error) if text.trim().is_empty() => Err(error.to_string()),
        Err(error) => {
            let expression = kind_of(size).and_then(|kind| expression_cube(text, kind));
            match expression {
                Ok(cube) => Ok((cube, None)),
                Err(reason) => Err(format!(
                    "not a cube ({error}) and not a move expression for a {size}x{size}x{size} ({reason}){}",
                    state_check::count_hint(text).map_or_else(String::new, |hint| format!("\n{hint}"))
                )),
            }
        }
    }
}

/// The state `--to` names, read like the start; a move expression is applied
/// to the solved cube of the start's size. A text with `X` or `?` stickers
/// gives a partial target; every other form is a complete state.
pub(crate) fn read_target(
    text: &str,
    args: &SolveArgs,
    start: &Facelets,
) -> Result<Target, String> {
    // A pattern name is taken only when the whole text is that name.
    if let Some(named) = named_target(text, start.kind()) {
        return named
            .map(|state| Target::complete(&state))
            .map_err(|error| format!("--to: {error}"));
    }
    if let Some(subset) = named_subset(text, start.kind()) {
        return subset.map_err(|error| format!("--to: {error}"));
    }
    if let Some(partial) =
        parse_partial(text, args.input.scheme).map_err(|error| format!("--to: {error}"))?
    {
        if partial.kind() != start.kind() {
            return Err(format!(
                "--to: the start is a {0}x{0}x{0} but the target has {1} stickers, a {2}x{2}x{2}",
                start.kind().size(),
                partial.kind().facelet_count(),
                partial.kind().size()
            ));
        }
        return Ok(partial);
    }
    let (target, _) = parse_text(text, args.input.scheme, start.kind().size())
        .map_err(|error| format!("--to: {error}"))?;
    state_check::require(&target).map_err(|error| format!("--to: {error}"))?;
    Ok(Target::complete(&target))
}
