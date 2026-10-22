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
| `GET /v1/scramble?moves=25&size=3` | `moves`: 1 to 200; `size`: 2, 3 or 5 | `{"scramble":"U' L2 F2 D U2"}` |
| `GET /v1/random?size=3` | `size`: 2 to 7 (only 2 to 5 have engines) | `{"kind":"2x2x2","facelets":"...","token":"..."}` |
| `POST /v1/convert` | `{"cube","scheme"?}` | `{"kind","facelets","token"?}` |
| `POST /v1/solve` | `{"cube","scheme"?,"maxMoves"?,"budgetMs"?,"minProbes"?,"improve"?,"slices"?}` | `{"kind","solution","length","elapsedMicros"}` |

For `POST` requests, send the header `Content-Type: application/json`.

### Request fields

| Field | Meaning |
| --- | --- |
| `cube` | The cube as an Orbit64 token, URFDLB facelets (spaced or compact) or W/R/G/Y/O/B colours. See [Input notations](cli.md#input-notations). |
| `scheme` | Optional. `boy`, `jp` or six letters in URFDLB order. |
| `maxMoves` | Optional. Longest accepted solution, 1 to 25 (1 to 255 for a 4x4x4 or 5x5x5, whose solutions have about 48 and 73 moves). |
| `slices` | Optional, default `false`. For a 4x4x4 or 5x5x5: allow single inner layers (`2R`) in the answer where they make it shorter. Without it the answer uses outer layers and two-layer turns only (a 5x5x5 also second layers). Ignored for other sizes. See [the move metric](performance.md#move-metric). |
| `budgetMs` | Optional. Time limit in milliseconds, 1 to 600000. Defaults to, and is capped by, `--request-timeout-ms`. |
| `minProbes` | Optional, 3x3x3 only. Keep improving the solution until this many probes were made, 1 to 100000000. Counts work, not time, so the answer is the same on every machine. See [3x3x3 performance](engines/3x3.md). |
| `improve` | Optional, 3x3x3 only. Keep improving until `budgetMs` is spent and return the best solution found; `budgetMs` is required and `minProbes` cannot be combined with it. Spending the budget is not an error once a solution exists. |

Other sizes answer 400 to `minProbes` and `improve`.

Unknown fields are rejected with a 400 error.

### Examples

```sh
curl -s localhost:8080/health
curl -s 'localhost:8080/v1/scramble?moves=20'
curl -s 'localhost:8080/v1/random?size=2'
curl -s 'localhost:8080/v1/random?size=5'
curl -s 'localhost:8080/v1/scramble?size=5&moves=8'
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

`convert` also answers the Orbit64 token of the cube (`token`), left out when the stickers are not a cube. Tokens of every size from 2x2x2 to 7x7x7 are accepted. A 4x4x4 token converts and solves. A 6x6x6 or 7x7x7 token converts, and `solve` of it answers 501 `no_engine`:

```sh
curl -s -XPOST localhost:8080/v1/convert \
  -H 'Content-Type: application/json' -d '{"cube":"BJStkD4cayBhsWj6eHgDZLTP1th"}'
curl -s -XPOST localhost:8080/v1/solve \
  -H 'Content-Type: application/json' -d '{"cube":"BJStkD4cayBhsWj6eHgDZLTP1th"}'
```

```json
{"kind":"4x4x4","facelets":"DLLDLLDLBFLFLRBRDRDLFBRLUURUUDDUFUUFLDFDRBUFURRLLBDFFFDFFFBRFBFRRDDDBLUUBURBRULBBFBBUDRURBLLBRDU","token":"BJStkD4cayBhsWj6eHgDZLTP1th"}
{"kind":"4x4x4","solution":"D' Uw Rw' Fw' U' F' Rw' L' D' R Uw' R2 U F' B U2 R' Uw2 F Rw2 B' Fw2 D2 Uw2 R Uw2 D' R' L B L2 U2 F D' B' R2 U' B2 D2 L2 D2 R2 F2 U2 R' Rw Lw' Fw' Bw","length":49,"elapsedMicros":71955}
```

A 4x4x4 solution has about 48 moves. Its last wide turns (`Rw Lw'` and the like, at most four) turn the whole cube back to the standard orientation, because a 4x4x4 has no fixed centres; the cube is solved exactly after them. The tables (12 MB) are built at startup, and each solve uses one thread, so concurrent requests do not compete for cores.

A 6x6x6 or 7x7x7 is converted, but `solve` of it answers 501 `no_engine`.

A 5x5x5 is sent as its 150 facelets or its 43-character token. This request solves the cube of a scramble:

```sh
CUBE=$(cubacle facelets "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F" --size 5)
curl -s -XPOST localhost:8080/v1/solve \
  -H 'Content-Type: application/json' \
  -d "{\"cube\":\"$CUBE\",\"budgetMs\":5000}"
```

```json
{"kind":"5x5x5","solution":"Fw' D F' Rw U B' Uw' Lw' U Bw ...","length":67,"elapsedMicros":28246}
```

The solution is shortened here; it has 67 moves; the time is that of the command-line run of the same cube with warm tables and will differ.

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
| 500 | `internal` | A handler or solver thread panicked, or an engine reached a state it cannot explain. |
| 501 | `no_engine` | `solve` of a valid cube size (6 or 7) that has no engine yet, whether sent as facelets or as an Orbit64 token. `convert` works for those sizes. |
| 504 | `budget_exhausted` | The time budget ran out. |

Some details:

- `GET /v1/random?size=6` returns 501, and `size=9` returns 400, because `size` accepts 2 to 7 but only 2 to 5 have engines. `GET /v1/scramble` takes the same sizes.
- A 4x4x4 solve with `maxMoves` runs the 3x3x3 stage for at most one second, whatever `budgetMs` is: the reduction is fixed, so a limit that no tail can meet answers 422 `no_solution_within_limit` after about a second, not 504 after the whole budget.
- A 5x5x5 solve request can be sent with the cube as 150 facelets. The server builds the 5x5x5 and 4x4x4 tables at startup, so the first request is not slower. Throughput and latency under load, and the thread defaults of the server (one thread per 4x4x4 solve, all cores per 5x5x5 solve), are measured on [Performance](performance.md#server-under-load).
- The server answers 504 rather than 408, because the client was not slow; the server was. Do not retry the same request. Retry with a larger `budgetMs` or an easier cube.
- On 429, wait and retry. The limit is `--max-concurrency` solves at once.

## Shutdown

On SIGINT or SIGTERM, the server shuts down gracefully. In-flight requests can finish, for at most the request timeout plus 5 seconds. After that, open connections are dropped. A second signal exits immediately. Connections that do not finish their request head within 10 seconds are closed.

## Logging

The server logs the method, path, status and latency of every request. It never logs bodies. Status 429 and 504 are logged at warning level.

## Browsers and CORS

The server has deliberately no CORS layer, and it requires JSON bodies, so cross-site simple requests cannot trigger solves. A browser frontend needs an explicit allow-list CORS layer, which is not available yet.
