---
title: "Engines and limits"
description: "Supported cube sizes, engines and limits."
weight: 5
---

# Engines and limits

An engine solves one cube size. cubacle picks the engine from the size of the cube you give it. Each engine has its own page with the same sections: method, input contract, options, tables, performance, comparison with the Java original, limits and how to reproduce the numbers. Cross-engine measurements are on [Performance](../performance.md).

| Size | Page | Algorithm | Moves (mean) | Solve time, 1 thread | Tables and memory | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 2x2x2 | [2x2x2 engine](2x2.md) | Exact distance table, optimal in the half-turn metric | 8.76 (optimal) | 0.002 ms | 1.8 MB, built in 64 ms, peak 18.7 MB | Supported |
| 3x3x3 | [3x3x3 engine](3x3.md) | Kociemba two-phase (min2phase) | 20.81 | 0.79 ms | built in 18 ms, peak 2.8 MB | Supported |
| 4x4x4 | [4x4x4 engine](4x4.md) | Three-phase reduction, then the 3x3x3 solver | 47.47 with the final rotation | 72 ms | 12 MB, 0.37 s, peak 65 MB | Supported |
| 5x5x5 | [5x5x5 engine](5x5.md) | Five-phase reduction, then the 3x3x3 solver | 73.1 | 444 ms | 237 MB cached, 18.3 s cold, peak 327 MB | Supported |
| 6x6x6, 7x7x7 | | Reduction | | | | Planned, not yet supported |

The 4x4x4 and 5x5x5 figures are from the benchmark runs recorded on their pages (Apple M2 Pro, release build); the 2x2x2 and 3x3x3 figures and all later figures (warm Java, core utilisation, thread scaling) are from the measurement on [Performance](../performance.md). The 4x4x4 and 5x5x5 times here are the first runs; the later run measured 72 / 41.6 ms and 408 / 100 ms at 1 / 12 threads.

## 6x6x6, 7x7x7: planned

The input notations and the API already accept cubes of these sizes. No engine exists, so a solve request fails:

- The command-line tool exits with a non-zero status.
- The REST API answers 501 with the code `no_engine`.

The planned approach is reduction, as for the 4x4x4 and 5x5x5. The 6x6x6 also has no fixed centres and needs parity handling. See [Architecture](../architecture.md).

## Limits

| Limit | Value |
| --- | --- |
| Supported sizes | 2x2x2, 3x3x3, 4x4x4 and 5x5x5 |
| Default move limit | 21 for the 2x2x2 and 3x3x3 (CLI), none for the 4x4x4 and 5x5x5 |
| REST `maxMoves` | 1 to 25, or 1 to 255 for the 4x4x4 and 5x5x5 |
| REST `budgetMs` | 1 to 600000 |
| REST scramble length | 1 to 200 moves |
