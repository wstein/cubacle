# cubacle-server

Stateless HTTP backend for the cubacle solvers (2x2x2 and 3x3x3). No sessions and no per-client state, so several instances can run behind a load balancer. Engine tables are built once at startup, before the listener accepts connections. Solving runs on blocking threads guarded by a semaphore (`--max-concurrency`); when all slots are busy the server answers 429 with `Retry-After`.

```sh
cargo run --release -p cubacle-server -- --bind 127.0.0.1:8080
```

| Flag | Env | Default |
|---|---|---|
| `--bind` | `CUBACLE_BIND` | `127.0.0.1:8080` (local only) |
| `--max-concurrency` | `CUBACLE_MAX_CONCURRENCY` | number of CPUs |
| `--request-timeout-ms` | `CUBACLE_REQUEST_TIMEOUT_MS` | 30000 (cap and default of `budgetMs`) |
| `--max-body-bytes` | `CUBACLE_MAX_BODY_BYTES` | 16384 |
| `--log` | `CUBACLE_LOG` | `info` |

SIGINT/SIGTERM trigger a graceful shutdown that lets in-flight requests finish.

## API

| Method and path | Body / query | Success |
|---|---|---|
| `GET /health` | | `{"status":"ok"}` |
| `GET /v1/scramble?moves=25` | moves 1..=200 | `{"scramble":"R U ..."}` |
| `GET /v1/random?size=3` | size 2 or 3 | `{"kind":"3x3x3","facelets":"..."}` |
| `POST /v1/convert` | `{"cube","scheme"?}` | `{"kind","facelets"}` |
| `POST /v1/solve` | `{"cube","scheme"?,"maxMoves"?,"budgetMs"?}` | `{"kind","solution","length","elapsedMicros"}` |

`cube` accepts Orbit64 tokens, URFDLB facelets (spaced or compact) or W/R/G/Y/O/B colours; `scheme` is `western-boy`, `japanese-bl-wt` or six letters.

Errors are `{"error":{"code","message"}}`:

| Status | code | Cause |
|---|---|---|
| 400 | `bad_input` | unparseable cube, bad JSON or parameter |
| 404 | `not_found` | unknown path |
| 413 | `payload_too_large` | body above `--max-body-bytes` |
| 422 | `impossible_cube` | unreachable cube state |
| 422 | `no_solution_within_limit` | nothing within `maxMoves` |
| 429 | `busy` | all solver slots in use (`Retry-After: 1`) |
| 501 | `no_engine` | size other than 2 or 3 |
| 504 | `budget_exhausted` | time budget ran out (not 408: the client was not slow, the server was) |

```sh
curl -s localhost:8080/health
curl -s 'localhost:8080/v1/scramble?moves=20'
curl -s 'localhost:8080/v1/random?size=2'
curl -s -XPOST localhost:8080/v1/convert -d '{"cube":"A4ib8aejxj4Y"}'
curl -s -XPOST localhost:8080/v1/solve -d '{"cube":"A4ib8aejxj4Y","maxMoves":21,"budgetMs":2000}'
curl -s -XPOST localhost:8080/v1/solve -d '{"cube":"CFJnI"}'
```
