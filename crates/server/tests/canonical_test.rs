//! The `canonical` field of the state descriptions (`/v1/convert`,
//! `/v1/random`): the canonical Orbit64 token and its schema name.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
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
async fn convert_answers_the_canonical_token_and_its_schema() {
    let (status, body) = call(post("/v1/convert", json!({"scramble": "R U"}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["canonical"],
        json!({"token": "C0RkZMybxEmw", "schema": "orbit64-3x3-draft@1"})
    );
    // the existing fields are unchanged
    assert_eq!(body["kind"], "3x3x3");
    assert!(body["facelets"].is_string() && body["token"].is_string());
}

#[tokio::test]
async fn a_mirror_image_has_the_same_canonical_token_on_every_size() {
    for (size, moves, mirror) in [
        (2, "R U", "L' U'"),
        (3, "R U F", "L' U' F'"),
        (4, "Rw U", "Lw' U'"),
        (7, "R U Rw", "L' U' Lw'"),
    ] {
        let (_, a) = call(post(
            "/v1/convert",
            json!({"scramble": moves, "size": size}),
        ))
        .await;
        let (_, b) = call(post(
            "/v1/convert",
            json!({"scramble": mirror, "size": size}),
        ))
        .await;
        assert_ne!(a["token"], b["token"], "{moves}");
        assert_eq!(a["canonical"], b["canonical"], "{moves}");
        let schema = format!("orbit64-{size}x{size}-draft@1");
        assert_eq!(a["canonical"]["schema"], schema.as_str());
    }
}

#[tokio::test]
async fn a_token_in_gives_its_canonical_token_out() {
    let (_, solved) = call(post("/v1/convert", json!({"cube": "AAAAAAAAAAAA"}))).await;
    assert_eq!(solved["canonical"]["token"], "AAAAAAAAAAAA");
    let (_, a) = call(post("/v1/convert", json!({"scramble": "R U"}))).await;
    let (_, b) = call(post(
        "/v1/convert",
        json!({"cube": a["canonical"]["token"]}),
    ))
    .await;
    assert_eq!(b["canonical"], a["canonical"]);
}

#[tokio::test]
async fn random_carries_the_field_and_an_impossible_cube_does_not() {
    let request = Request::get("/v1/random?size=4")
        .body(Body::empty())
        .unwrap();
    let (_, body) = call(request).await;
    assert_eq!(body["canonical"]["schema"], "orbit64-4x4-draft@1");
    assert_eq!(body["canonical"]["token"].as_str().unwrap().len(), 27);
    // a twisted corner is no cube: the answer has no token and no canonical
    let twisted = "UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let (_, body) = call(post("/v1/convert", json!({"cube": twisted}))).await;
    assert!(body.get("canonical").is_none(), "{body}");
    assert!(body.get("token").is_none(), "{body}");
}
