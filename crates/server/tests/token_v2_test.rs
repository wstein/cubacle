//! The `tokenV2` field of the state descriptions (`/v1/convert`, `/v1/random`):
//! the Orbit64 V2 token (color model, framed pose), additive to the other keys.

mod support;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::notation::orbit64::v2;
use cubacle_server::{router, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn call(request: Request<Body>) -> (StatusCode, Value) {
    let app = router(support::state(Config::default()));
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

/// The facelets a V2 token shows, without the field separators.
fn facelets_of(token: &str) -> String {
    v2::facelets::decode(token)
        .expect("a V2 token")
        .1
        .replace(' ', "")
}

#[tokio::test]
async fn convert_answers_the_v2_token_of_the_state() {
    let (status, body) = call(post("/v1/convert", json!({"scramble": "R U x y2"}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // a rotation stays in the framed token
    assert_eq!(body["tokenV2"], "AwtB4B1fWQAG");
    assert_eq!(
        facelets_of(body["tokenV2"].as_str().unwrap()),
        body["facelets"]
    );
    // the existing fields are unchanged
    assert_eq!(body["kind"], "3x3x3");
    assert!(body["token"].is_string() && body["canonical"].is_object());
}

#[tokio::test]
async fn the_token_field_is_the_framed_color_v2_token() {
    for size in [2, 3, 4, 5, 6, 7] {
        let request = Request::get(format!("/v1/random?size={size}"))
            .body(Body::empty())
            .unwrap();
        let (_, body) = call(request).await;
        assert_eq!(body["tokenV2"], body["token"], "size {size}");
        assert_eq!(
            facelets_of(body["tokenV2"].as_str().unwrap()),
            body["facelets"],
            "size {size}"
        );
    }
    assert_eq!(
        call(
            Request::get("/v1/random?size=5")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .1["token"]
            .as_str()
            .unwrap()
            .len(),
        42
    );
}

#[tokio::test]
async fn a_twisted_corner_has_no_v2_token() {
    let twisted = "UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let (_, body) = call(post("/v1/convert", json!({"cube": twisted}))).await;
    assert!(body.get("tokenV2").is_none(), "{body}");
}
