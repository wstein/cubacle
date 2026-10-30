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
| `GET /v1/scramble?moves=25&size=3` | `moves`: 1 to 200; `size`: 2 to 7; `notation`?: `wca`, `sign` or `sse` | `{"scramble":"U' L2 F2 D U2"}` |
| `GET /v1/random?size=3` | `size`: 2 to 7 (only 2 to 5 have engines) | `{"kind":"2x2x2","facelets":"...","token":"...","canonical":{"token":"...","schema":"..."}}` |
| `POST /v1/convert` | `{"cube" or "scramble","size"?,"scheme"?}` | `{"kind","facelets","token"?,"canonical"?}` |
| `POST /v1/solve` | `{"cube" or "scramble","size"?,"scheme"?,"maxMoves"?,"budgetMs"?,"minProbes"?,"improve"?,"slices"?,"notation"?,"optimize"?,"target"?,"formatVersion"?}` | `{"kind","solution","inverse","length","metrics","compressed"?,"compressedLength"?,"compressedMetrics"?,"elapsedMicros","formatVersion","toolVersion","gitCommit","frame","proven","lowerBound"}` |

For `POST` requests, send the header `Content-Type: application/json`.

### Request fields

| Field | Meaning |
| --- | --- |
| `cube` | The cube as an Orbit64 token, URFDLB facelets (spaced or compact) or W/R/G/Y/O/B colors. See [Input notations](cli/input-notations.md#input-notations). |
| `scheme` | Optional. `boy`, `jp` or six letters in URFDLB order. |
| `maxMoves` | Optional. Longest accepted solution, 1 to 25 (1 to 255 for a 4x4x4 or 5x5x5, whose solutions have about 48 and 73 moves). |
| `slices` | Optional, default `false`. For a 4x4x4 or 5x5x5: allow single inner layers (`2R`) in the answer where they make it shorter. Without it the answer uses outer layers and two-layer turns only (a 5x5x5 also second layers). Ignored for other sizes. See [the move metric](performance.md#move-metric). |
| `budgetMs` | Optional. Time limit in milliseconds, 1 to 600000. Defaults to, and is capped by, `--request-timeout-ms`. |
| `minProbes` | Optional, 3x3x3 only. Keep improving the solution until this many probes were made, 1 to 100000000. Counts work, not time, so the answer is the same on every machine. See [3x3x3 performance](engines/3x3.md). |
| `improve` | Optional, 3x3x3 only. Keep improving until `budgetMs` is spent and return the best solution found; `budgetMs` is required and `minProbes` cannot be combined with it. Spending the budget is not an error once a solution exists. |
| `notation` | Optional, `wca` (default), `sign` or `sse`. How the `solution` is written: `Rw`, `3Rw` or `r`, `3r` or `TR`, `T3R` (see [Move notations](cli/move-notations.md#move-notations)). Also a query parameter of `/v1/scramble`. Anything else is a 400. |
| `optimize` | Optional, `ranges`. Adds `compressed` (the solution with layer ranges such as `2-3r`, in the chosen `notation`), `compressedLength` and `compressedMetrics` to the answer; `solution`, `length` and `metrics` are unchanged. Anything else is a 400. See [`--optimize`](cli.md). |

| `target` | Optional, `/v1/solve` only. Solve to this state instead of to the solved cube; the `solution` takes the start (`cube` or `scramble`) to it. How the server reads the text: if it is exactly `superflip`, `checkerboard` or `cube-in-cube` it is that pattern (3x3x3 only); otherwise, if it parses as a cube (an Orbit64 token, URFDLB facelets or colors, read with `scheme`), it is that cube; otherwise it is read as a move expression applied to the solved cube of the start's size. Cube text always wins over a move expression, as it does for the start on the command line. On a 4x4x4 to 7x7x7 a target whose centers of one kind show different colors on one face (built with inner slices or wide turns) is reached by a center fix-up, the last moves of the `solution`; `maxMoves` and `budgetMs` count the whole route. See [Solve to a target](cli/solve.md#solve-to-a-target) and [Targets with mixed centers](cli/solve.md#targets-with-mixed-centers). |

Other sizes answer 400 to `minProbes` and `improve`.

Unknown fields are rejected with a 400 error.

Instead of `cube`, `/v1/convert` and `/v1/solve` take a `scramble`: moves in WCA, SiGN or SSE spelling, with layer ranges (`2-3r`, `N2-4R`), groups (`(R U)3`, `(R U)'`, `(R U)*3`) and commutators and conjugates (`[A, B]`, `[A: B]`), exactly as the command line reads them (see [Move notations](cli/move-notations.md#move-notations)). `size` (2 to 7, default 3) says which cube the scramble is applied to. Send `cube` or `scramble`, not both; `size` goes only with `scramble` and `scheme` only with `cube`. A scramble that does not parse is a 400 whose message names the character, such as `scramble: character 7: a [ ] has one comma or colon; nest another [ ] for more`. `notation` chooses how the answer is written; it has the same moves and the same `length` in both.

A `target` is solved to like this (the first request is the `superflip` pattern, the second a scramble as target, the third a mismatched size):

```sh
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' \
  -d '{"scramble":"R U F2 D'"'"' L","target":"superflip"}'
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' \
  -d '{"scramble":"R U R'"'"' U'"'"'","target":"F2 B2"}'
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' \
  -d '{"scramble":"R U","target":"UUUU RRRR FFFF DDDD LLLL BBBB"}'
```

```json
{"kind":"3x3x3","solution":"L' U' F B' R U D' F L2 D2 F' U2 F' R2 L2 F' R2 F2 U2 L","inverse":"L' U2 F2 R2 F L2 R2 F U2 F D2 L2 F' D U' R' B F' U L","length":20,"metrics":{"htm":20,"qtm":28,"stm":20,"etm":20,"obtm":20,"btm":20,"atm":17},"elapsedMicros":521}
{"kind":"3x3x3","solution":"U R U R F2 R L' U2 R' B2 R' D2 R' L F2","inverse":"F2 L' R D2 R B2 R U2 L R' F2 R' U' R' U'","length":15,"metrics":{"htm":15,"qtm":20,"stm":15,"etm":15,"obtm":15,"btm":15,"atm":13},"elapsedMicros":22}
{"error":{"code":"bad_input","message":"the start is a 3x3x3 but the target is a 2x2x2"}}
```

A target of another size is a 400. A target with an impossible piece is a 422 `impossible_cube`; a state the engine finds impossible only while solving says `the start or the target state is impossible`. A target with the wrong center colors (a color on more or fewer than four places of an orbit, or centers no cube has) is a 422 `impossible_cube`; a middle-slice target of the 5x5x5 or 7x7x7 is reached too. A route that cannot be finished within `maxMoves` is a 422 `no_solution_within_limit`, one that the time budget ends a 504 `budget_exhausted`, as for any target. The 2x2x2 reaches the target up to a rotation of the whole cube. `/v1/convert` has no `target` and answers 400 to it, like any unknown field.

### Examples

```sh
curl -s localhost:8080/health
curl -s 'localhost:8080/v1/scramble?moves=20'
curl -s 'localhost:8080/v1/scramble?size=5&moves=8&notation=sign'
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
{"kind":"3x3x3","solution":"B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R","inverse":"R' F2 U L2 U L2 R2 D' L2 R2 F2 R2 D2 B' D' U L B' L' R' B'","length":21,"metrics":{"htm":21,"qtm":30,"stm":21,"etm":21,"obtm":21,"btm":21,"atm":17},"elapsedMicros":776}
```

The `elapsedMicros` value varies between runs. For the 4x4x4 to 7x7x7 the original fields include one more key after it, `phases`: an array of `{"name":"3x3 stage","moves":2}` objects, one per phase of the engine, in order (a phase can have `"moves":0`). The 2x2x2 and 3x3x3 engines report no phases, so their body has no `phases` key. The server has no option for the pattern hint, so it never sends `patternHint`, which `cubacle solve --json -v` can add.

The same body comes from `cubacle solve --json` for an equivalent request: both are written by one function of the library, `cubacle::report`. See [`--json`](cli/solve-options.md#--json).

### Move counts

Every solve answer also carries `inverse`, next to `solution`: the moves that undo `solution`, written in the same `notation` (reversed, each turned the other way). Applied to the solved cube, or to the `target` when one was given, they build the start cube. The field is always present; the other fields are unchanged. For the scramble `R U F D2 L' B R2 U' F2 D L2 B' U2 R F`:

```json
{"kind":"3x3x3","solution":"R' U2 D' R B' D L' U B U R L' D2 R' F2 B2 R U2 R' F2 U2","inverse":"U2 F2 R U2 R' B2 F2 R D2 L R' U' B' U' L D' B R' D U2 R","length":21,"metrics":{"htm":21,"qtm":28,"stm":21,"etm":21,"obtm":21,"btm":21,"atm":18},"elapsedMicros":2249}
```

(The `elapsedMicros` value is that of one run.)

Every solve answer carries `metrics`: the length of `solution` in all seven metrics of [Move metrics](metrics.md), as integers, always in the key order `htm`, `qtm`, `stm`, `etm`, `obtm`, `btm`, `atm`. The command line shows three or four of them on its `metrics:` line; the server always sends all seven, so a client picks what it needs and ignores the rest. The counts are those of `cubacle::metrics::count` on the returned moves (`length` is the solver's own count, `moves N` on the command line). The same object under `compressedMetrics` counts `compressed` when `optimize` asked for it. The counts are part of every answer and cannot be turned off.

The server has no endpoint for `solve --now`, so there is no route variant to count; `/v1/solve` is the only answer with moves.

The same request with `"optimize":"ranges"` (`{"cube":"A4ib8aejxj4Y","maxMoves":21,"budgetMs":2000,"optimize":"ranges"}`), run against the server:

```json
{"kind":"3x3x3","solution":"B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R","inverse":"R' F2 U L2 U L2 R2 D' L2 R2 F2 R2 D2 B' D' U L B' L' R' B'","length":21,"metrics":{"htm":21,"qtm":30,"stm":21,"etm":21,"obtm":21,"btm":21,"atm":17},"compressed":"B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R","compressedLength":21,"compressedMetrics":{"htm":21,"qtm":30,"stm":21,"etm":21,"obtm":21,"btm":21,"atm":17},"elapsedMicros":737}
```

Here the compression saves nothing, so the compressed counts equal the plain ones. Plain solve options are unchanged: the server uses the default `SolveOptions` (no probe minimum) unless the request sets `minProbes`, whereas the command line makes 3000 probes on a plain 3x3x3 solve.

`convert` also answers the Orbit64 token of the cube (`token`), left out when the stickers are not a cube. Tokens of every size from 2x2x2 to 7x7x7 are accepted. A 4x4x4 token converts and solves. A 6x6x6 or 7x7x7 token converts and solves:

```sh
curl -s -XPOST localhost:8080/v1/convert \
  -H 'Content-Type: application/json' -d '{"cube":"BJStkD4cayBhsWj6eHgDZLTP1th"}'
curl -s -XPOST localhost:8080/v1/solve \
  -H 'Content-Type: application/json' -d '{"cube":"BJStkD4cayBhsWj6eHgDZLTP1th"}'
```

```json
{"kind":"4x4x4","facelets":"DLLDLLDLBFLFLRBRDRDLFBRLUURUUDDUFUUFLDFDRBUFURRLLBDFFFDFFFBRFBFRRDDDBLUUBURBRULBBFBBUDRURBLLBRDU","token":"BJStkD4cayBhsWj6eHgDZLTP1th"}
{"kind":"4x4x4","solution":"D' Uw Rw' Bw' R' F' Uw' U' F' D Bw' D2 B R' L B2 D' Fw2 L Dw2 L' Lw2 F2 Bw2 D Fw2 F' U' D R D2 B2 L F' R' U2 B' R2 F2 D2 F2 U2 L2 B2 U'","inverse":"U B2 L2 U2 F2 D2 F2 R2 B U2 R F L' B2 D2 R' D' U F Fw2 D' Bw2 F2 Lw2 L Dw2 L' Fw2 D B2 L' R B' D2 Bw D' F U Uw F R Bw Rw Uw' D","length":45,"metrics":{"htm":45,"qtm":63,"stm":55,"etm":45,"obtm":45,"btm":45,"atm":38},"elapsedMicros":88735}
```

A 4x4x4 solution has about 48 moves. Its last wide turns (`Rw Lw'` and the like, at most four) turn the whole cube back to the standard orientation, because a 4x4x4 has no fixed centers; the cube is solved exactly after them. The tables (12 MB) are built at startup, and each solve uses one thread, so concurrent requests do not compete for cores.

The 6x6x6 and 7x7x7 are solved by reduction through the 4x4x4, 5x5x5 and 3x3x3 engines (see [the dwalton port](design/dwalton-port.md)). They ignore `maxMoves` limits below the reduction's length: the engine does not search for shorter answers. `budgetMs` ends a solve in progress: the center searches look at the clock every few thousand nodes and the wing stages get what is left, so a spent budget is a 504 `budget_exhausted`.

A 3x3x3, 5x5x5 or 7x7x7 whose face centers a slice turn moved is solved by the color of its centers, as on the command line: the answer leaves every face one color, the color of its center, which is not the color a solved cube shows on that face. A `target` still needs the centers in place (`centers_off_face`).

A 5x5x5 is sent as its 150 facelets or its 43-character token. This request solves the cube of a scramble:

```sh
CUBE=$(cubacle facelets "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F" --size 5)
curl -s -XPOST localhost:8080/v1/solve \
  -H 'Content-Type: application/json' \
  -d "{\"cube\":\"$CUBE\",\"budgetMs\":5000}"
```

```json
{"kind":"5x5x5","solution":"Fw' D F' Rw U B' Uw' Lw' U Bw ...","inverse":"...","length":67,"metrics":{...},"elapsedMicros":28246}
```

The `solution` is shortened here, and the `inverse` and `metrics` values are left out (`...`); it has 67 moves; the time is that of the command-line run of the same cube with warm tables and will differ.

### The canonical field

`/v1/convert` and `/v1/random` describe a cube with `kind`, `facelets` and `token`. They also carry `canonical`, an object with the canonical Orbit64 token of the cube (the same for the cube, its turned and mirrored images and any renaming of its colors) and `schema`, the name of the slot convention the token is written in (`orbit64-Nxn-draft@1`, N the size). Like `token`, it is absent when the stickers are not a cube. The other fields are unchanged. No separate endpoint was added: `/v1/convert` already takes every input form. The same request as `cubacle canon`, for a scramble and its mirror image:

```sh
curl -s -XPOST localhost:8080/v1/convert \
  -H 'Content-Type: application/json' -d '{"scramble":"R U"}'
curl -s -XPOST localhost:8080/v1/convert \
  -H 'Content-Type: application/json' -d '{"scramble":"L'"'"' U'"'"'"}'
```

```json
{"kind":"3x3x3","facelets":"UUUUUUFFFUBBRRRRRRRRRFFDFFDDDBDDBDDBFFDLLLLLLLLLUBBUBB","token":"AwtB4B1fWQAA","canonical":{"token":"C0RkZMybxEmw","schema":"orbit64-3x3-draft@1"}}
{"kind":"3x3x3","facelets":"UUUUUUFFFDFFRRRRRRLLLDFFDFFBDDBDDBDDBBULLLLLLRRRBBUBBU","token":"JIWvcIcwO0AA","canonical":{"token":"C0RkZMybxEmw","schema":"orbit64-3x3-draft@1"}}
```

The tokens differ, the canonical tokens are one. A stored identifier of a pattern is the canonical token together with the `schema`. See [Canonical token](cli/canon.md) for the command line.

### Solve JSON versions (changed in 0.2.0)

`POST /v1/solve` defaults to format 2 and appends `formatVersion`, `toolVersion`,
`gitCommit`, `frame`, `proven`, and `lowerBound` after all original keys (including
`phases` when present). Send `"formatVersion": 1` for the exact original body, or
`"formatVersion": 2` explicitly. Unknown or non-integer versions return 400 `bad_input`
before solver work. Other endpoint formats and error bodies are unchanged.

The goal frame and HTM proof metadata come from the actual result, not requested flags.
The ordinary 2x2 solver reports `"any-orientation"`; an unproved two-phase/reduction result
has `proven: false` and no established lower bound. `gitCommit` is null for builds without
release revision metadata. Both versions use the same library writer as the CLI; compare
only equivalent requests and mask `elapsedMicros` when checking byte equality. See the
[solve options](cli/solve-options.md#--json) and
[solve-result schema](static/schema/v1/solve-result.schema.json).

The earlier recorded solve transcripts on this page show format 1, preserving their original
solutions and timing measurements. Select `"formatVersion": 1` to reproduce that body shape;
a request without the selector now also carries the six format-2 keys.

## Errors

Errors have this shape:

```json
{"error":{"code":"bad_input","message":"size must be between 2 and 7"}}
```

| Status | Code | Cause |
| --- | --- | --- |
| 400 | `bad_input` | Unparsable cube, bad JSON, unknown field or query parameter, or an out-of-range value. |
| 404 | `not_found` | Unknown path. |
| 405 | `method_not_allowed` | Wrong method. The response has an `Allow` header. |
| 413 | `payload_too_large` | Body larger than `--max-body-bytes`. |
| 415 | `unsupported_media_type` | `POST` without `Content-Type: application/json`. |
| 422 | `impossible_cube` | The cube state cannot be reached by legal moves. |
| 422 | `centers_off_face` | A `target` was sent with a cube whose face centers a slice turn moved (3x3x3, 5x5x5, 7x7x7); without a `target` such a cube is solved. |
| 422 | `no_solution_within_limit` | No solution within `maxMoves`. |
| 429 | `busy` | All solver slots are in use. The response has `Retry-After: 1`. |
| 500 | `internal` | A handler or solver thread panicked, or an engine reached a state it cannot explain. |
| 504 | `budget_exhausted` | The time budget ran out (every size; the budget is `budgetMs` or the request timeout, so the server has a limit on all sizes). |

Some details:

- `size=9` returns 400, because `size` accepts 2 to 7. `GET /v1/scramble` takes the same sizes.
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

## Notation of the answer

A 7x7x7 solved with `"notation":"wca"` (the default) and `"notation":"sign"`, the same cube and the same 187 moves (the start and the end of `solution` are shown, the `metrics` object is left out):

```json
{"kind":"7x7x7","solution":"U 3Uw 3Lw' B' 3Rw 3Bw 3Uw 3Lw 3Bw2 3Uw Bw2 3Lw F2 ... U' L2 R","length":187}
{"kind":"7x7x7","solution":"U 3u 3l' B' 3r 3b 3u 3l 3b2 3u b2 3l F2 ... U' L2 R","length":187}
```

An unknown value answers 400:

```json
{"error":{"code":"bad_input","message":"notation must be wca, sign or sse, not \"ruwix\""}}
```

`sse` writes Superset ENG (`TR`, `T3R`, `NL`, `MR`, `N2-4R`; see [Move notations](cli/move-notations.md#move-notations)), and the `scramble` field reads it. Run against the server, with a random scramble in the first answer:

```sh
curl -s 'localhost:8080/v1/scramble?size=7&moves=8&notation=sse'
curl -s -X POST localhost:8080/v1/convert -H 'content-type: application/json' -d '{"scramble":"R N2R","size":4}'
```

```json
{"scramble":"N3L2 U2 NR' NB' ND' R ND2 NL'"}
{"error":{"code":"bad_input","message":"scramble: character 2: invalid move \"N2R\" (N twists exist from the 5x5 on; use M, W or T on smaller cubes)"}}
```
