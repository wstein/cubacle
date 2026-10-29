//! The `metrics` object of `/v1/solve`: seven counts that equal what
//! `cubacle::metrics` counts on the returned moves.

mod shared;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::metrics::{count, count_text, Metric};
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

async fn solve(body: Value) -> (Value, String) {
    let request = Request::post("/v1/solve")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let raw = String::from_utf8(bytes.to_vec()).unwrap();
    (serde_json::from_str(&raw).unwrap(), raw)
}

fn scrambled(kind: PuzzleKind, moves: &str) -> String {
    Facelets::solved(kind)
        .apply(&parse_moves(moves).unwrap())
        .unwrap()
        .to_string()
}

fn check(answer: &Value, kind: PuzzleKind) {
    let moves = parse_moves(answer["solution"].as_str().unwrap()).unwrap();
    for metric in Metric::ALL {
        assert_eq!(
            answer["metrics"][metric.name()].as_u64().unwrap() as usize,
            count(&moves, kind, metric),
            "{}",
            metric.name()
        );
    }
}

#[tokio::test]
async fn the_counts_equal_the_library_counts_on_every_size() {
    let cases = [
        (PuzzleKind::Cube2, "R U R' U' F2"),
        (PuzzleKind::Cube3, "R U R' U' F2 D L2 B'"),
        (PuzzleKind::Cube4, "R U 2R 2U' f 2L' B2 D 2D' R' U2 2F"),
    ];
    for (kind, scramble) in cases {
        let (answer, _) = solve(json!({"cube": scrambled(kind, scramble)})).await;
        check(&answer, kind);
    }
}

#[tokio::test]
async fn the_keys_come_in_a_fixed_order() {
    let (_, raw) = solve(json!({"cube": scrambled(PuzzleKind::Cube3, "R U F")})).await;
    let at = |key: &str| raw.find(&format!("\"{key}\":")).unwrap();
    let order = ["htm", "qtm", "stm", "etm", "obtm", "btm", "atm"];
    for pair in order.windows(2) {
        assert!(at(pair[0]) < at(pair[1]), "{raw}");
    }
}

#[tokio::test]
async fn ranges_get_their_own_counts() {
    let cube = scrambled(
        PuzzleKind::Cube4,
        "R U 2R 2U' f 2L' B2 D 2D' R' U2 2F u r' F2 l",
    );
    let (answer, _) = solve(json!({"cube": cube, "optimize": "ranges"})).await;
    check(&answer, PuzzleKind::Cube4);
    let text = answer["compressed"].as_str().unwrap();
    for metric in Metric::ALL {
        assert_eq!(
            answer["compressedMetrics"][metric.name()].as_u64().unwrap() as usize,
            count_text(text, PuzzleKind::Cube4, metric).unwrap(),
            "{}",
            metric.name()
        );
    }
    let (plain, _) = solve(json!({"cube": cube})).await;
    assert!(plain.get("compressedMetrics").is_none());
}
