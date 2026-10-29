//! The `optimize` field of `/v1/solve`: opt-in layer ranges.

mod shared;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_moves;
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> Router {
    let five = shared::five();
    let four = shared::four();
    router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ))
}

async fn call(body: Value) -> (StatusCode, Value) {
    let request = Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn ranges_are_added_only_on_request() {
    let scramble = parse_moves("R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l").unwrap();
    let cube = Facelets::solved(PuzzleKind::Cube4)
        .apply(&scramble)
        .unwrap();
    let cube_text = cube.to_string();

    let (status, plain) = call(json!({"cube": cube_text})).await;
    assert_eq!(status, StatusCode::OK, "{plain}");
    assert!(plain.get("compressed").is_none() && plain.get("compressedLength").is_none());

    let (status, packed) = call(json!({"cube": cube_text, "optimize": "ranges"})).await;
    assert_eq!(status, StatusCode::OK, "{packed}");
    assert_eq!(packed["solution"], plain["solution"]);
    let text = packed["compressed"].as_str().unwrap();
    assert_eq!(
        packed["compressedLength"].as_u64().unwrap() as usize,
        text.split_whitespace().count()
    );
    assert!(packed["compressedLength"].as_u64() <= packed["length"].as_u64());
}

#[tokio::test]
async fn an_unknown_optimization_is_a_400() {
    let cube = Facelets::solved(PuzzleKind::Cube3).to_string();
    let (status, _) = call(json!({"cube": cube, "optimize": "all"})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
