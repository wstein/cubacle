//! The `target` field of `/v1/solve`: the moves lead from the start to a state
//! other than the solved cube.

mod support;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::notation::parse_expression;
use cubacle::puzzle::{reaches, Facelets, MoveSequence, PuzzleKind};
use cubacle_server::{router, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn post(path: &str, body: Value) -> (StatusCode, Value) {
    let app = router(support::state(Config::default()));
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

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = parse_expression(moves, kind).unwrap();
    Facelets::solved(kind).apply(&moves).unwrap()
}

#[test]
fn sharing_test_tables_does_not_share_request_limits() {
    let first = support::state(Config {
        max_concurrency: 1,
        ..Config::default()
    });
    let second = support::state(Config {
        max_concurrency: 2,
        ..Config::default()
    });
    assert!(!Arc::ptr_eq(first.permits(), second.permits()));
    let permit = first.permits().try_acquire().unwrap();
    assert!(first.permits().try_acquire().is_err());
    assert_eq!(second.permits().available_permits(), 2);
    drop(permit);
    assert_eq!(first.permits().available_permits(), 1);
}

/// Solves `scramble` to `target` and checks by replay that the answer ends there.
async fn solved_to(size: usize, scramble: &str, target: Value, end: &Facelets) {
    let kind = PuzzleKind::of_size(size).unwrap();
    let (status, body) = post(
        "/v1/solve",
        json!({"scramble": scramble, "size": size, "target": target}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let moves: MoveSequence = body["solution"].as_str().unwrap().parse().unwrap();
    assert_eq!(body["length"].as_u64().unwrap() as usize, moves.len());
    assert!(reaches(&cube(kind, scramble), &moves, end), "{body}");
}

#[tokio::test]
async fn a_target_given_as_moves_or_as_a_cube_is_reached() {
    let end = cube(PuzzleKind::Cube3, "L2 B' U F D2");
    solved_to(3, "R U F2 D' L", json!("L2 B' U F D2"), &end).await;
    solved_to(3, "R U F2 D' L", json!(end.to_string()), &end).await;
    let (_, token) = post("/v1/convert", json!({"scramble": "L2 B' U F D2"})).await;
    solved_to(3, "R U F2 D' L", token["token"].clone(), &end).await;
}

#[tokio::test]
async fn a_pattern_name_is_a_target() {
    let flip = cube(
        PuzzleKind::Cube3,
        "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
    );
    solved_to(3, "R U F2 D' L", json!("superflip"), &flip).await;
}

#[tokio::test]
async fn smaller_and_larger_cubes_have_targets_too() {
    solved_to(
        2,
        "R U F2",
        json!("U R2 F'"),
        &cube(PuzzleKind::Cube2, "U R2 F'"),
    )
    .await;
    let end = cube(PuzzleKind::Cube4, "U2 R' L");
    solved_to(4, "R U Rw Fw", json!("U2 R' L"), &end).await;
}

#[tokio::test]
async fn a_target_with_mixed_centers_is_reached_by_a_center_fix_up() {
    // inner slices and wide turns leave centers of one color mixed on a face
    let end = cube(PuzzleKind::Cube4, "2R 2U' 2F");
    solved_to(4, "R U Rw Fw", json!("2R 2U' 2F"), &end).await;
    solved_to(4, "R U Rw Fw", json!(end.to_string()), &end).await;
    let end = cube(PuzzleKind::Cube4, "Rw");
    solved_to(4, "", json!("Rw"), &end).await;
    let end = cube(PuzzleKind::Cube5, "2R 4U' Fw");
    solved_to(5, "R U F2", json!("2R 4U' Fw"), &end).await;
}

#[tokio::test]
async fn a_cube_string_wins_over_a_move_expression() {
    // "UUUU..." is a 3x3x3 in facelets; a text that is no cube is moves
    let solved = Facelets::solved(PuzzleKind::Cube3);
    solved_to(3, "R U", json!(solved.to_string()), &solved).await;
}

#[tokio::test]
async fn a_bad_target_is_refused() {
    let two = Facelets::solved(PuzzleKind::Cube2).to_string();
    let mut stickers: Vec<char> = Facelets::solved(PuzzleKind::Cube3)
        .to_string()
        .chars()
        .collect();
    stickers.swap(8, 10);
    let broken: String = stickers.into_iter().collect();
    // two center stickers of different orbits swapped: no 5x5 looks so
    let mut stickers = cube(PuzzleKind::Cube5, "").stickers().to_vec();
    stickers.swap(6, 32);
    let swapped = Facelets::from_stickers(stickers).unwrap().to_string();
    for (body, status, needle) in [
        (
            json!({"scramble": "R U", "target": two}),
            StatusCode::BAD_REQUEST,
            "2x2x2",
        ),
        (
            json!({"scramble": "R U", "target": "superflip R"}),
            StatusCode::BAD_REQUEST,
            "not a cube",
        ),
        (
            json!({"scramble": "R U", "size": 4, "target": "superflip"}),
            StatusCode::BAD_REQUEST,
            "3x3x3 pattern",
        ),
        (
            json!({"scramble": "R U", "target": ""}),
            StatusCode::BAD_REQUEST,
            "target",
        ),
        (
            json!({"scramble": "R U", "target": broken}),
            StatusCode::UNPROCESSABLE_ENTITY,
            "impossible",
        ),
        (
            json!({"scramble": "R U", "size": 5, "target": swapped}),
            StatusCode::UNPROCESSABLE_ENTITY,
            "center orbit",
        ),
    ] {
        let (got, value) = post("/v1/solve", body.clone()).await;
        assert_eq!(got, status, "{body}: {value}");
        let message = value["error"]["message"].as_str().unwrap();
        assert!(message.contains(needle), "{body}: {message}");
    }
}

#[tokio::test]
async fn target_belongs_to_solve_only() {
    for path in ["/v1/convert"] {
        let (status, _) = post(path, json!({"scramble": "R", "target": "U"})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}");
    }
}
