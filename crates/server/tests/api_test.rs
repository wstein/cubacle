use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use cubacle_server::{router, AppState, Config};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app_with(config: Config) -> (Router, AppState) {
    let state = AppState::new(config);
    (router(state.clone()), state)
}

fn app() -> Router {
    app_with(Config::default()).0
}

async fn call(app: Router, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, value)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post(uri: &str, body: impl Into<Body>) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

fn post_json(uri: &str, value: Value) -> Request<Body> {
    post(uri, value.to_string())
}

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap()
}

const SOLVED3: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

#[tokio::test]
async fn health_is_ok() {
    let (status, _, body) = call(app(), get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"status": "ok"}));
}

#[tokio::test]
async fn scramble_default_and_validation() {
    let (status, _, body) = call(app(), get("/v1/scramble")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["scramble"]
            .as_str()
            .unwrap()
            .split(' ')
            .filter(|m| !m.is_empty())
            .count(),
        25
    );
    let (_, _, body) = call(app(), get("/v1/scramble?moves=7")).await;
    assert_eq!(
        body["scramble"]
            .as_str()
            .unwrap()
            .split(' ')
            .filter(|m| !m.is_empty())
            .count(),
        7
    );
    for bad in ["0", "201", "abc", "-1"] {
        let (status, _, body) = call(app(), get(&format!("/v1/scramble?moves={bad}"))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(code(&body), "bad_input");
    }
}

#[tokio::test]
async fn random_cubes_for_supported_sizes() {
    let (status, _, body) = call(app(), get("/v1/random")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["kind"], "3x3x3");
    assert_eq!(body["facelets"].as_str().unwrap().len(), 54);

    let (_, _, body) = call(app(), get("/v1/random?size=2")).await;
    assert_eq!(body["kind"], "2x2x2");
    let facelets = body["facelets"].as_str().unwrap().to_owned();
    assert_eq!(facelets.len(), 24);
    // A random 2x2 must be solvable.
    let (status, _, _) = call(app(), post_json("/v1/solve", json!({"cube": facelets}))).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _, body) = call(app(), get("/v1/random?size=4")).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(code(&body), "no_engine");
}

#[tokio::test]
async fn convert_token_and_colours() {
    let (status, _, body) = call(
        app(),
        post_json("/v1/convert", json!({"cube": "A4ib8aejxj4Y"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["kind"], "3x3x3");
    assert_eq!(body["facelets"].as_str().unwrap().len(), 54);

    let colours = "WWWWWWWWWRRRRRRRRRGGGGGGGGGYYYYYYYYYOOOOOOOOOBBBBBBBBB";
    let (status, _, body) = call(
        app(),
        post_json(
            "/v1/convert",
            json!({"cube": colours, "scheme": "western-boy"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["facelets"], SOLVED3);

    let (status, _, body) = call(
        app(),
        post_json(
            "/v1/convert",
            json!({"cube": colours, "scheme": "nonsense"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(code(&body), "bad_input");
}

#[tokio::test]
async fn solves_a_3x3_token_and_the_solution_replays() {
    let (status, _, body) = call(
        app(),
        post_json("/v1/convert", json!({"cube": "A4ib8aejxj4Y"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let facelets = body["facelets"].as_str().unwrap().to_owned();

    let (status, _, body) = call(
        app(),
        post_json("/v1/solve", json!({"cube": "A4ib8aejxj4Y", "maxMoves": 21})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["kind"], "3x3x3");
    let solution = body["solution"].as_str().unwrap();
    assert_eq!(
        body["length"].as_u64().unwrap() as usize,
        solution.split(' ').filter(|m| !m.is_empty()).count()
    );
    assert!(body["elapsedMicros"].is_u64());
    assert_eq!(
        cubacle::apply_moves(&facelets, solution).as_deref(),
        Some(SOLVED3)
    );
}

#[tokio::test]
async fn solves_a_2x2_token() {
    let (status, _, body) = call(app(), post_json("/v1/solve", json!({"cube": "CFJnI"}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["kind"], "2x2x2");
    assert!(body["length"].as_u64().unwrap() <= 11);
}

#[tokio::test]
async fn bad_requests() {
    let cases = [
        post("/v1/solve", "{not json"),
        post("/v1/solve", "{}"),
        post_json("/v1/solve", json!({"cube": "hello"})),
        post_json("/v1/solve", json!({"cube": "CFJnI", "maxMoves": 0})),
        post_json("/v1/solve", json!({"cube": "CFJnI", "budgetMs": 0})),
        post_json("/v1/solve", json!({"cube": "CFJnI", "extra": 1})),
    ];
    for request in cases {
        let (status, _, body) = call(app(), request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(code(&body), "bad_input");
    }
}

#[tokio::test]
async fn impossible_cube_is_422() {
    // One corner twisted in place: right sticker counts, unreachable state.
    let twisted = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let mut chars: Vec<char> = twisted.chars().collect();
    chars.swap(8, 9); // U9 <-> R1
    let twisted: String = chars.into_iter().collect();
    let (status, _, body) = call(app(), post_json("/v1/solve", json!({"cube": twisted}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(code(&body), "impossible_cube");
}

#[tokio::test]
async fn move_limit_too_small_is_422() {
    let scramble = cubacle::from_moves("R U F D L B R U F D").unwrap();
    let (status, _, body) = call(
        app(),
        post_json("/v1/solve", json!({"cube": scramble, "maxMoves": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(code(&body), "no_solution_within_limit");
}

#[tokio::test]
async fn unsupported_size_is_501() {
    let cube = "U".repeat(16)
        + &"R".repeat(16)
        + &"F".repeat(16)
        + &"D".repeat(16)
        + &"L".repeat(16)
        + &"B".repeat(16);
    let (status, _, body) = call(app(), post_json("/v1/solve", json!({"cube": cube}))).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{body}");
    assert_eq!(code(&body), "no_engine");
}

#[tokio::test]
async fn oversize_body_is_413() {
    let config = Config {
        max_body_bytes: 64,
        ..Config::default()
    };
    let (app, _) = app_with(config);
    let big = json!({"cube": "x".repeat(500)});
    let (status, _, body) = call(app, post_json("/v1/solve", big)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(code(&body), "payload_too_large");
}

#[tokio::test]
async fn saturated_pool_answers_429_with_retry_after() {
    let config = Config {
        max_concurrency: 1,
        ..Config::default()
    };
    let (app, state) = app_with(config);
    let held = state.permits().clone().try_acquire_owned().unwrap();
    let (status, headers, body) = call(
        app.clone(),
        post_json("/v1/solve", json!({"cube": "CFJnI"})),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(&body), "busy");
    assert!(headers.contains_key(header::RETRY_AFTER));
    drop(held);
    let (status, _, _) = call(app, post_json("/v1/solve", json!({"cube": "CFJnI"}))).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unknown_path_is_json_404() {
    let (status, _, body) = call(app(), get("/nope")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(code(&body), "not_found");
}

#[tokio::test]
async fn real_socket_smoke() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = app();
    let server = tokio::spawn(async move { axum::serve(listener, app).await });

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).await.unwrap();
    assert!(text.starts_with("HTTP/1.1 200"), "{text}");
    assert!(text.contains(r#"{"status":"ok"}"#), "{text}");
    server.abort();
}
