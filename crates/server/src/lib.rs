//! Stateless HTTP backend for the cubacle solvers.
//!
//! The router is built by [`router`] from an [`AppState`]; `main.rs` only wires
//! configuration, logging and shutdown around it.
//!
//! # Statelessness
//!
//! There are no sessions and no per-client data. The only shared state is the
//! immutable engines (their lookup tables live in process-wide statics, built
//! once by [`AppState::new`]) and the concurrency semaphore, so any number of
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
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::notation::orbit64::Orbit64Error;
use cubacle::notation::{parse_cube, ColourScheme, InputError};
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, Solver};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tower_http::catch_panic::CatchPanicLayer;

/// Longest time budget a solve may be given (10 minutes).
pub const MAX_TIMEOUT_MS: u64 = 600_000;
/// Largest `--max-concurrency`. Stays well below tokio's blocking-thread limit,
/// so a permit holder never waits for a thread and its budget clock starts on time.
pub const MAX_CONCURRENCY: usize = 256;
/// Largest `maxMoves` the 3x3 engine honours (it silently clamps above this).
pub const MAX_MOVES: u8 = 25;

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
}

impl AppState {
    /// Builds the state and prepares both engines (distance tables), so a
    /// router made from it is ready to answer the moment it exists.
    pub fn new(config: Config) -> AppState {
        PocketSolver::new().prepare();
        Min2PhaseSolver::new().prepare();
        AppState::unprepared(config)
    }

    /// The state without table construction (tests that do not solve).
    pub fn unprepared(config: Config) -> AppState {
        AppState {
            permits: Arc::new(Semaphore::new(config.max_concurrency.max(1))),
            config: Arc::new(config),
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

    fn no_engine(size: usize) -> ApiError {
        ApiError::new(
            StatusCode::NOT_IMPLEMENTED,
            "no_engine",
            format!("no engine for the {size}x{size}x{size} yet; only 2x2x2 and 3x3x3 exist"),
        )
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
    Ok(Json(ScrambleBody {
        scramble: cubacle::random_moves(moves)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    }))
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
}

async fn random(params: Result<Query<RandomParams>, QueryRejection>) -> ApiResult<CubeBody> {
    let Query(params) = params.map_err(|e| ApiError::bad_input(e.body_text()))?;
    let size = params.size.unwrap_or(3);
    let facelets = match size {
        3 => cubacle::random_cube(),
        2 => random_pocket(),
        4..=7 => return Err(ApiError::no_engine(size)),
        _ => return Err(ApiError::bad_input("size must be between 2 and 7")),
    };
    let kind = PuzzleKind::of_size(size).expect("2 and 3 are sizes");
    Ok(Json(CubeBody {
        kind: kind_name(kind),
        facelets,
    }))
}

/// A uniformly random 2x2x2: the corner stickers of a uniformly random 3x3x3
/// (corner permutation and orientation are uniform and independent of edges
/// up to the parity coupling, which a 2x2x2 does not have).
fn random_pocket() -> String {
    let full: Vec<char> = cubacle::random_cube().chars().collect();
    let mut out = String::with_capacity(24);
    for face in 0..6 {
        for index in [0, 2, 6, 8] {
            out.push(full[face * 9 + index]);
        }
    }
    out
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConvertRequest {
    cube: String,
    scheme: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SolveRequest {
    cube: String,
    scheme: Option<String>,
    /// 1..=25.
    max_moves: Option<u8>,
    /// 1..=600000, further capped by the server's request timeout.
    budget_ms: Option<u64>,
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

fn read_cube(cube: &str, scheme: Option<&str>) -> Result<Facelets, ApiError> {
    let scheme = scheme
        .map(str::parse::<ColourScheme>)
        .transpose()
        .map_err(|e| ApiError::bad_input(e.to_string()))?;
    parse_cube(cube, scheme).map_err(|e| match e {
        InputError::Orbit64(Orbit64Error::Unsupported(size)) => ApiError::no_engine(size),
        other => ApiError::bad_input(other.to_string()),
    })
}

async fn convert(headers: HeaderMap, body: Result<Bytes, BytesRejection>) -> ApiResult<CubeBody> {
    let request: ConvertRequest = read_body(&headers, body)?;
    let facelets = read_cube(&request.cube, request.scheme.as_deref())?;
    Ok(Json(CubeBody {
        kind: kind_name(facelets.kind()),
        facelets: facelets.to_string(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SolveBody {
    kind: String,
    solution: String,
    length: usize,
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
    let facelets = read_cube(&request.cube, request.scheme.as_deref())?;
    let kind = facelets.kind();
    if !matches!(kind, PuzzleKind::Cube2 | PuzzleKind::Cube3) {
        return Err(ApiError::no_engine(kind.size()));
    }

    let mut options = SolveOptions::default();
    if let Some(moves) = request.max_moves {
        if !(1..=MAX_MOVES).contains(&moves) {
            return Err(ApiError::bad_input(format!(
                "maxMoves must be between 1 and {MAX_MOVES}"
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

    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit; // released when the solve finishes
        match kind {
            PuzzleKind::Cube2 => PocketSolver::new().solve(&facelets, &options),
            _ => {
                let result = Min2PhaseSolver::new().solve(&facelets, &options);
                debug_assert!(result.solution().is_none_or(|s| {
                    cubacle::apply_moves(&facelets.to_string(), &s.to_string())
                        == Some(Facelets::solved(PuzzleKind::Cube3).to_string())
                }));
                result
            }
        }
    })
    .await
    .map_err(|e| {
        tracing::error!("solver task failed: {e}");
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
        Ok(solution) => Ok(Json(SolveBody {
            kind: kind_name(kind),
            solution: solution.to_string(),
            length: solution.len(),
            elapsed_micros: u64::try_from(result.metrics.elapsed.as_micros()).unwrap_or(u64::MAX),
        })),
        Err(Failure::InvalidInput) => Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "impossible_cube",
            detail("the cube state is impossible"),
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
