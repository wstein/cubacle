//! Stateless HTTP backend for the cubacle solvers.
//!
//! The router is built by [`router`] from an [`AppState`]; `main.rs` only wires
//! configuration, logging and shutdown around it.
//!
//! # Statelessness
//!
//! There are no sessions and no per-client data. The only shared state is the
//! immutable engines (their lookup tables are built once by [`AppState::new`]
//! and shared by every clone of the state) and the concurrency semaphore, so any number of
//! instances can run behind a load balancer.
//!
//! # Compute isolation
//!
//! Solving runs on `tokio::task::spawn_blocking` threads, never on the async
//! worker threads. A [`Semaphore`] with `max_concurrency` permits bounds how
//! many solves run at once; a request that cannot take a permit immediately
//! is answered with 429 instead of being queued, so memory and latency stay
//! bounded under overload. This was preferred over the crate's own pool
//! because that pool is sized for one parallel 3x3 search; here the unit of
//! work is a whole request and the semaphore gives explicit back-pressure.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, QueryRejection};
use axum::extract::{DefaultBodyLimit, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use cubacle::centres;
use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::generate;

use cubacle::notation::orbit64;
use cubacle::notation::{parse_cube, parse_expression, ColourScheme, Notation};
use cubacle::puzzle::{
    reaches, relative_state, Facelets, Failure, PuzzleKind, RelativeError, SolveOptions, Solver,
};
use cubacle::ranges::{compress, write as write_ranges, Optimise};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tower_http::catch_panic::CatchPanicLayer;

mod counts;
mod input;
use counts::Counts;
use input::{read_input, read_target};

/// Longest time budget a solve may be given (10 minutes).
pub const MAX_TIMEOUT_MS: u64 = 600_000;
/// Largest `--max-concurrency`. Stays well below tokio's blocking-thread limit,
/// so a permit holder never waits for a thread and its budget clock starts on time.
pub const MAX_CONCURRENCY: usize = 256;
/// Largest `minProbes` (the budget bounds the time anyway).
const MAX_MIN_PROBES: u64 = 100_000_000;
/// Largest `maxMoves` the 3x3 engine honours (it silently clamps above this).
pub const MAX_MOVES: u8 = 25;
/// Largest `maxMoves` for the 5x5x5 (a byte; its solutions have about 73 moves).
pub const MAX_MOVES_5X5: u8 = u8::MAX;
/// Largest `maxMoves` for the 4x4x4 (a byte; its solutions have about 48 moves).
pub const MAX_MOVES_4X4: u8 = u8::MAX;

/// Runtime limits of the server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Solves allowed to run at the same time.
    pub max_concurrency: usize,
    /// Upper bound for the time one solve may take; also the default budget.
    pub request_timeout: Duration,
    /// Largest accepted request body.
    pub max_body_bytes: usize,
    /// How long a client may take to send the request head (slowloris guard).
    pub header_read_timeout: Duration,
    /// Extra time, beyond `request_timeout`, that in-flight requests get
    /// after a shutdown signal before connections are dropped.
    pub shutdown_grace: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_concurrency: std::thread::available_parallelism().map_or(1, |n| n.get()),
            request_timeout: Duration::from_secs(30),
            max_body_bytes: 16 * 1024,
            header_read_timeout: Duration::from_secs(10),
            shutdown_grace: Duration::from_secs(5),
        }
    }
}

/// Everything the handlers share. Cloning is cheap.
#[derive(Clone)]
pub struct AppState {
    permits: Arc<Semaphore>,
    config: Arc<Config>,
    /// The 3x3x3 engine; clones share its tables.
    three: Min2PhaseSolver,
    /// The 5x5x5 engine, finishing with a clone of `three`.
    five: FiveSolver,
    /// The 4x4x4 engine, finishing with a clone of `three`; one thread per
    /// solve, so concurrent requests do not compete for cores.
    four: FourSolver,
    /// The 6x6x6 engine, built on clones of the others.
    six: Arc<DwaltonSolver>,
    /// The 7x7x7 engine, likewise.
    seven: Arc<DwaltonSolver>,
}

/// The engine for a big cube over clones of the engines it finishes with.
fn big_of(
    kind: PuzzleKind,
    three: &Min2PhaseSolver,
    four: &FourSolver,
    five: &FiveSolver,
) -> Arc<DwaltonSolver> {
    Arc::new(DwaltonSolver::new(
        kind,
        Arc::new(three.clone()),
        Arc::new(four.clone()),
        Arc::new(five.clone()),
    ))
}

