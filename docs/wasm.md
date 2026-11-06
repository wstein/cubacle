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
| `cubacle_alloc(len) -> ptr` | A buffer of `len` bytes for the host to fill. |
| `cubacle_solve(size, ptr, len, max_moves) -> result` | Solves the scramble (UTF-8, standard notation) on a 2x2x2 or 3x3x3. |
| `cubacle_result_len(result) -> len` | The byte length of the result buffer. |
| `cubacle_free(ptr, len)` | Releases a buffer from `cubacle_alloc` or `cubacle_solve`. |

A result is four bytes of total length (little endian), one status byte (0 solved, 1 bad input, 2 no solution) and the UTF-8 text: the solution, or the reason. The module imports two functions from `env`: `cubacle_host_now() -> f64` (milliseconds, for example `performance.now()`), because `std::time::Instant` panics on this target, and `cubacle_host_random(ptr, len)`, which fills a buffer with random bytes. The tables of the engines are built in memory on the first solve of each size; no table file is loaded.

## G1 measurement

Commands, from the repository root:

```sh
cargo build --release -p cubacle-wasm --target wasm32-unknown-unknown
node tools/wasm/bench.mjs target/wasm32-unknown-unknown/release/cubacle_wasm.wasm
```

Setup: code at `80d554ad4724f7a04842f01e32a6335f111fd886` (`develop` `74fc155` plus the three commits of the pull request), rustc 1.99.1, release profile (`lto=fat`, `codegen-units=1`, no `wasm-opt`), Node v22.23.4, aarch64 Linux virtual machine with 10 cores, shared with other work. Three runs of the command; the ranges are the lowest and highest of the three medians. The input is the 24 fixed scrambles of `tools/wasm/golden.txt` (12 for each size, 25 random moves for the 3x3x3, 11 for the 2x2x2 from a seeded generator). Every answer equals the golden file; the golden file equals the native answers, which `cargo test -p cubacle-wasm` replays to the solved cube.

| Quantity | Result |
| --- | --- |
| `.wasm` size | 162,079 bytes raw, 61,134 gzip (level 9), 50,241 brotli (quality 11) |
| 3x3x3, cold: first solve in a new instance (builds the tables) | 34.2 ms median of 7, 55 to 58 ms maximum |
| 2x2x2, cold: first solve in a new instance (builds the tables) | 94.6 to 100.0 ms median of 7, 120 to 125 ms maximum |
| Instantiate | 0.04 to 0.1 ms |
| 3x3x3, warm, per solve (240 solves) | median 0.207 to 0.221 ms, p90 0.464 to 0.507 ms, maximum 0.61 to 0.67 ms; mean 20.83 moves |
| 2x2x2, warm, per solve (240 solves) | median 0.004 to 0.005 ms, p90 0.005 ms, maximum 0.11 to 0.13 ms; mean 8.83 moves (optimal) |
| Linear memory | 1,114,112 bytes after instantiate; 2,228,224 after the first 3x3x3 solve; 21,364,736 after the first 2x2x2 solve (also the final size) |

Linear memory never shrinks, so the last figure is the peak of the module for these calls (about 20.4 MiB, 2x2x2 tables included). The resident set of the Node process was not measured.

Native, for reference (from [Performance](performance.md), another machine, 1000 random cubes): 3x3x3 mean 0.482 ms; table build 41 ms (3x3x3) and 64 ms (2x2x2). The two runs differ in machine, scrambles and the sample, so they do not give a speed ratio.

## What this does not show

- The comparison with the current Calendar bundles of the consuming project is not made here: those bundles are not part of this repository. The size row is the input for that comparison.
- No browser was used; the module ran in Node. Browser timing resolution, worker start and memory limits are unmeasured.
- Solvers beyond the 2x2x2 and 3x3x3 (4x4x4 and up) are not part of the module.
- The decision of gate G1 (continue or stop) is not taken here.
