# cubacle-server

Stateless HTTP backend for the cubacle solvers (2x2x2, 3x3x3, 4x4x4 and 5x5x5). No sessions and no per-client state, so several instances can run behind a load balancer. Engine tables are built once at startup, before the listener accepts connections. Solving runs on blocking threads guarded by a semaphore (`--max-concurrency`); when all slots are busy the server answers 429 with `Retry-After`.

```sh
cargo run --release -p cubacle-server -- --bind 127.0.0.1:8080
```

| Flag | Env | Default |
|---|---|---|
| `--bind` | `CUBACLE_BIND` | `127.0.0.1:8080` (local only) |
| `--max-concurrency` | `CUBACLE_MAX_CONCURRENCY` | number of CPUs (range 1..=256) |
| `--request-timeout-ms` | `CUBACLE_REQUEST_TIMEOUT_MS` | 30000 (range 1..=600000; cap and default of `budgetMs`) |
| `--max-body-bytes` | `CUBACLE_MAX_BODY_BYTES` | 16384 |
| `--log` | `CUBACLE_LOG` | `info` |

SIGINT/SIGTERM trigger a graceful shutdown: in-flight requests may finish, but for at most `request-timeout + 5 s`, after which open connections are dropped. A second signal exits immediately. Connections that do not finish their request head within 10 s are closed (slowloris guard).

The server logs method, path, status and latency of every request (never bodies); 429 and 504 are logged at warn.

## API

| Method and path | Body / query | Success |
|---|---|---|
| `GET /health` | | `{"status":"ok"}` |
| `GET /v1/scramble?moves=25&size=3` | moves 1..=200, size 2, 3, 4 or 5 | `{"scramble":"R U ..."}` |
| `GET /v1/random?size=3` | size 2..=7 (only 2 to 5 have engines) | `{"kind":"3x3x3","facelets":"..."}` |
| `POST /v1/convert` | `{"cube","scheme"?}` | `{"kind","facelets","token"?}` |
| `POST /v1/solve` | `{"cube","scheme"?,"maxMoves"? (1..=25; 1..=255 for a 4x4x4 or 5x5x5),"budgetMs"? (1..=600000)}` | `{"kind","solution","length","elapsedMicros"}` |

`cube` accepts Orbit64 tokens (every size, 2x2x2 to 7x7x7; `convert` answers the token too), URFDLB facelets (spaced or compact) or W/R/G/Y/O/B colours; `scheme` is `boy`, `jp` or six letters.

Errors are `{"error":{"code","message"}}`:

| Status | code | Cause |
|---|---|---|
| 400 | `bad_input` | unparseable cube, bad JSON, unknown field or query parameter, out-of-range value (`size` outside 2..=7, `maxMoves`, `budgetMs`) |
| 405 | `method_not_allowed` | wrong method (with an `Allow` header) |
| 404 | `not_found` | unknown path |
| 415 | `unsupported_media_type` | POST without `Content-Type: application/json` |
| 413 | `payload_too_large` | body above `--max-body-bytes` |
| 422 | `impossible_cube` | unreachable cube state |
| 422 | `no_solution_within_limit` | nothing within `maxMoves` |
| 429 | `busy` | all solver slots in use (`Retry-After: 1`) |
| 500 | `internal` | a handler or solver thread panicked |
| 501 | `no_engine` | `solve` of a valid cube size (4, 6 or 7) without an engine, as facelets or as an Orbit64 token |
| 504 | `budget_exhausted` | time budget ran out (not 408: the client was not slow, the server was). Do not retry unchanged: retry only with a larger `budgetMs` or an easier input |

```sh
curl -s localhost:8080/health
curl -s 'localhost:8080/v1/scramble?moves=20'
curl -s 'localhost:8080/v1/random?size=2'
curl -s -XPOST localhost:8080/v1/convert -H 'Content-Type: application/json' -d '{"cube":"A4ib8aejxj4Y"}'
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' -d '{"cube":"A4ib8aejxj4Y","maxMoves":21,"budgetMs":2000}'
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' -d '{"cube":"CFJnI"}'
```

## Browsers and CORS

There is deliberately no CORS layer, and JSON-only bodies (415 otherwise) keep cross-site "simple requests" from triggering solves. A browser frontend will need an explicit allow-list CORS layer in a later round (never `*` together with credentials).

## Tests

The server tests are not run by a plain `cargo test` at the repository root. Use `cargo test -p cubacle-server --release` or `cargo test --workspace --release`.