impl AppState {
    /// Builds the state and prepares all engines (tables), so a
    /// router made from it is ready to answer the moment it exists. The 3x3x3
    /// engine is the process-wide default instance.
    pub fn new(config: Config) -> AppState {
        AppState::with_solver(config, Min2PhaseSolver::shared())
    }

    /// Like [`AppState::new`], with the 3x3x3 engine injected.
    pub fn with_solver(config: Config, three: Min2PhaseSolver) -> AppState {
        let five = FiveSolver::new(Arc::new(three.clone()));
        let four = FourSolver::new(Arc::new(three.clone())).with_threads(1);
        AppState::with_engines(config, three, five, four)
    }

    /// Like [`AppState::with_solver`], with the 5x5x5 and 4x4x4 engines injected
    /// too (a clone shares its tables, so tests can build them once and keep
    /// the 5x5x5 cache directory private).
    pub fn with_engines(
        config: Config,
        three: Min2PhaseSolver,
        five: FiveSolver,
        four: FourSolver,
    ) -> AppState {
        PocketSolver::new().prepare();
        let state = AppState {
            permits: Arc::new(Semaphore::new(config.max_concurrency.max(1))),
            config: Arc::new(config),
            six: big_of(PuzzleKind::Cube6, &three, &four, &five),
            seven: big_of(PuzzleKind::Cube7, &three, &four, &five),
            four,
            five,
            three,
        };
        state.three.prepare();
        state.five.prepare();
        state.four.prepare();
        state
    }

    /// The state without table construction (tests that do not solve).
    pub fn unprepared(config: Config) -> AppState {
        AppState::assemble(config, Min2PhaseSolver::shared())
    }

    fn assemble(config: Config, three: Min2PhaseSolver) -> AppState {
        let five = FiveSolver::new(Arc::new(three.clone()));
        let four = FourSolver::new(Arc::new(three.clone())).with_threads(1);
        AppState {
            permits: Arc::new(Semaphore::new(config.max_concurrency.max(1))),
            config: Arc::new(config),
            six: big_of(PuzzleKind::Cube6, &three, &four, &five),
            seven: big_of(PuzzleKind::Cube7, &three, &four, &five),
            five,
            four,
            three,
        }
    }

    /// The semaphore guarding the compute pool, exposed so tests can saturate it.
    pub fn permits(&self) -> &Arc<Semaphore> {
        &self.permits
    }
}

/// Builds the router with all endpoints.
pub fn router(state: AppState) -> Router {
    let routes = Router::new()
        .route("/health", get(health))
        .route("/v1/scramble", get(scramble))
        .route("/v1/random", get(random))
        .route("/v1/convert", post(convert))
        .route("/v1/solve", post(solve));
    finish(routes, &state).with_state(state)
}

/// Adds the fallbacks and the shared layers (body limit, panic catching,
/// request logging) to a set of routes. Public so tests can add routes.
pub fn finish(routes: Router<AppState>, state: &AppState) -> Router<AppState> {
    routes
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(DefaultBodyLimit::max(state.config.max_body_bytes))
        .layer(CatchPanicLayer::custom(|_panic| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "the server failed unexpectedly",
            )
            .into_response()
        }))
        .layer(middleware::from_fn(log_request))
}

/// Logs method, path, status and latency; never the body. The path is logged
/// with `Debug` formatting so control characters are escaped.
async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    let status = response.status();
    let micros = started.elapsed().as_micros();
    if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::GATEWAY_TIMEOUT {
        tracing::warn!(%method, path = ?path, status = status.as_u16(), micros, "request");
    } else {
        tracing::info!(%method, path = ?path, status = status.as_u16(), micros, "request");
    }
    response
}

/// A JSON error with its HTTP status.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    retry_after: Option<u32>,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> ApiError {
        ApiError {
            status,
            code,
            message: message.into(),
            retry_after: None,
        }
    }

    fn bad_input(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::BAD_REQUEST, "bad_input", message)
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorDetail<'a>,
}

#[derive(Serialize)]
struct ErrorDetail<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: &self.message,
            },
        };
        let mut response = (self.status, Json(body)).into_response();
        if let Some(seconds) = self.retry_after {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

fn kind_name(kind: PuzzleKind) -> String {
    let size = kind.size();
    format!("{size}x{size}x{size}")
}

async fn method_not_allowed(method: Method) -> ApiError {
    ApiError::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method_not_allowed",
        format!("{method} is not allowed on this endpoint"),
    )
}

async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such endpoint")
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

