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

    // Replay on a 3x3 whose edges and centres are solved: afterwards every
    // face must show one colour on its four corner stickers.
    let (_, _, converted) = call(app(), post_json("/v1/convert", json!({"cube": "CFJnI"}))).await;
    let two: Vec<char> = converted["facelets"].as_str().unwrap().chars().collect();
    let mut three: Vec<char> = SOLVED3.chars().collect();
    for face in 0..6 {
        for (k, index) in [0, 2, 6, 8].into_iter().enumerate() {
            three[face * 9 + index] = two[face * 4 + k];
        }
    }
    // The 3x3 must be a legal cube: if the corners are an odd permutation,
    // swap two edges (UR and UF) to match the parity.
    let solution = body["solution"].as_str().unwrap();
    let plain: String = three.iter().collect();
    three.swap(5, 7);
    three.swap(10, 19);
    let swapped: String = three.iter().collect();
    let replayed = cubacle::apply_moves(&plain, solution)
        .or_else(|| cubacle::apply_moves(&swapped, solution))
        .expect("one of the two embeddings is a legal 3x3");
    let r: Vec<char> = replayed.chars().collect();
    for face in 0..6 {
        let corners = [0, 2, 6, 8].map(|i| r[face * 9 + i]);
        assert!(
            corners.iter().all(|c| *c == corners[0]),
            "face {face}: {replayed}"
        );
    }
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
    // Two stickers swapped (U9 and R1): right sticker counts, unreachable state.
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

const SUPERFLIP: &str = "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

#[tokio::test]
async fn text_plain_posts_are_rejected_with_415() {
    for uri in ["/v1/solve", "/v1/convert"] {
        let request = Request::post(uri)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(r#"{"cube":"CFJnI"}"#))
            .unwrap();
        let (status, _, body) = call(app(), request).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE, "{uri}");
        assert_eq!(code(&body), "unsupported_media_type");
        let request = Request::post(uri)
            .body(Body::from(r#"{"cube":"CFJnI"}"#))
            .unwrap();
        let (status, _, _) = call(app(), request).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let request = Request::post("/v1/convert")
        .header(header::CONTENT_TYPE, "Application/JSON; charset=utf-8")
        .body(Body::from(r#"{"cube":"CFJnI"}"#))
        .unwrap();
    assert_eq!(call(app(), request).await.0, StatusCode::OK);
}

#[tokio::test]
async fn wrong_methods_get_json_405() {
    for request in [
        Request::post("/v1/scramble").body(Body::empty()).unwrap(),
        get("/v1/solve"),
        Request::options("/v1/solve").body(Body::empty()).unwrap(),
    ] {
        let (status, headers, body) = call(app(), request).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(code(&body), "method_not_allowed");
        assert!(headers.contains_key(header::ALLOW), "{headers:?}");
    }
}

#[tokio::test]
async fn size_validation_distinguishes_400_and_501() {
    for size in ["0", "1", "8", "18446744073709551615"] {
        let (status, _, body) = call(app(), get(&format!("/v1/random?size={size}"))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{size}");
        assert_eq!(code(&body), "bad_input");
    }
    for size in ["4", "7"] {
        let (status, _, body) = call(app(), get(&format!("/v1/random?size={size}"))).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{size}");
        assert_eq!(code(&body), "no_engine");
    }
}

#[tokio::test]
async fn orbit64_tokens_of_unsupported_sizes_are_501() {
    // 27 characters is the width of a 4x4x4 token.
    for uri in ["/v1/solve", "/v1/convert"] {
        let (status, _, body) = call(app(), post_json(uri, json!({"cube": "A".repeat(27)}))).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{uri}: {body}");
        assert_eq!(code(&body), "no_engine");
    }
}

#[tokio::test]
async fn unknown_parameters_and_wrong_fields_are_400() {
    for uri in [
        "/v1/scramble?moves=5&x=1",
        "/v1/random?size=3&x=1",
        "/v1/random?foo",
    ] {
        let (status, _, body) = call(app(), get(uri)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(code(&body), "bad_input");
    }
    for extra in [json!({"maxMoves": 5}), json!({"budgetMs": 5})] {
        let mut request = json!({"cube": "CFJnI"});
        request
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let (status, _, _) = call(app(), post_json("/v1/convert", request)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    for (field, value) in [("maxMoves", 26), ("maxMoves", 0), ("budgetMs", 600_001)] {
        let (status, _, body) = call(
            app(),
            post_json("/v1/solve", json!({"cube": "CFJnI", field: value})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{field}={value}");
        assert_eq!(code(&body), "bad_input");
    }
}

#[tokio::test]
async fn engine_errors_are_readable() {
    // One edge flipped in place: an error from the engine.
    let mut chars: Vec<char> = SOLVED3.chars().collect();
    chars.swap(5, 10);
    let cube: String = chars.into_iter().collect();
    let (status, _, body) = call(app(), post_json("/v1/solve", json!({"cube": cube}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(!body["error"]["message"]
        .as_str()
        .unwrap()
        .starts_with("Error"));
}

#[tokio::test]
async fn exhausted_budget_is_504() {
    // Constructed directly: the CLI refuses a zero timeout.
    let config = Config {
        request_timeout: std::time::Duration::ZERO,
        ..Config::default()
    };
    let (app, _) = app_with(config);
    let (status, _, body) = call(app, post_json("/v1/solve", json!({"cube": SUPERFLIP}))).await;
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT, "{body}");
    assert_eq!(code(&body), "budget_exhausted");

    // And through a real budget: the superflip has no 12-move solution and
    // the search runs until the budget ends.
    let (status, _, body) = call(
        app_with(Config::default()).0,
        post_json(
            "/v1/solve",
            json!({"cube": SUPERFLIP, "maxMoves": 12, "budgetMs": 150}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT, "{body}");
}

async fn boom() -> &'static str {
    panic!("test panic")
}

#[tokio::test]
async fn handler_panics_become_json_500() {
    use axum::routing::get as route_get;
    let state = AppState::unprepared(Config::default());
    let routes = Router::new().route("/boom", route_get(boom));
    let app = cubacle_server::finish(routes, &state).with_state(state);
    let (status, _, body) = call(app, get("/boom")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(code(&body), "internal");
}

#[tokio::test]
async fn permit_is_held_until_the_solve_ends_even_if_the_client_leaves() {
    let config = Config {
        max_concurrency: 1,
        ..Config::default()
    };
    let (app, state) = app_with(config);
    let slow = json!({"cube": SUPERFLIP, "maxMoves": 12, "budgetMs": 400});
    let abandoned = tokio::spawn(app.clone().oneshot(post_json("/v1/solve", slow)));
    // Wait until the solve really holds the permit, then drop the client.
    for _ in 0..200 {
        if state.permits().available_permits() == 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    abandoned.abort();
    let (status, _, body) = call(
        app.clone(),
        post_json("/v1/solve", json!({"cube": "CFJnI"})),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    for _ in 0..200 {
        if state.permits().available_permits() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let (status, _, _) = call(app, post_json("/v1/solve", json!({"cube": "CFJnI"}))).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unprepared_state_serves_non_solving_endpoints() {
    let app = router(AppState::unprepared(Config::default()));
    assert_eq!(call(app.clone(), get("/health")).await.0, StatusCode::OK);
    assert_eq!(call(app, get("/v1/scramble")).await.0, StatusCode::OK);
}
