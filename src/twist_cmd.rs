//! The center twist of a 3x3x3 on the command line: `--center-twist`,
//! `--twist-target` and `--upright-centers`.
//!
//! The target is a state of the stickers and a condition on the twist of each
//! of the six centers (see `cubacle::puzzle::CenterTwists` for the convention).
//! The start twist is read from `--center-twist`, or derived from the scramble
//! (face turns, wide turns, slices and rotations; a cube that ends with its
//! centers off their faces is turned as a whole into the frame where they are
//! home, which costs no move). The search itself is the exact search of
//! `engines::three`.

use cubacle::notation::{parse_cube, parse_expression};
use cubacle::puzzle::{
    parse_twist_masks, rotation_moves, rotation_words, CenterTwists, Facelets, MoveSequence,
    PuzzleKind, Target, FREE_TWIST,
};

use crate::cli::{InputArgs, SolveArgs};

/// What the start cube looks like beyond its stickers.
pub(crate) struct Picture {
    /// The twist of the six centers at the start, after the whole-cube rotation
    /// that turned a scramble with moved centers home (it costs no move).
    pub(crate) twist: CenterTwists,
}

/// Whether any of the twist options is given.
pub(crate) fn wanted(args: &SolveArgs) -> bool {
    args.upright_centers || args.twist_target.is_some() || args.center_twist.is_some()
}

/// The move expression the input stands for, when it is one (a `--scramble`,
/// or text that is no cube but a move expression); `None` for a cube state.
pub(crate) fn moves_of_input(input: &InputArgs) -> Result<Option<MoveSequence>, String> {
    let kind = PuzzleKind::of_size(input.size).unwrap_or(PuzzleKind::Cube3);
    let expression = |text: &str| {
        parse_expression(text, kind)
            .map_err(|error| format!("scramble is not valid move notation ({error})"))
    };
    if let Some(scramble) = &input.scramble {
        return expression(scramble).map(Some);
    }
    let text = input.cube.join(" ");
    if text.trim().is_empty() || text.trim() == "-" || parse_cube(&text, input.scheme).is_ok() {
        return Ok(None);
    }
    Ok(parse_expression(&text, kind).ok())
}

/// The cube to solve, the target with its twist condition and the start twist.
pub(crate) fn prepare(
    args: &SolveArgs,
    start: Facelets,
    target: Option<Target>,
) -> Result<(Facelets, Target, Picture), String> {
    if start.kind() != PuzzleKind::Cube3 {
        return Err(format!(
            "--upright-centers, --twist-target and --center-twist work on the 3x3x3 only, not the {0}x{0}x{0}",
            start.kind().size()
        ));
    }
    let masks = if args.upright_centers {
        [1; 6]
    } else if let Some(text) = &args.twist_target {
        parse_twist_masks(text).map_err(|error| format!("--twist-target: {error}"))?
    } else {
        FREE_TWIST
    };
    if masks == FREE_TWIST {
        return Err("--center-twist needs --upright-centers or --twist-target: without a condition on the twists it has no effect".into());
    }
    let target = target
        .unwrap_or_else(|| Target::complete(&Facelets::solved(PuzzleKind::Cube3)))
        .with_twist(masks)
        .map_err(|error| format!("--twist-target: {error}"))?;
    let given = args
        .center_twist
        .as_deref()
        .map(|text| CenterTwists::parse(PuzzleKind::Cube3, text))
        .transpose()
        .map_err(|error| format!("--center-twist: {error}"))?;
    let moves = moves_of_input(&args.input)?;
    let (twist, frame) = match (given, moves) {
        (Some(_), Some(_)) => {
            return Err("--center-twist describes a cube given as stickers; a scramble derives its own twist (give one of them)".into())
        }
        (Some(twist), None) => (twist, String::new()),
        (None, Some(moves)) => {
            let start_twist = CenterTwists::untwisted(PuzzleKind::Cube3).expect("odd size");
            let tracked = start_twist
                .track(&moves)
                .map_err(|error| format!("the twist of the scramble: {error}"))?;
            (tracked.twists, tracked.frame)
        }
        (None, None) => {
            return Err("the cube is given as stickers, which do not say how the pictures of its centers sit: add --center-twist T (six numbers 0 to 3 for U,R,F,D,L,B, like 0,1,2,0,3,0), or give a scramble".into())
        }
    };
    let start = if frame.is_empty() {
        start
    } else {
        eprintln!(
            "note: the scramble leaves the centers off their faces; the cube is turned as a whole ({frame}) first, which costs no move"
        );
        let rotation = rotation_moves(&frame).map_err(|error| error.to_string())?;
        start.apply(&rotation).map_err(|error| error.to_string())?
    };
    eprintln!("center twist at the start: {twist} (U,R,F,D,L,B)");
    Ok((start, target, Picture { twist }))
}

