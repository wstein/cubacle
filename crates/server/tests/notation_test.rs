//! The `notation` field (`wca` or `sign`) of `/v1/scramble` and `/v1/solve`.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_moves;
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> Router {
    // never the user's real table cache
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(Some(dir));
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ))
}

async fn call(request: Request<Body>) -> (StatusCode, Value) {
    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn solve(body: Value) -> Request<Body> {
    Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn solve_writes_sign_when_asked_and_wca_by_default() {
    let kind = PuzzleKind::Cube4;
    let scramble = parse_moves("R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l").unwrap();
    let cube = Facelets::solved(kind).apply(&scramble).unwrap();
    let cube_text = cube.to_string();

    let (status, wca) = call(solve(json!({"cube": cube_text}))).await;
    assert_eq!(status, StatusCode::OK, "{wca}");
    let (status, sign) = call(solve(json!({"cube": cube_text, "notation": "sign"}))).await;
    assert_eq!(status, StatusCode::OK, "{sign}");
    let (_, explicit) = call(solve(json!({"cube": cube_text, "notation": "wca"}))).await;
    assert_eq!(wca["solution"], explicit["solution"]);

    let sign_text = sign["solution"].as_str().unwrap();
    assert!(!sign_text.contains('w'), "{sign_text}");
    let moves = parse_moves(sign_text).unwrap();
    assert_eq!(
        moves,
        parse_moves(wca["solution"].as_str().unwrap()).unwrap()
    );
    assert_eq!(sign["length"], wca["length"]);
    assert!(cube.apply(&moves).unwrap().is_solved());
}

#[tokio::test]
async fn scramble_writes_the_requested_notation() {
    let request = |query: &str| Request::get(format!("/v1/scramble{query}")).body(Body::empty());
    let (status, body) = call(request("?size=7&moves=50&notation=sign").unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let text = body["scramble"].as_str().unwrap();
    assert!(!text.contains('w'));
    assert_eq!(parse_moves(text).unwrap().len(), 50);
    let (status, _) = call(request("?notation=wca").unwrap()).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn an_unknown_notation_is_a_400() {
    let cube = Facelets::solved(PuzzleKind::Cube3).to_string();
    for notation in ["ruwix", "SIGN", ""] {
        let (status, body) = call(solve(json!({"cube": cube, "notation": notation}))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{notation}");
        assert_eq!(body["error"]["code"], "bad_input");
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("wca or sign"));
    }
    let (status, _) = call(
        Request::get("/v1/scramble?notation=x")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
