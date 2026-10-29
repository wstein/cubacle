//! The `inverse` field of `/v1/solve`: the moves that undo `solution`.

mod shared;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_moves;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn call(body: Value) -> (StatusCode, Value) {
    let app = router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        shared::five(),
        shared::four(),
    ));
    let request = Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn the_inverse_builds_the_start_from_the_solved_cube() {
    let scramble = "R U F D2 L' B R2 U' F2 D L2 B' U2 R F";
    for notation in ["wca", "sign"] {
        let (status, answer) = call(json!({"scramble": scramble, "notation": notation})).await;
        assert_eq!(status, StatusCode::OK, "{answer}");
        let inverse = answer["inverse"].as_str().expect("inverse is a string");
        let back = parse_moves(inverse).unwrap();
        let start = Facelets::solved(PuzzleKind::Cube3)
            .apply(&parse_moves(scramble).unwrap())
            .unwrap();
        let built = Facelets::solved(PuzzleKind::Cube3).apply(&back).unwrap();
        assert_eq!(built, start, "{notation}");
        // the existing fields are unchanged
        let solution: MoveSequence = parse_moves(answer["solution"].as_str().unwrap()).unwrap();
        assert_eq!(solution.len() as u64, answer["length"].as_u64().unwrap());
    }
}
