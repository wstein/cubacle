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
//! worker threads. A [`Semaphore`](tokio::sync::Semaphore) with `max_concurrency` permits bounds how
//! many solves run at once; a request that cannot take a permit immediately
//! is answered with 429 instead of being queued, so memory and latency stay
//! bounded under overload. This was preferred over the crate's own pool
//! because that pool is sized for one parallel 3x3 search; here the unit of
//! work is a whole request and the semaphore gives explicit back-pressure.

use axum::extract::{DefaultBodyLimit, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use serde::Serialize;
use tower_http::catch_panic::CatchPanicLayer;

mod canonical;
mod cube;
mod error;
mod input;
mod request;
mod serve;
mod solve;
mod state;

pub use error::ApiError;
use error::{method_not_allowed, not_found};
pub use serve::serve;
pub use state::{
    AppState, Config, MAX_CONCURRENCY, MAX_MOVES, MAX_MOVES_4X4, MAX_MOVES_5X5, MAX_TIMEOUT_MS,
};

/// Builds the router with all endpoints.
pub fn router(state: AppState) -> Router {
    let routes = Router::new()
        .route("/health", get(health))
        .route("/v1/scramble", get(cube::scramble))
        .route("/v1/random", get(cube::random))
        .route("/v1/convert", post(cube::convert))
        .route("/v1/solve", post(solve::solve));
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

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

/// Always ok: a router only exists after [`AppState::new`] prepared the engines.
async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
