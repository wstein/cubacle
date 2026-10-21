---
title: "REST API reference"
description: "Reference for the cubacle-server REST API."
weight: 4
---

# REST API reference

`cubacle-server` is a stateless HTTP backend for the 2x2x2 and 3x3x3 solvers. It keeps no sessions and no per-client state, so you can run several instances behind a load balancer. All bodies are JSON.

## Run the server

```sh
cargo run --release -p cubacle-server -- --bind 127.0.0.1:8080
```

The server builds its engine tables once at startup, before it accepts connections.

| Flag | Environment variable | Default |
| --- | --- | --- |
| `--bind` | `CUBACLE_BIND` | `127.0.0.1:8080` (local only) |
| `--max-concurrency` | `CUBACLE_MAX_CONCURRENCY` | Number of CPUs. Range 1 to 256. |
| `--request-timeout-ms` | `CUBACLE_REQUEST_TIMEOUT_MS` | 30000. Range 1 to 600000. Cap and default of `budgetMs`. |
| `--max-body-bytes` | `CUBACLE_MAX_BODY_BYTES` | 16384 |
| `--log` | `CUBACLE_LOG` | `info` |

The server binds to the local machine by default. It has no authentication, so do not expose it to untrusted networks.

## Endpoints

| Method and path | Input | Success response |
| --- | --- | --- |
| `GET /health` | None | `{"status":"ok"}` |
| `GET /v1/scramble?moves=25` | `moves`: 1 to 200 | `{"scramble":"U' L2 F2 D U2"}` |
| `GET /v1/random?size=3` | `size`: 2 or 3 | `{"kind":"2x2x2","facelets":"..."}` |
| `POST /v1/convert` | `{"cube","scheme"?}` | `{"kind","facelets"}` |
| `POST /v1/solve` | `{"cube","scheme"?,"maxMoves"?,"budgetMs"?}` | `{"kind","solution","length","elapsedMicros"}` |

For `POST` requests, send the header `Content-Type: application/json`.

### Request fields

| Field | Meaning |
| --- | --- |
| `cube` | The cube as an Orbit64 token, URFDLB facelets (spaced or compact) or W/R/G/Y/O/B colours. See [Input notations](cli.md#input-notations). |
| `scheme` | Optional. `western-boy`, `japanese-bl-wt` or six letters in URFDLB order. |
| `maxMoves` | Optional. Longest accepted solution, 1 to 25. |
| `budgetMs` | Optional. Time limit in milliseconds, 1 to 600000. Defaults to, and is capped by, `--request-timeout-ms`. |

Unknown fields are rejected with a 400 error.

### Examples

```sh
curl -s localhost:8080/health
curl -s 'localhost:8080/v1/scramble?moves=20'
curl -s 'localhost:8080/v1/random?size=2'
curl -s -XPOST localhost:8080/v1/convert \
  -H 'Content-Type: application/json' -d '{"cube":"A4ib8aejxj4Y"}'
curl -s -XPOST localhost:8080/v1/solve \
  -H 'Content-Type: application/json' \
  -d '{"cube":"A4ib8aejxj4Y","maxMoves":21,"budgetMs":2000}'
```

The solve request returns:

```json
{"kind":"3x3x3","solution":"B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R","length":21,"elapsedMicros":389}
```

The `elapsedMicros` value varies between runs.

## Errors

Errors have this shape:

```json
{"error":{"code":"bad_input","message":"size must be between 2 and 7"}}
```

| Status | Code | Cause |
| --- | --- | --- |
| 400 | `bad_input` | Unparseable cube, bad JSON, unknown field or query parameter, or an out-of-range value. |
| 404 | `not_found` | Unknown path. |
| 405 | `method_not_allowed` | Wrong method. The response has an `Allow` header. |
| 413 | `payload_too_large` | Body larger than `--max-body-bytes`. |
| 415 | `unsupported_media_type` | `POST` without `Content-Type: application/json`. |
| 422 | `impossible_cube` | The cube state cannot be reached by legal moves. |
| 422 | `no_solution_within_limit` | No solution within `maxMoves`. |
| 429 | `busy` | All solver slots are in use. The response has `Retry-After: 1`. |
| 500 | `internal` | A handler or solver thread panicked. |
| 501 | `no_engine` | A valid cube size (4 to 7) that has no engine yet. |
| 504 | `budget_exhausted` | The time budget ran out. |

Some details:

- `GET /v1/random?size=4` returns 501, and `size=9` returns 400, because `size` accepts 2 to 7 but only 2 and 3 have engines.
- The server answers 504 rather than 408, because the client was not slow; the server was. Do not retry the same request. Retry with a larger `budgetMs` or an easier cube.
- On 429, wait and retry. The limit is `--max-concurrency` solves at once.

## Shutdown

On SIGINT or SIGTERM, the server shuts down gracefully. In-flight requests can finish, for at most the request timeout plus 5 seconds. After that, open connections are dropped. A second signal exits immediately. Connections that do not finish their request head within 10 seconds are closed.

## Logging

The server logs the method, path, status and latency of every request. It never logs bodies. Status 429 and 504 are logged at warning level.

## Browsers and CORS

The server has deliberately no CORS layer, and it requires JSON bodies, so cross-site simple requests cannot trigger solves. A browser frontend needs an explicit allow-list CORS layer, which is not available yet.
