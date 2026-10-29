//! The bytes of the `/v1/solve` body. `tests/solve_json_cli_test.rs` of the
//! root crate compares `cubacle solve --json` with the same file
//! (`elapsedMicros` masked to 0), for the equivalent request:
//! `--fast --max-moves 21` is `{"maxMoves":21}`.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use tower::ServiceExt;

const BODY: &str = include_str!("../../../tests/solve_json_body.txt");

/// Replaces the number after `"elapsedMicros":` by 0.
fn masked(body: &str) -> String {
    let key = "\"elapsedMicros\":";
    let at = body.find(key).expect("elapsedMicros") + key.len();
    let digits = body[at..].bytes().take_while(u8::is_ascii_digit).count();
    format!("{}0{}", &body[..at], &body[at + digits..])
}

#[tokio::test]
async fn the_solve_body_is_the_shared_literal() {
    // never the user's real table cache
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(dir));
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    let app = router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ));
    let request = Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"scramble":"R U","maxMoves":21}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(masked(&body), BODY.trim_end());
}

#[tokio::test]
async fn the_solve_body_of_a_staged_engine_is_the_shared_literal() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(dir));
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    let app = router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ));
    let request = Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"scramble":"R U Rw Uw","size":4}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(
        masked(&body),
        include_str!("../../../tests/solve_json_body_four.txt").trim_end()
    );
}
