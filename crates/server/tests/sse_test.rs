//! SSE (Superset ENG) in the REST API: the `scramble` field reads it, the
//! `notation` field (`sse`) writes it.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_expression;
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn call(request: Request<Body>) -> (StatusCode, Value) {
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(None);
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    let app = router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ));
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn post(path: &str, body: Value) -> Request<Body> {
    Request::post(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn convert_reads_sse_like_wca() {
    let cases = [
        (5, "TR N2-3U' V3F (R NL)2", "Rw 3Uw' U 4Fw F' R 2L R 2L"),
        (3, "MR SU CF2 R-", "2R U D' z2 R'"),
        (7, "WL M2R T4B2", "6Lw L' 4Rw Rw' 4Bw2"),
    ];
    for (size, sse, wca) in cases {
        let (status, a) = call(post("/v1/convert", json!({"scramble": sse, "size": size}))).await;
        assert_eq!(status, StatusCode::OK, "{a}");
        let (_, b) = call(post("/v1/convert", json!({"scramble": wca, "size": size}))).await;
        assert_eq!(a["facelets"], b["facelets"], "{sse}");
    }
}

#[tokio::test]
async fn a_bad_sse_token_is_a_400_with_its_position() {
    let (status, body) = call(post("/v1/convert", json!({"scramble": "R N2R", "size": 4}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("character 2") && message.contains("5x5"),
        "{message}"
    );
}

#[tokio::test]
async fn solve_and_scramble_write_sse() {
    let scramble = "R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l";
    let (status, body) = call(post(
        "/v1/solve",
        json!({"scramble": scramble, "size": 4, "notation": "sse"}),
    ))
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let kind = PuzzleKind::Cube4;
    let solution = parse_expression(body["solution"].as_str().unwrap(), kind).unwrap();
    assert!(!body["solution"].as_str().unwrap().contains('w'));
    let cube = Facelets::solved(kind)
        .apply(&parse_expression(scramble, kind).unwrap())
        .unwrap();
    assert!(cube.apply(&solution).unwrap().is_solved());

    for size in 2..=7usize {
        let uri = format!("/v1/scramble?size={size}&moves=40&notation=sse");
        let (status, body) = call(Request::get(uri).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let kind = PuzzleKind::of_size(size).unwrap();
        let text = body["scramble"].as_str().unwrap();
        assert_eq!(parse_expression(text, kind).unwrap().len(), 40, "{text}");
    }
}
