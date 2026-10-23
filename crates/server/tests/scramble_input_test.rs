//! The `scramble` and `size` fields of `/v1/convert` and `/v1/solve`: moves in
//! WCA or SiGN notation, with layer ranges, groups and commutators.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::{five::FiveSolver, four::FourSolver};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn post(path: &str, body: Value) -> (StatusCode, Value) {
    let five = FiveSolver::new(Arc::new(Min2PhaseSolver::shared())).with_cache_dir(None);
    let four = FourSolver::new(Arc::new(Min2PhaseSolver::shared())).with_threads(1);
    let app = router(AppState::with_engines(
        Config::default(),
        Min2PhaseSolver::shared(),
        five,
        four,
    ));
    let request = Request::post(path)
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

async fn facelets(body: Value) -> String {
    let (status, value) = post("/v1/convert", body).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["facelets"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn convert_reads_a_scramble_in_either_notation() {
    let cases = [
        (5, "2-3r", "3Rw R'"),
        (5, "2-3Rw", "3Rw R'"),
        (4, "[r: U]", "Rw U Rw'"),
        (3, "[R, U]", "R U R' U'"),
        (3, "(R U)2'", "U' R' U' R'"),
        (7, "[3-5l2: u]", "5Lw2 2Lw2 Uw 2Lw2 5Lw2"),
    ];
    for (size, short, plain) in cases {
        let a = facelets(json!({"scramble": short, "size": size})).await;
        let b = facelets(json!({"scramble": plain, "size": size})).await;
        assert_eq!(a, b, "{short}");
    }
    // 3x3x3 by default
    let a = facelets(json!({"scramble": "R U"})).await;
    assert_eq!(a, facelets(json!({"scramble": "R U", "size": 3})).await);
}

#[tokio::test]
async fn solve_takes_a_scramble_and_writes_sign() {
    let (status, body) = post(
        "/v1/solve",
        json!({"scramble": "[R: U2] (F D)2", "notation": "sign", "maxMoves": 25}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["length"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn bad_scrambles_are_400_with_a_position() {
    for (body, needle) in [
        (json!({"scramble": "R [U, F, D]"}), "character 7"),
        (json!({"scramble": "R 2-3r"}), "does not exist"),
        (json!({"scramble": "R U", "size": 9}), "size must be"),
        (json!({"scramble": "R U", "cube": "UUUU"}), "not both"),
        (json!({}), "send a cube or a scramble"),
        (
            json!({"cube": "UUUURRRRFFFFDDDDLLLLBBBB", "size": 2}),
            "size goes",
        ),
        (json!({"scramble": "R", "scheme": "boy"}), "scheme goes"),
        (json!({"scramble": "(R U"}), "unbalanced"),
        (json!({"scramble": "(R)*1000000"}), "more than"),
    ] {
        let (status, value) = post("/v1/convert", body.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        let message = value["error"]["message"].as_str().unwrap();
        assert!(message.contains(needle), "{body}: {message}");
    }
}
