//! The cube of a request: given as `cube`, or as a `scramble` of moves in WCA
//! or SiGN notation (layer ranges, groups and commutators included).

use cubacle::notation::{named_target, parse_expression};
use cubacle::puzzle::{Facelets, PuzzleKind};

use crate::{engine_kind, read_cube, ApiError};

/// The cube named by `cube` or by `scramble` (exactly one of them). `size`
/// (default 3) says which cube a scramble is applied to and is refused with
/// `cube`, which carries its own size.
pub(crate) async fn read_input(
    cube: Option<String>,
    scramble: Option<String>,
    size: Option<usize>,
    scheme: Option<String>,
) -> Result<Facelets, ApiError> {
    match (cube, scramble) {
        (Some(_), Some(_)) => Err(ApiError::bad_input("send cube or scramble, not both")),
        (None, None) => Err(ApiError::bad_input("send a cube or a scramble")),
        (Some(cube), None) => {
            if size.is_some() {
                return Err(ApiError::bad_input(
                    "size goes with a scramble; a cube has its size already",
                ));
            }
            read_cube(&cube, scheme.as_deref())
        }
        (None, Some(scramble)) => {
            if scheme.is_some() {
                return Err(ApiError::bad_input(
                    "scheme goes with a cube, not a scramble",
                ));
            }
            let kind = engine_kind(size.unwrap_or(3))?;
            // up to 100000 moves are applied: not on the async threads
            tokio::task::spawn_blocking(move || {
                let moves = parse_expression(&scramble, kind)
                    .map_err(|e| ApiError::bad_input(format!("scramble: {e}")))?;
                Facelets::solved(kind)
                    .apply(&moves)
                    .map_err(|e| ApiError::bad_input(format!("scramble: {e}")))
            })
            .await
            .map_err(|_| ApiError::bad_input("the scramble could not be read"))?
        }
    }
}

/// The target of a solve: a pattern name (`superflip`, `checkerboard`,
/// `cube-in-cube`, only when that is the whole text), else a cube in any
/// notation `cube` takes, else (when the text is no cube) a move expression
/// applied to the solved cube of the start's size. Cube text wins over a
/// move expression, as for the start of the command line.
pub(crate) async fn read_target(
    target: String,
    scheme: Option<String>,
    kind: PuzzleKind,
) -> Result<Facelets, ApiError> {
    if let Some(named) = named_target(&target, kind) {
        return named.map_err(|e| ApiError::bad_input(format!("target: {e}")));
    }
    let not_a_cube = match read_cube(&target, scheme.as_deref()) {
        Ok(cube) => return Ok(cube),
        Err(error) => error,
    };
    if target.trim().is_empty() {
        return Err(ApiError::bad_input(format!(
            "target: {}",
            not_a_cube.message
        )));
    }
    // up to 100000 moves are applied: not on the async threads
    tokio::task::spawn_blocking(move || {
        let moves = parse_expression(&target, kind).map_err(|e| {
            let size = kind.size();
            ApiError::bad_input(format!(
                "target is not a cube ({}) and not a move expression for a {size}x{size}x{size} ({e})",
                not_a_cube.message
            ))
        })?;
        Facelets::solved(kind)
            .apply(&moves)
            .map_err(|e| ApiError::bad_input(format!("target: {e}")))
    })
    .await
    .map_err(|_| ApiError::bad_input("the target could not be read"))?
}
