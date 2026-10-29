//! HTTP solve version selection uses the same writer and actual-result evidence as the CLI.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn solve(body: Value) -> (StatusCode, Value) {
    // These cases use only 2x2/3x3 or reject before search; do not build unrelated 5x5 tables.
    let app = router(AppState::unprepared(Config::default()));
    let response = app
        .oneshot(
            Request::post("/v1/solve")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn default_two_and_explicit_one_have_the_same_original_fields() {
    let request = json!({"scramble":"R U", "maxMoves":21});
    let (status, mut two) = solve(request.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(two["formatVersion"], 2);
    assert_eq!(two["frame"], "fixed");
    assert_eq!(two["proven"], false);
    assert!(two["lowerBound"].is_null());
    let mut old = request;
    old["formatVersion"] = json!(1);
    let (status, mut one) = solve(old).await;
    assert_eq!(status, StatusCode::OK);
    for key in [
        "formatVersion",
        "toolVersion",
        "gitCommit",
        "frame",
        "proven",
        "lowerBound",
    ] {
        two.as_object_mut().unwrap().remove(key);
        assert!(one.get(key).is_none());
    }
    one.as_object_mut().unwrap().remove("elapsedMicros");
    two.as_object_mut().unwrap().remove("elapsedMicros");
    assert_eq!(one, two);
}

#[tokio::test]
async fn quotient_solver_reports_its_real_goal_and_htm_proof() {
    let (status, body) = solve(json!({"scramble":"U D'", "size":2})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["frame"], "any-orientation");
    assert_eq!(body["proven"], true);
    assert_eq!(body["lowerBound"], 0);
    assert_eq!(body["length"], 0);
}

#[tokio::test]
async fn unknown_or_non_integer_versions_are_rejected_before_solving() {
    for version in [json!(0), json!(3), json!(1.0), json!("1"), json!(null)] {
        let (status, body) = solve(json!({"scramble":"R U", "formatVersion":version})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["error"]["code"], "bad_input");
    }
}
