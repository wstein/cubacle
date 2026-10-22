use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

const CUBE: &str = "A4ib8aejxj4Y";

async fn solve(body: Value) -> (StatusCode, Value) {
    let three = Min2PhaseSolver::shared();
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("five-tables");
    let five = FiveSolver::new(std::sync::Arc::new(three.clone())).with_cache_dir(Some(dir));
    let four = FourSolver::new(std::sync::Arc::new(three.clone())).with_threads(1);
    let app = router(AppState::with_engines(Config::default(), three, five, four));
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
async fn min_probes_is_deterministic_and_never_longer() {
    let (_, plain) = solve(json!({"cube": CUBE})).await;
    let (status, a) = solve(json!({"cube": CUBE, "minProbes": 500})).await;
    assert_eq!(status, StatusCode::OK, "{a}");
    let (_, b) = solve(json!({"cube": CUBE, "minProbes": 500})).await;
    assert_eq!(a["solution"], b["solution"]);
    assert!(a["length"].as_u64() <= plain["length"].as_u64());
}

#[tokio::test]
async fn improve_returns_a_solution_when_the_budget_is_spent() {
    let (status, body) = solve(json!({"cube": CUBE, "improve": true, "budgetMs": 100})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["solution"].is_string());
    assert!(body["elapsedMicros"].as_u64().unwrap() < 5_000_000);
}

#[tokio::test]
async fn improve_options_are_validated() {
    let two = "CFJnI";
    let cases = [
        json!({"cube": CUBE, "improve": true}),
        json!({"cube": CUBE, "minProbes": 0}),
        json!({"cube": CUBE, "minProbes": 100_000_001u64}),
        json!({"cube": CUBE, "improve": true, "minProbes": 5, "budgetMs": 100}),
        json!({"cube": two, "minProbes": 5}),
        json!({"cube": two, "improve": true, "budgetMs": 100}),
    ];
    for case in cases {
        let (status, body) = solve(case.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{case}: {body}");
        assert_eq!(body["error"]["code"], "bad_input", "{case}");
    }
}

#[tokio::test]
async fn min_probes_works_for_the_4x4x4_and_5x5x5_and_is_deterministic() {
    for size in [4usize, 5] {
        let kind = cubacle::puzzle::PuzzleKind::of_size(size).unwrap();
        let cube = cubacle::puzzle::Facelets::solved(kind)
            .apply(&"R U F' Rw2 Uw Fw' L D2 B".parse().unwrap())
            .unwrap()
            .to_string();
        let (status, a) = solve(json!({"cube": cube, "minProbes": 1000})).await;
        assert_eq!(status, StatusCode::OK, "{size}: {a}");
        let (_, b) = solve(json!({"cube": cube, "minProbes": 1000})).await;
        assert_eq!(a["solution"], b["solution"], "{size}");
        let (status, body) = solve(json!({"cube": cube, "improve": true, "budgetMs": 100})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{size}: {body}");
        let (status, body) = solve(json!({"cube": cube, "minProbes": 0})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{size}: {body}");
    }
}