/// Where the cube ends: after the answer and the final whole-cube rotation.
pub(crate) struct End {
    pub(crate) twist: CenterTwists,
    /// The rotation (`x y z` text, empty for none) after the last move.
    pub(crate) rotation: String,
}

/// Replays `solution` from `start` with the twist `picture`, independently of
/// the engine: the stickers through the facelet model, the twist through the
/// center tracker. The end must meet `target` (stickers and twist), after one
/// of the 24 rotations when `any_orientation`.
pub(crate) fn replay(
    start: &Facelets,
    picture: &Picture,
    target: &Target,
    solution: &MoveSequence,
    any_orientation: bool,
) -> Result<End, String> {
    let end = start.apply(solution).map_err(|error| error.to_string())?;
    let tracked = picture
        .twist
        .track(solution)
        .map_err(|error| error.to_string())?;
    if !tracked.frame.is_empty() {
        return Err("internal error: the answer moves the centers".into());
    }
    let words = if any_orientation {
        rotation_words()
    } else {
        vec![String::new()]
    };
    for word in words {
        let rotation = rotation_moves(&word).map_err(|error| error.to_string())?;
        let state = end.apply(&rotation).map_err(|error| error.to_string())?;
        let twist = tracked
            .twists
            .rotated(&word)
            .map_err(|error| error.to_string())?;
        if target.matches(&state) && target.twist_met(&twist) {
            return Ok(End {
                twist,
                rotation: word,
            });
        }
    }
    Err("internal error: solution does not reach the target".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves(text: &str) -> MoveSequence {
        parse_expression(text, PuzzleKind::Cube3).unwrap()
    }

    fn upright_target() -> Target {
        Target::complete(&Facelets::solved(PuzzleKind::Cube3))
            .with_twist([1; 6])
            .unwrap()
    }

    fn picture(twist: [u8; 6]) -> Picture {
        Picture {
            twist: CenterTwists::new(PuzzleKind::Cube3, twist).unwrap(),
        }
    }

    #[test]
    fn the_replay_accepts_a_right_answer_and_names_the_end_twist() {
        // R turns the R picture a quarter, R' turns it back
        let start = Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves("R"))
            .unwrap();
        let end = replay(
            &start,
            &picture([0, 1, 0, 0, 0, 0]),
            &upright_target(),
            &moves("R'"),
            false,
        )
        .unwrap();
        assert_eq!(end.twist.to_string(), "0,0,0,0,0,0");
        assert!(end.rotation.is_empty());
    }

    #[test]
    fn the_replay_refuses_an_answer_that_misses_the_stickers() {
        let start = Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves("R"))
            .unwrap();
        let error = replay(
            &start,
            &picture([0, 1, 0, 0, 0, 0]),
            &upright_target(),
            &moves("U"),
            false,
        )
        .unwrap_err();
        assert!(error.contains("internal error"), "{error}");
    }

    #[test]
    fn the_replay_refuses_an_answer_that_misses_the_twists() {
        // the stickers are solved by R' from R, but the start twist says the R
        // picture sits half turned, so R' leaves it a quarter off
        let start = Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves("R"))
            .unwrap();
        let error = replay(
            &start,
            &picture([0, 2, 0, 0, 0, 0]),
            &upright_target(),
            &moves("R'"),
            false,
        )
        .unwrap_err();
        assert!(error.contains("internal error"), "{error}");
    }
}
