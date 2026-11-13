---
title: "Browser WebAssembly build"
description: "The cubacle-wasm facade for the 2x2x2 and 3x3x3, and the G1 measurement of its size, time and memory."
weight: 9
---

# Browser WebAssembly build

`crates/wasm` builds `cubacle-wasm`, a WebAssembly module that solves the 2x2x2 and the 3x3x3 in a browser, a worker or Node. It is the measurement behind gate G1 of issue [#21](https://github.com/wstein/cubacle/issues/21). It is a measurement, not a release: nothing here is published, and the module has no loading, cancel or pinning layer (issues [#24](https://github.com/wstein/cubacle/issues/24) and [#25](https://github.com/wstein/cubacle/issues/25)).

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo build --release -p cubacle-wasm --target wasm32-unknown-unknown
```

The module is `target/wasm32-unknown-unknown/release/cubacle_wasm.wasm`. It is built without the `fetch` feature, so the network dependency (`ureq`) is not built. `.cargo/config.toml` sets the `getrandom` custom backend for this target.

## Interface

There is no JavaScript glue. The module exports a C ABI over its linear memory (`memory`):

| Export | Meaning |
| --- | --- |
| `cubacle_abi_version() -> u32` | ABI version of the module. 2 is this table; ABI 1 is the same table without `cubacle_solve_budget`. |
| `cubacle_alloc(len) -> ptr` | A buffer of `len` bytes for the host to fill. |
| `cubacle_solve(size, ptr, len, max_moves) -> result` | Solves the scramble (UTF-8, standard notation) on a 2x2x2 or 3x3x3. |
| `cubacle_solve_budget(size, ptr, len, max_moves, budget_ms) -> result` | ABI 2. Like `cubacle_solve` with a wall-clock budget in milliseconds; 0 means the 2000 ms default. |
| `cubacle_result_len(result) -> len` | The byte length of the result buffer. |
| `cubacle_free(ptr, len)` | Releases a buffer from `cubacle_alloc` or `cubacle_solve`. |

A result is four bytes of total length (little endian), one status byte (0 solved, 1 bad input, 2 no solution, 3 timeout) and the UTF-8 text: the solution, or the reason. The module imports two functions from `env`: `cubacle_host_now() -> f64` (milliseconds, for example `performance.now()`), because `std::time::Instant` panics on this target, and `cubacle_host_random(ptr, len)`, which fills a buffer with random bytes. The tables of the engines are built in memory on the first solve of each size; no table file is loaded.

Budget. `cubacle_solve` (ABI 1, unchanged signature) runs with a fixed budget of 2000 ms (`DEFAULT_BUDGET_MS`); a search without a solution when it runs out returns status 2, as it does when `max_moves` admits no solution. On the 3x3x3 ABI 1 does not tell the two cases apart (both return status 2 with the same engine text); `cubacle_solve_budget` does. `cubacle_solve_budget` (ABI 2) takes the budget in milliseconds and returns status 3 when it runs out (status 2 stays for a `max_moves` that admits no solution); a call whose budget suffices answers exactly as `cubacle_solve` does. The clock is `cubacle_host_now`. The new function has its own name because a changed signature of `cubacle_solve` would break ABI 1 hosts without a link error. There is no cancel, progress or streaming call, and a solution is not a proof of optimality.

## G1 measurement

Commands, from the repository root:

```sh
cargo build --release -p cubacle-wasm --target wasm32-unknown-unknown
node tools/wasm/bench.mjs target/wasm32-unknown-unknown/release/cubacle_wasm.wasm
```

Setup: the code of this pull request rebased onto `develop` `492e592` (the measurement ran on this revision; the commits after it change only documentation), rustc 1.99.1, release profile (`lto=fat`, `codegen-units=1`, no `wasm-opt`), Node v22.23.4, aarch64 Linux virtual machine with 10 cores, shared with other work. Three runs of the command; the ranges are the lowest and highest of the three medians. The input is the 24 fixed scrambles of `tools/wasm/golden.txt` (12 for each size, 25 random moves for the 3x3x3, 11 for the 2x2x2 from a seeded generator, Python `random.Random(20261106)`, no two consecutive moves on one axis; the generator is not in the repository). Every answer equals the golden file; the golden file equals the native answers, which `cargo test -p cubacle-wasm` replays to the solved cube.

| Quantity | Result |
| --- | --- |
| `.wasm` size | 162,079 bytes raw, 61,134 gzip (level 9), 50,241 brotli (quality 11) |
| 3x3x3, cold: first solve in a new instance (builds the tables) | 32.6 to 33.1 ms median of 7, 55 to 56 ms maximum |
| 2x2x2, cold: first solve in a new instance (builds the tables) | 92.7 to 94.2 ms median of 7, 112 to 115 ms maximum |
| Instantiate (module already compiled) | 0.02 ms (3x3x3 runs) and 0.05 to 0.06 ms (2x2x2 runs) |
| 3x3x3, warm, per solve (240 solves) | median 0.199 to 0.203 ms, p90 0.473 to 0.477 ms, maximum 0.60 to 0.62 ms; mean 20.83 moves |
| 2x2x2, warm, per solve (240 solves) | median 0.004 to 0.005 ms, p90 0.005 ms, maximum 0.10 to 0.12 ms; mean 8.83 moves (optimal) |
| Linear memory | 1,114,112 bytes after instantiate; 2,228,224 after the first 3x3x3 solve; 21,364,736 after the first 2x2x2 solve (also the final size) |

Linear memory never shrinks, so the last figure is the peak of the module for these calls (about 20.4 MiB, 2x2x2 tables included). The resident set of the Node process was not measured.

Native, for reference (from [Performance](performance.md), another machine, 1000 random cubes): 3x3x3 mean 0.482 ms; table build 41 ms (3x3x3) and 64 ms (2x2x2). The two runs differ in machine, scrambles and the sample, so they do not give a speed ratio.

## Browser run

Commands, from the repository root (`playwright-core` is not a dependency of this repository; the path is that of an installed copy):

```sh
node tools/wasm/browser.mjs target/wasm32-unknown-unknown/release/cubacle_wasm.wasm <path to playwright-core>
```

Setup: the module built from revision `6ab5dba` of `develop` (tree equal to the former `1ac33d5`; 162,079 bytes, the same size as above), Playwright 1.64.1, headless Chromium 156.0.8078.4, the same machine as above. The page is served from `127.0.0.1` with cross-origin isolation (timer resolution about 5 microseconds). Each of the 7 runs uses a new browser context; the module is fetched with `WebAssembly.compileStreaming`. Three runs of the command; the ranges are the lowest and highest of the three medians. Every answer equals the golden file.

| Quantity | Result |
| --- | --- |
| Compile (`compileStreaming`, fetch from localhost included) | 1.54 to 1.74 ms median of 7 |
| Instantiate | 0.04 ms |
| 3x3x3, first solve in a new instance (builds the tables) | 53.6 to 54.8 ms median of 7, 55.1 to 56.3 ms maximum |
| 3x3x3, compile + instantiate + first solve | 55.4 to 56.5 ms median of 7 |
| 2x2x2, first solve (after the 3x3x3 solves of the same instance) | 108.5 to 110.6 ms median of 7, 113.2 to 114.6 ms maximum |
| 3x3x3, warm, per solve (840 solves) | median 0.205 to 0.21 ms, p90 0.49 to 0.5 ms, maximum 0.75 to 0.84 ms; batch mean 0.250 to 0.254 ms |
| 2x2x2, warm, per solve (840 solves) | median 0.005 ms, p90 0.01 ms, maximum 0.025 to 0.03 ms; batch mean 0.006 to 0.007 ms |
| Linear memory | 1,114,112 bytes after instantiate; 2,228,224 after the first 3x3x3 solve; 21,364,736 after the first 2x2x2 solve (also the final size); the same figures as in Node |

The first 3x3x3 solve takes about 21 ms longer in Chromium than in Node (53.6 to 54.8 ms against 32.6 to 33.1 ms); the cause was not investigated. The resident set of the browser process was not measured: `performance.measureUserAgentSpecificMemory()` is refused (`SecurityError`) on this page and a sum of the resident set over the browser processes would count shared pages more than once (not attempted). Compile time with a warm code cache, worker start and the memory limits of other browsers are unmeasured.

## Comparison with the current Cubing.js bundles

The consuming project is [Cubistry](https://github.com/wstein/cubistry). Revision `fb19f8f17b6d28c359280ddfa285c9585d8c8ac2` (`origin/main`). Revision `09d865a`, 29 commits earlier, did not build in the same way (Vite 8 reported `manualChunks is not a function`, because its configuration uses the object form); the parent of `fb19f8f` was not built. Cubing.js 0.66.1. Build, in a scratch copy of `git archive`, with Node v22.23.4 (the project asks for Node 24 or newer; `npm` printed an engine warning and the build succeeded):

```sh
node scripts/prepare-puzlwerk.mjs && node scripts/prepare-puzlwerk.mjs --picture
npm ci --ignore-scripts
npm run build
node tools/wasm/bundle-size.mjs <scratch copy>/dist/assets '^calendar-solver-vendor-.*\.js$'
```

The build output has two Calendar vendor bundles (one per worker). Reading the installed Cubing.js 0.66.1 source (`node_modules/cubing/dist/lib/cubing/chunks`) shows two different solvers:

- 3x3x3: `solve333` calls a JavaScript solver, `solvePattern` of the chunk `search-dynamic-solve-3x3x3-*.js`, whose header names its source `src/cubing/vendor/mit/cs0x7f/min2phase/3x3x3-min2phase.ts`. It is plain JavaScript inside both vendor bundles (`solvePattern` is found in each). Its size inside a bundle is UNKNOWN, because the bundles are minified and have no boundary for it. The unbundled npm chunk is 64,588 bytes raw, 13,770 gzip, 12,223 brotli (a different, unminified form; `tools/wasm/bundle-size.mjs` on that directory).
- 2x2x2: `solve222` runs the twips WebAssembly module (`wasmTwips`, generators U, F, L, R). The module is not a separate file in the build: it is embedded as base64 text in each vendor bundle and measured here after decoding. It is loaded lazily on its first use.

| File | Raw | gzip (level 9) | brotli (quality 11) |
| --- | --- | --- | --- |
| Vendor bundle `calendar-solver-vendor-BkB0rmtE.js` | 1,233,958 | 337,874 | 277,564 |
| Vendor bundle `calendar-solver-vendor-CNbo6oql.js` | 1,256,504 | 344,378 | 282,937 |
| Both vendor bundles | 2,490,462 | 682,252 | 560,501 |
| Embedded twips module, used for the 2x2x2 only (identical in both, 670,540 base64 characters) | 502,904 | 152,781 | 126,646 |
| `cubacle_wasm.wasm` (for comparison) | 162,079 | 61,134 | 50,241 |

The cubacle module solves the 2x2x2 and the 3x3x3. Against the two parts of Cubing.js it stands as follows:

- (a) Against a whole vendor bundle, an upper bound for the Cubing.js code of one worker (the bundle holds more than the solvers): 13 percent by raw size (162,079 of 1,233,958 or 1,256,504 bytes), 18 percent by gzip and by brotli.
- (b) Against the twips module, which serves the 2x2x2 only: 32 percent raw, 40 percent by gzip and by brotli. The cubacle figure covers both sizes; the 3x3x3 part of Cubing.js is the JavaScript solver above, whose size inside a bundle is UNKNOWN, so no size ratio of the two 3x3x3 solvers is given.

Cubing.js offers more than a solve call (patterns and transformations as a KPuzzle interface); the cubacle module has none. The memory of Cubing.js was not measured (UNKNOWN).

Time and move count, from the 24 golden scrambles:

```sh
node tools/wasm/cubing-moves.mjs <scratch copy> 5
```

Cubing.js calls `experimentalSolve3x3x3IgnoringCenters` and `experimentalSolve2x2x2` in Node v22.23.4 (the library chooses its search and worker); 5 rounds, so 59 later calls per size; three runs, ranges over the three runs. Every solution of every round (120 per run), applied to its scramble, gives a solved cube with whole-cube rotation and center orientation ignored. The target definitions of the two solvers were not compared.

| Quantity | Cubing.js | cubacle (this module) |
| --- | --- | --- |
| 3x3x3, mean moves (12 scrambles) | 20.5 (20 moves: 6, 21 moves: 6) | 20.83 (20: 4, 21: 6, 22: 2) |
| 2x2x2, mean moves (12 scrambles) | 8.83 (8: 3, 9: 8, 10: 1) | 8.83 (the same split) |
| 3x3x3, first call or cold solve, Node | 119.8 to 121.9 ms (one call per run) | 32.6 to 33.1 ms median of 7 |
| 2x2x2, first call or cold solve, Node | 16.6 to 20.3 ms (one call per run, after the 3x3x3 calls) | 92.7 to 94.2 ms median of 7 |
| 3x3x3, later calls, Node | median 4.39 to 4.59 ms, p90 13.3 ms, maximum 14.3 to 14.6 ms | median 0.199 to 0.203 ms, p90 0.473 to 0.477 ms |
| 2x2x2, later calls, Node | median 2.44 to 2.47 ms, p90 3.8 to 3.9 ms, maximum 9.8 to 10.9 ms | median 0.004 to 0.005 ms, p90 0.005 ms |

The cubacle 3x3x3 answers are 0.33 moves longer on average than those of Cubing.js on these 12 scrambles. Cubing.js is faster on the first 2x2x2 call (16.6 to 20.3 ms against 92.7 to 94.2 ms): cubacle builds its tables on the first solve (about 93 ms in Node, 109 ms in Chromium). That Cubing.js call comes after the 3x3x3 calls, so its worker already runs; the lazy load of the twips module falls into this call. Cubing.js times include the library's own call overhead and, for the first call, its lazy loading; cubacle times are the `cubacle_solve` call only. This is a hint for gate G2, not a parity result.

## What this does not show

- The browser run uses one Chromium version in headless mode; Firefox and Safari are not measured.
- Compile time with a warm code cache and the caching of the module in a browser are unmeasured; the compile time above is a cold fetch from localhost.
- The resident set of a browser or Node process, for either side, is not measured. The memory of Cubing.js is not measured.
- The size comparison covers the two Calendar vendor bundles and the module embedded in them. The other files of the Calendar page (its entry, the workers and the shared chunks) are not part of it.
- Solvers beyond the 2x2x2 and 3x3x3 (4x4x4 and up) are not part of the module.
- The decision of gate G1 (continue or stop) is not taken here.
