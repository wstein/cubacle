//! The solve endpoint.

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use cubacle::centers;
use cubacle::engines::two::PocketSolver;
use cubacle::notation::parse_expression;
use cubacle::puzzle::{
    oriented_by_centers, reaches, relative_state, solves_by_color, Facelets, Failure, PuzzleKind,
    RelativeError, SolveOptions, Solver,
};
use cubacle::ranges::{compress, write as write_ranges, Optimize};
use serde::{Deserialize, Serialize};

use crate::counts::Counts;
use crate::error::{ApiError, ApiResult};
use crate::input::{read_input, read_target};
use crate::request::*;
use crate::state::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SolveRequest {
    pub(crate) cube: Option<String>,
    /// Moves in WCA, SiGN or SSE notation instead of `cube`.
    pub(crate) scramble: Option<String>,
    /// Cube size of a `scramble`, 2 to 7 (default 3).
    pub(crate) size: Option<usize>,
    pub(crate) scheme: Option<String>,
    /// 1..=25, or 1..=255 for the 5x5x5.
    pub(crate) max_moves: Option<u8>,
    /// 1..=600000, further capped by the server's request timeout.
    pub(crate) budget_ms: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: make at least this many probes in the 3x3x3 search. 1..=MAX_MIN_PROBES.
    pub(crate) min_probes: Option<u64>,
    /// 3x3x3 only: keep improving until `budgetMs` is spent (required).
    pub(crate) improve: Option<bool>,
    /// Allow single inner layers (`2R`) in 4x4x4 and 5x5x5 answers.
    #[serde(default)]
    pub(crate) slices: bool,
    /// How the solution is written: `wca` (default) or `sign`.
    pub(crate) notation: Option<String>,
    /// Post-solve compression: `ranges` also returns the solution with layer
    /// ranges (`2-3r`) as `compressed` and its move count as `compressedLength`.
    pub(crate) optimize: Option<String>,
    /// Solve to this state instead of to the solved cube: a cube in any
    /// notation `cube` takes, else a move expression applied to the solved
    /// cube of the start's size, or one of the names `superflip`,
    /// `checkerboard`, `cube-in-cube` (3x3x3).
    pub(crate) target: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SolveBody {
    pub(crate) kind: String,
    pub(crate) solution: String,
    /// The moves that undo `solution`, in the same notation.
    pub(crate) inverse: String,
    pub(crate) length: usize,
    pub(crate) metrics: Counts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) compressed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) compressed_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) compressed_metrics: Option<Counts>,
    pub(crate) elapsed_micros: u64,
}