/// Always ok: a router only exists after [`AppState::new`] prepared the engines.
async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrambleParams {
    moves: Option<u16>,
    size: Option<usize>,
    /// `wca` (default) or `sign`.
    notation: Option<String>,
}

#[derive(Serialize)]
struct ScrambleBody {
    scramble: String,
}

async fn scramble(
    params: Result<Query<ScrambleParams>, QueryRejection>,
) -> ApiResult<ScrambleBody> {
    let Query(params) = params.map_err(|e| ApiError::bad_input(e.body_text()))?;
    let moves = params.moves.unwrap_or(25);
    if !(1..=200).contains(&moves) {
        return Err(ApiError::bad_input("moves must be between 1 and 200"));
    }
    let notation = read_notation(params.notation.as_deref())?;
    let kind = engine_kind(params.size.unwrap_or(3))?;
    Ok(Json(ScrambleBody {
        scramble: notation.write(kind, &generate::scramble(kind, usize::from(moves))),
    }))
}

/// The `notation` field of a request: `wca` when absent.
fn read_notation(text: Option<&str>) -> Result<Notation, ApiError> {
    text.map_or(Ok(Notation::Wca), str::parse)
        .map_err(ApiError::bad_input)
}

/// The cube size of a request, if an engine (and so a generator) exists for it.
pub(crate) fn engine_kind(size: usize) -> Result<PuzzleKind, ApiError> {
    match size {
        2..=7 => Ok(PuzzleKind::of_size(size).expect("2 to 7 are sizes")),
        _ => Err(ApiError::bad_input("size must be between 2 and 7")),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RandomParams {
    size: Option<usize>,
}

#[derive(Serialize)]
struct CubeBody {
    kind: String,
    facelets: String,
    /// The Orbit64 state token; absent when the stickers are not a cube.
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

impl CubeBody {
    fn of(facelets: &Facelets) -> CubeBody {
        CubeBody {
            kind: kind_name(facelets.kind()),
            facelets: facelets.to_string(),
            token: orbit64::encode(facelets).ok(),
        }
    }
}

async fn random(params: Result<Query<RandomParams>, QueryRejection>) -> ApiResult<CubeBody> {
    let Query(params) = params.map_err(|e| ApiError::bad_input(e.body_text()))?;
    let kind = engine_kind(params.size.unwrap_or(3))?;
    let facelets = generate::random_state(kind).expect("the 2x2x2 to 7x7x7 have generators");
    Ok(Json(CubeBody::of(&facelets)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConvertRequest {
    cube: Option<String>,
    /// Moves in WCA, SiGN or SSE notation instead of `cube`.
    scramble: Option<String>,
    /// Cube size of a `scramble`, 2 to 7 (default 3).
    size: Option<usize>,
    scheme: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SolveRequest {
    cube: Option<String>,
    /// Moves in WCA, SiGN or SSE notation instead of `cube`.
    scramble: Option<String>,
    /// Cube size of a `scramble`, 2 to 7 (default 3).
    size: Option<usize>,
    scheme: Option<String>,
    /// 1..=25, or 1..=255 for the 5x5x5.
    max_moves: Option<u8>,
    /// 1..=600000, further capped by the server's request timeout.
    budget_ms: Option<u64>,
    /// 3x3x3, 4x4x4, 5x5x5: make at least this many probes in the 3x3x3 search. 1..=MAX_MIN_PROBES.
    min_probes: Option<u64>,
    /// 3x3x3 only: keep improving until `budgetMs` is spent (required).
    improve: Option<bool>,
    /// Allow single inner layers (`2R`) in 4x4x4 and 5x5x5 answers.
    #[serde(default)]
    slices: bool,
    /// How the solution is written: `wca` (default) or `sign`.
    notation: Option<String>,
    /// Post-solve compression: `ranges` also returns the solution with layer
    /// ranges (`2-3r`) as `compressed` and its move count as `compressedLength`.
    optimise: Option<String>,
    /// Solve to this state instead of to the solved cube: a cube in any
    /// notation `cube` takes, else a move expression applied to the solved
    /// cube of the start's size, or one of the names `superflip`,
    /// `checkerboard`, `cube-in-cube` (3x3x3).
    target: Option<String>,
}

/// Requires `Content-Type: application/json`. Without this a web page could
/// send a CORS "simple request" (`text/plain`, no preflight) and make a local
/// server burn CPU on its behalf.
fn require_json(headers: &HeaderMap) -> Result<(), ApiError> {
    let essence = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(|v| v.trim().to_ascii_lowercase());
    if essence.as_deref() == Some("application/json") {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "send the body with Content-Type: application/json",
        ))
    }
}

fn read_body<T: serde::de::DeserializeOwned>(
    headers: &HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<T, ApiError> {
    require_json(headers)?;
    let bytes = body.map_err(|e| {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "request body is too large",
            )
        } else {
            ApiError::bad_input(e.body_text())
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ApiError::bad_input(format!("bad JSON: {e}")))
}

pub(crate) fn read_cube(cube: &str, scheme: Option<&str>) -> Result<Facelets, ApiError> {
    let scheme = scheme
        .map(str::parse::<ColourScheme>)
        .transpose()
        .map_err(|e| ApiError::bad_input(e.to_string()))?;
    parse_cube(cube, scheme).map_err(|e| ApiError::bad_input(e.to_string()))
}

async fn convert(headers: HeaderMap, body: Result<Bytes, BytesRejection>) -> ApiResult<CubeBody> {
    let request: ConvertRequest = read_body(&headers, body)?;
    let facelets = read_input(request.cube, request.scramble, request.size, request.scheme).await?;
    Ok(Json(CubeBody::of(&facelets)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SolveBody {
    kind: String,
    solution: String,
    length: usize,
    metrics: Counts,
    #[serde(skip_serializing_if = "Option::is_none")]
    compressed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compressed_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compressed_metrics: Option<Counts>,
    elapsed_micros: u64,
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
async fn solve(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> ApiResult<SolveBody> {
    let request: SolveRequest = read_body(&headers, body)?;
    let notation = read_notation(request.notation.as_deref())?;
    let optimise: Option<Optimise> = request
        .optimise
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
    // A target whose centres differ in colour on one face is reached by a
    // centre fix-up, planned with the engine inside the solve below.
    let mixed = target.as_ref().is_some_and(|t| !centres::uniform(t));
    let facelets = match &target {
        Some(_) if mixed => start.clone(),
        Some(target) => relative_state(&start, target).map_err(|e| match e {
            RelativeError::SizeMismatch { .. } => ApiError::bad_input(e.to_string()),
            RelativeError::ImpossiblePiece { .. } => ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "impossible_cube",
                e.to_string(),
            ),
            // targets with mixed centres take the centre fix-up and never get here
            RelativeError::CentresMoved => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            }
        })?,
        None => start.clone(),
    };
    let wanted = target.unwrap_or_else(|| Facelets::solved(kind));
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
                centres::solve_to(engine, &start, &wanted, &options, hint.as_ref())
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
        // Defence in depth, in release builds too: never serve a wrong answer.
        let solves = result
            .solution()
            .is_none_or(|s| reaches(&start, s, &wanted));
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
            let packed = optimise.map(|_| compress(kind, solution));
            Ok(Json(SolveBody {
                kind: kind_name(kind),
                solution: notation.write(kind, solution),
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

/// Serves `app` on `listener` until `shutdown` resolves, then drains.
///
/// `axum::serve` has no header-read timeout, so one half-open connection
/// could keep a graceful shutdown waiting forever. This loop uses hyper's
/// HTTP/1 server directly (hyper and hyper-util are already axum's
/// dependencies, so nothing new is compiled) to get a header-read timeout,
/// and bounds the drain by `request_timeout + shutdown_grace`; connections
/// still open after that are dropped.
///
/// Returns `true` if everything drained in time.
pub async fn serve(
    listener: tokio::net::TcpListener,
    app: Router,
    config: &Config,
    shutdown: impl std::future::Future<Output = ()>,
) -> bool {
    use hyper_util::rt::{TokioIo, TokioTimer};
    use hyper_util::server::graceful::GracefulShutdown;
    use hyper_util::service::TowerToHyperService;

    let graceful = GracefulShutdown::new();
    let header_timeout = config.header_read_timeout;
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let stream = match accepted {
                    Ok((stream, _)) => stream,
                    Err(error) => {
                        tracing::warn!("accept failed: {error}");
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        continue;
                    }
                };
                let service = TowerToHyperService::new(app.clone());
                let mut http = hyper::server::conn::http1::Builder::new();
                http.timer(TokioTimer::new()).header_read_timeout(header_timeout);
                let connection = http.serve_connection(TokioIo::new(stream), service);
                let connection = graceful.watch(connection);
                tokio::spawn(async move {
                    let _ = connection.await;
                });
            }
            () = &mut shutdown => break,
        }
    }
    drop(listener);
    let deadline = config.request_timeout + config.shutdown_grace;
    tokio::time::timeout(deadline, graceful.shutdown())
        .await
        .is_ok()
}
