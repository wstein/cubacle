---
title: "Performance"
description: "Measured solve times, move counts, tables and memory of all four engines, and how to use the cores."
weight: 7
---

# Performance

All numbers come from measurements on one Apple M2 Pro (12 cores: 8 performance, 4 efficiency), release build. Method and raw tables: PENDING_PERF (cite the measurement report). Per-engine detail: [2x2x2](engines/2x2.md), [3x3x3](engines/3x3.md), [4x4x4](engines/4x4.md), [5x5x5](engines/5x5.md).

## All engines in one table

| | 2x2x2 | 3x3x3 | 4x4x4 | 5x5x5 |
| --- | --- | --- | --- | --- |
| Moves, mean / p90 / max | PENDING_PERF | PENDING_PERF | 47.47 / 49 / 51 (with rotation) | 73.1 / 75 / PENDING_PERF |
| Time per solve, 1 thread (mean / median / p90) | PENDING_PERF | PENDING_PERF | 72 / 62 / 117 ms | 444 / 404 / 723 ms |
| Time per solve, all cores, one solve | PENDING_PERF | PENDING_PERF | 39.5 / 38.1 / 53.4 ms (12 threads) | 92 / 77 / 187 ms (12 threads) |
| Core utilisation of one multithreaded solve | PENDING_PERF | PENDING_PERF | PENDING_PERF | PENDING_PERF |
| Throughput, solves per second, all cores, one solve per core | PENDING_PERF | PENDING_PERF | PENDING_PERF | PENDING_PERF |
| Table build, cold, 1 thread / all cores | PENDING_PERF | PENDING_PERF | 0.37 s / 0.17 s | 18.3 s / 3.3 s |
| Table load, warm | none (built each process) | none | none | about 1.0 s |
| Tables | 1.8 MB | PENDING_PERF | 12 MB | 237 MB |
| Peak memory | PENDING_PERF | PENDING_PERF | 65 MB (1 thread), 66 MB (12) | 327 MB (1), 335 MB (12) |
| Java original, same machine | none exists | PENDING_PERF | 115 / 99 / 197 ms, 44.59 moves | 865 / 656 / 1776 ms, 73.1 moves |

The 4x4x4 figures are from 1000 cubes and the 5x5x5 figures from 100 cubes of the harnesses in `tools/bench`; the other cells come from the measurement report.

## Two ways to use several cores

- **Multithreaded algorithm.** One solve uses several cores. `CUBACLE_THREADS` (environment) or `with_threads(n)` on `FourSolver` and `FiveSolver` sets the count; `Min2PhaseSolver::solve_parallel` searches the six URF orientations of one cube at once. The 2x2x2 has nothing to split. This lowers the latency of one answer.
- **Distributed mode.** Each solve is single-threaded and many solves run side by side: `Min2PhaseSolver::solve_batch` on its worker pool, or the REST server, which gives every request one thread and runs up to `--max-concurrency` requests at once (default: the number of cores). This raises the throughput.

## 100 solutions: one multithreaded solve at a time, or one solve per core?

Spreading single-thread solves over the cores is the more efficient way whenever you want many answers and not the lowest latency for one. Parallel parts of a solve have sequential parts (for the 4x4x4: phase 1 and the 3x3x3 hand-off are 23 ms of 73 ms, so four threads give 0.56 of the single-thread time, not 0.25), while independent solves have none. PENDING_PERF gives the measured time for 100 cubes in both modes per engine and the core utilisation; the conclusion above is the expected direction from the measured scaling and is confirmed or corrected there.

## Apple M2 Pro: performance and efficiency cores

PENDING_PERF: measured scaling at 1, 2, 4, 6, 8, 10 and 12 threads, the effect of the four efficiency cores, and the thread-count recommendation. Until then: for the multithreaded algorithm the 4x4x4 gains little between 4 and 12 threads (40.0 against 39.5 ms mean).

## Reproduce

PENDING_PERF: exact commands of the measurement report. Commands already documented: `tools/bench/run_four.sh /path/to/TPR-4x4x4-Solver /tmp/bench4 1000 1` (4x4x4) and `tools/bench/run.sh /path/to/cube555 /tmp/bench 100` (5x5x5); set `CUBACLE_THREADS=1` for single-thread numbers.