/// Failure statuses:
///
/// * `BudgetExhausted` -> 504. 408 means the client was too slow sending its
///   request, which is not the case here; the server could not finish within
///   the time it allotted, which is the gateway-timeout shape. Repeating the
///   same request with the same limits will usually fail the same way and
///   multiplies CPU cost, so clients should retry only with a larger budget
///   or an easier input.
/// * `Exhausted` and `InvalidInput` -> 422: the request was well formed but
///   the cube is impossible or has no solution within `maxMoves`.
pub(crate) async fn solve(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> ApiResult<SolveBody> {
    let request: SolveRequest = read_body(&headers, body)?;
    let notation = read_notation(request.notation.as_deref())?;
    let optimize: Option<Optimize> = request
        .optimize
        .as_deref()
        .map(str::parse)
        .transpose()
        .map_err(ApiError::bad_input)?;
    let scheme = request.scheme.clone();
    let start = read_input(request.cube, request.scramble, request.size, request.scheme).await?;
    let kind = start.kind();
    engine_kind(kind.size())?;
    cubacle::notation::check(&start).map_err(|reason| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "impossible_cube",
            match reason.hint() {
                Some(hint) => format!("the cube state is impossible ({reason}); hint: {hint}"),
                None => format!("the cube state is impossible ({reason})"),
            },
        )
    })?;
    let target_text = request.target.clone();
    let target = match request.target {
        Some(text) => Some(read_target(text, scheme, kind).await?),
        None => None,
    };
    let has_target = target.is_some();
    // The cube to solve: the start itself, or what is left of it after the
    // target is taken out.
    // A target whose centers differ in color on one face is reached by a
    // center fix-up, planned with the engine inside the solve below.
    let mixed = target.as_ref().is_some_and(|t| !centers::uniform(t));
    let facelets = match &target {
        Some(_) if mixed => start.clone(),
        Some(target) => relative_state(&start, target).map_err(|e| match e {
            RelativeError::SizeMismatch { .. } => ApiError::bad_input(e.to_string()),
            RelativeError::ImpossiblePiece { .. } => ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "impossible_cube",
                e.to_string(),
            ),
            // targets with mixed centers take the center fix-up and never get here
            RelativeError::CentersMoved => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            }
        })?,
        None => start.clone(),
    };
    let wanted = target.unwrap_or_else(|| Facelets::solved(kind));
    // A cube whose centers a slice turn moved is solved as the cube with its
    // colors named after its centers; every face ends one color.
    let renamed = if has_target {
        None
    } else {
        oriented_by_centers(&facelets)
    };
    let oriented = renamed.is_some();
    let facelets = renamed.unwrap_or(facelets);
    let mut options = SolveOptions::default();
    if let Some(moves) = request.max_moves {
        let most = match kind {
            PuzzleKind::Cube4 => MAX_MOVES_4X4,
            PuzzleKind::Cube5 | PuzzleKind::Cube6 | PuzzleKind::Cube7 => MAX_MOVES_5X5,
            _ => MAX_MOVES,
        };
        if !(1..=most).contains(&moves) {
            return Err(ApiError::bad_input(format!(
                "maxMoves must be between 1 and {most}"
            )));
        }
        options = options.with_max_moves(moves);
    }
    let budget = match request.budget_ms {
        Some(ms) if !(1..=MAX_TIMEOUT_MS).contains(&ms) => {
            return Err(ApiError::bad_input(format!(
                "budgetMs must be between 1 and {MAX_TIMEOUT_MS}"
            )))
        }
        Some(ms) => Duration::from_millis(ms).min(state.config.request_timeout),
        None => state.config.request_timeout,
    };
    options = options.with_time_budget(budget);
    let improve = request.improve.unwrap_or(false);
    if improve && kind != PuzzleKind::Cube3 {
        return Err(ApiError::bad_input(
            "improve is only available for the 3x3x3",
        ));
    }
    if request.min_probes.is_some()
        && !matches!(
            kind,
            PuzzleKind::Cube3 | PuzzleKind::Cube4 | PuzzleKind::Cube5
        )
    {
        return Err(ApiError::bad_input(
            "minProbes is only available for the 3x3x3, 4x4x4 and 5x5x5",
        ));
    }
    if improve && request.budget_ms.is_none() {
        return Err(ApiError::bad_input("improve needs budgetMs"));
    }
    if improve && request.min_probes.is_some() {
        return Err(ApiError::bad_input(
            "improve and minProbes cannot be combined",
        ));
    }
    if let Some(probes) = request.min_probes {
        if !(1..=MAX_MIN_PROBES).contains(&probes) {
            return Err(ApiError::bad_input(format!(
                "minProbes must be between 1 and {MAX_MIN_PROBES}"
            )));
        }
        options = options.with_min_probes(probes);
    }
    options = options.with_time_budget(budget).with_slices(request.slices);

    // One semaphore serves both sizes; separate pools per size are a possible
    // later refinement.
    let permit = state.permits.clone().try_acquire_owned().map_err(|_| {
        let mut error = ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "busy",
            "all solver slots are in use; retry shortly",
        );
        error.retry_after = Some(1);
        error
    })?;

    let (three, four, five) = (state.three.clone(), state.four.clone(), state.five.clone());
    let (six, seven) = (state.six.clone(), state.seven.clone());
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit; // released when the solve finishes
        let run = |engine: &dyn Solver| {
            if mixed {
                // the moves of the target, when given as moves, bound the fix-up
                let hint = target_text
                    .as_deref()
                    .and_then(|text| parse_expression(text, kind).ok());
                centers::solve_to(engine, &start, &wanted, &options, hint.as_ref())
            } else {
                engine.solve(&facelets, &options)
            }
        };
        let result = match kind {
            PuzzleKind::Cube2 => return Some(PocketSolver::new().solve(&facelets, &options)),
            PuzzleKind::Cube4 => run(&four),
            PuzzleKind::Cube5 => run(&five),
            PuzzleKind::Cube6 => run(six.as_ref()),
            PuzzleKind::Cube7 => run(seven.as_ref()),
            _ if improve => three.solve_with_minimum_time(&facelets, &options, budget),
            _ => three.solve(&facelets, &options),
        };
        // Defense in depth, in release builds too: never serve a wrong answer.
        let solves = result.solution().is_none_or(|s| {
            if oriented {
                solves_by_color(&start, s)
            } else {
                reaches(&start, s, &wanted)
            }
        });
        solves.then_some(result)
    })
    .await
    .map_err(|e| {
        tracing::error!("solver task failed: {e}");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "the solver failed unexpectedly",
        )
    })?
    .ok_or_else(|| {
        tracing::error!("the solver returned a solution that does not solve the cube");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "the solver failed unexpectedly",
        )
    })?;

    let detail = |fallback: &str| {
        result
            .message
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    };
    match &result.outcome {
        Ok(solution) => {
            let packed = optimize.map(|_| compress(kind, solution));
            Ok(Json(SolveBody {
                kind: kind_name(kind),
                solution: notation.write(kind, solution),
                inverse: notation.write(kind, &solution.inverse()),
                length: solution.len(),
                metrics: Counts::of(solution, kind),
                compressed: packed
                    .as_ref()
                    .map(|moves| write_ranges(notation, kind, moves)),
                compressed_length: packed.as_ref().map(Vec::len),
                compressed_metrics: packed.as_ref().map(|moves| Counts::of_ranges(moves, kind)),
                elapsed_micros: u64::try_from(result.metrics.elapsed.as_micros())
                    .unwrap_or(u64::MAX),
            }))
        }
        Err(Failure::InvalidInput) => Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "impossible_cube",
            detail(if has_target {
                "the start or the target state is impossible"
            } else {
                "the cube state is impossible"
            }),
        )),
        Err(Failure::CentersOffFace) => Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "centers_off_face",
            "the face centers are not on their own faces (a middle-slice turn moved them); \
             this solver needs the centers of each face in place",
        )),
        Err(Failure::BudgetExhausted) => Err(ApiError::new(
            StatusCode::GATEWAY_TIMEOUT,
            "budget_exhausted",
            detail("the time budget ran out"),
        )),
        Err(Failure::Exhausted) => Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "no_solution_within_limit",
            detail("no solution within the move limit"),
        )),
        Err(Failure::Internal) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            detail("internal error"),
        )),
    }
}
