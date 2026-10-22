---
title: "Performance"
description: "Measured solve times, move counts, tables and memory of all four engines, how to use the cores, and the planned optimisations."
weight: 7
---

# Performance

Per-engine detail: [2x2x2](engines/2x2.md), [3x3x3](engines/3x3.md), [4x4x4](engines/4x4.md), [5x5x5](engines/5x5.md).

## Method and marks

All numbers are from the measurement of 2026-10-22 on one Apple M2 Pro with 12 cores, **8 performance (P) and 4 efficiency (E) cores**, 16 GB, Rust release build (`lto=fat`, `codegen-units=1`), OpenJDK 21.0.12, code at `develop` `afee323`. Section numbers below, written as "report 2", refer to the sections of that measurement report (kept outside the repository, together with its scratch harness).

- **[M]** measured, **[D]** derived from measured numbers, **[E]** estimate. Estimates are never presented as measurements.
- A harness crate outside the repository wrapped a copy of the code with scratch counters (phase times, node counts, thread spawns, mutex wait). The instrumentation cost +0.6 % at 1 thread and +5 % at 12 threads (4x4, 200 cubes) [M].
- Tables are prepared first and 3 warm-up solves run before the timed loop (2x2: none). Cores busy = CPU time / wall time; utilisation = cores busy / threads.
- Exactly one measurement ran at a time, after a check that the machine was at least 90 % idle; two runs that failed the check were discarded and redone.
- Cubes: 2x2 and 3x3 1000 random states; 4x4 1000 cubes of the TPR generator (seed 1); 5x5 300 random states (subsets of 100 and 30).
- The Java solvers run one warm JVM each (3x3: 3 passes over 1000 cubes, the third reported; 4x4: 50 warm-up solves; 5x5: 5). They are single-threaded: no `Thread` or `Executor` appears in the three sources, and the measured cores busy were 1.00 to 1.01. Node counters were added at the same place as in Rust, so "same work" can be checked.

## All engines in one table [M unless marked]

Milliseconds per cube. 4x4 moves include the closing rotation; 5x5 includes the 3x3x3 tail.

| | 2x2x2 | 3x3x3 | 4x4x4 | 5x5x5 |
| --- | --- | --- | --- | --- |
| Moves, mean / median / p90 / max | 8.76 / 9 / 10 / 10 (optimal) | 20.81 / 21 / 21 / 21 | 47.47 / 48 / 49 / 51 (44.30 without rotation) | 72.90 / 73 / 75 / 77 |
| Time, 1 thread, mean / median / p90 | 0.002 / 0.002 / 0.003 | 0.788 / 0.404 / 1.690 | 72.2 / 62.6 / 116.8 | 408.4 / 362.5 / 666.5 |
| 2 threads, one solve, mean (cores busy, utilisation) | n/a | n/a | 52.1 (1.53, 77 %) | 239.5 (1.97, 99 %) |
| 4 threads | n/a | n/a | 41.7 (2.22, 56 %) | 142.4 (3.84, 96 %) |
| 8 threads | n/a | n/a | 40.6 (3.60, 45 %) | 89.8 (7.38, 92 %) |
| 12 threads, mean / median / p90 | n/a | n/a | 41.6 / 40.3 / 55.6 (5.13 cores, 43 %) | 100.2 / 79.1 / 158.9 (9.24 cores, 77 %) |
| CPU per cube, 1 thread / 12 threads | n/a | n/a | 72 / 214 ms (x3.0) | 408 / 927 ms (x2.3) |
| Table build, cold, 1 thread | 64 ms (95 ms first run) | 18 ms | 317 ms | 18.2 s |
| Table build, cold, 4 / 8 / 12 threads | n/a | n/a | 173 / 156 / 151 ms | 5.86 / 3.66 / 3.28 s |
| Table load, warm | none (built each process) | none | none (always built) | 0.97 to 1.0 s |
| Peak RSS, Rust | 18.7 MB | 2.8 MB | 62 to 64 MB (tables 12.1 MB) | 306 to 319 MB |
| **Java, 1 thread, warm**, mean / median / p90 / max | none exists | 0.567 / 0.349 / 1.165 / 12.9 | 114.1 / 98.0 / 196.1 / 657 | 575.7 / 511.1 / 921.9 / 2321 (reduction only) |
| Java moves | n/a | 20.52 / 21 / 21 / 21 | 44.59 without rotation (from the port notes, not re-measured) | reduction 51.32 plus the same 3x3x3 tail |
| Java init | n/a | 0.13 s | 0.80 s | 1.70 s from table files; cold build about 50 s (notes, not re-measured) |
| Peak RSS, Java | n/a | 68 MB | 129 MB | 657 MB |
| Rust against Java, 1 thread | n/a | 0.72x (Rust is slower) | 1.58x | 1.41x (0.55 ms hand-off subtracted) [D] |
| Rust against Java, 12-thread latency | n/a | n/a | 2.74x | 5.75x |
| Rust against Java, throughput, 12 solves in flight x 1 thread | n/a | 6.5x | 12.1x (106 / s against 8.8 / s) | 8.3x (14.4 / s against 1.74 / s) [D] |

The 5x5 rows for 2, 4 and 8 threads are from the 100-cube set (report 2: 1 thread 427.3 ms, 6 threads 114.9, 10 threads 88.7, 12 threads 87.6 ms, 10.5 cores, 88 %). The 5x5 Java figure is the warm mean of 576 ms; the older 865 ms mean in the first benchmark and in the 5x5 port page is a cold-start figure and is not used for comparisons. On the same cubes and work the speed-up of 5x5 at 12 threads is therefore 5.75x, not "9x".

Java does the same work: node counts of the Rust and Java searches agree within 3 to 8 % in 4x4 and 5x5 (report 7), so the single-thread speed-up (1.58x and 1.41x) is a per-node speed-up, and the rest comes from parallelism. The 3x3x3 is the exception: ours is an older min2phase variant without twist-flip pruning and with `MAX_DEPTH2` 13 (upstream: twist-flip pruning with conjugate pruning, `MAX_DEPTH2` 12). It searches 4.1x more phase 1 nodes and 1.7x more probes than upstream Java, and is 1.4x slower [M, report 7].

### 3x3x3 `solve_batch`, 1000 cubes [M]

| Pool threads | 1 | 2 | 4 | 6 | 8 | 10 | 12 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Wall time (ms) | 788 | 427 | 218 | 146 | 112 | 98 | 87.5 |
| Utilisation | 100 % | 99 % | 96 % | 93 % | 90 % | 89 % | 87 % |

That is 9.0x at 12 threads and 7.1x at 8. `solve_parallel` (the six URF searches of one cube on the pool) has no latency mode: without a time budget it did not finish one cube in 30 s, because it runs in improve mode and returns only at the deadline.

## Two ways to use several cores

- **Multithreaded algorithm.** One solve uses several cores. `CUBACLE_THREADS` (environment) or `with_threads(n)` on `FourSolver` and `FiveSolver` sets the count (default all cores). The 4x4x4 uses it in phases 2 and 3 only; phase 1 and the 3x3x3 hand-off are single-threaded. The 5x5x5 uses it in all five phases, the start-node generation and the table build. The 3x3x3 `solve` is always one thread; the 2x2x2 has nothing to split. This lowers the latency of one answer.
- **Distributed mode.** Each solve is single-threaded and many run side by side: `Min2PhaseSolver::solve_batch` on its process-wide pool (size `CUBACLE_THREADS`, read once), or the REST server, which runs up to `--max-concurrency` requests at once (default: the number of cores). This raises throughput.

## 100 solutions: multithreaded algorithm or single-core solves spread over cores?

Spreading single-thread solves over the cores. Measured with 100 cubes and one shared solver [M, report 3]:

| Size | In flight x threads | Wall | Solves / s | Cores busy | Median latency |
| --- | --- | --- | --- | --- | --- |
| 3x3x3 | 1 x 1 | 0.797 s | 1255 | 1.0 | 0.40 ms |
| 3x3x3 | 12 x 1 | 0.088 s | 11392 | 11.4 | 0.50 ms |
| 4x4x4 | 1 x 12 (one at a time, all cores) | 4.12 s | 24.3 | 5.5 | 40.8 ms |
| 4x4x4 | 3 x 4 | 1.67 s | 60.0 | 7.0 | 48.1 ms |
| 4x4x4 | 6 x 2 | 1.14 s | 88.0 | 9.1 | 60.4 ms |
| 4x4x4 | 8 x 1 | 1.09 s | 91.4 | 7.5 | 68.4 ms |
| 4x4x4 | 12 x 1 | 0.94 s | 106.1 | 10.6 | 86.5 ms |
| 4x4x4 | 12 x 12 (oversubscribed) | 2.00 s | 50.0 | 11.7 | 226 ms |
| 5x5x5 | 1 x 12 | 8.77 s | 11.4 | 10.5 | 79 ms |
| 5x5x5 | 3 x 4 | 6.89 s | 14.5 | 11.0 | 192 ms |
| 5x5x5 | 6 x 2 | 7.24 s | 13.8 | 10.3 | 375 ms |
| 5x5x5 | 8 x 1 | 8.22 s | 12.2 | 7.4 | 564 ms |
| 5x5x5 | 12 x 1 | 6.95 s | 14.4 | 10.7 | 694 ms |
| 5x5x5 | 12 x 12 (oversubscribed) | 10.03 s | 10.0 | 11.8 | 1091 ms |

- 4x4x4: one thread per solve gives 4.4x the throughput of sequential all-core solves (106 against 24 per second). 100 cubes take 0.94 s instead of 4.12 s. The all-core mode leaves the machine about 55 % idle on average, because the sequential 3x3x3 hand-off is 32 % of the single-thread time and the tasks are too fine (see below).
- 5x5x5: the gain is 1.26x; the hybrid 3 solves x 4 threads matches the throughput of 12 x 1 (14.5 against 14.4 per second) with 3.6x lower median latency (192 against 694 ms). For a batch of 100, 6.9 s instead of 8.8 s.
- 3x3x3: only the batch mode exists as a multi-core mode (9.0x at 12 threads).
- Oversubscription (12 solves each with 12 threads) is the worst case for both sizes.

Use the multithreaded algorithm when you need the lowest latency of a single answer (4x4x4: 4 threads; 5x5x5: 8 threads). Use one thread per solve when you want many answers.

## Apple M2 Pro: performance and efficiency cores

macOS has no hard thread pinning. `taskpolicy -b` (background QoS) confines a process to the E cores and lowers their clock, so the following slow-downs against a P core are lower bounds for the E cores, not their speed at default QoS [M]: 3x3x3 solve 4.7x, 2x2x2 build 5.5x, 4x4x4 solve 5.5x, 5x5x5 solve 6.0x, 5x5x5 cold build 6.1x (111 s against 18.2 s).

At default QoS, adding the 4 E cores to 8 busy P cores raises throughput by 26.5 % for the 3x3x3 (one E core is worth 0.53 P), 16 % for the 4x4x4 (0.32 P) and 18 % for the 5x5x5 (0.37 P, limited by memory latency) [D]. So an E core is worth 0.3 to 0.5 of a P core.

Scaling [M]: 3x3x3 batch is near-linear to 8 threads (7.1x), 9.0x at 12. 4x4x4 latency is flat from 4 threads on (41.7 ms at 4, 40.6 at 8, 41.6 at 12). 5x5x5 latency improves up to 8 threads, then only about 2 % more (89.8 to 87.6 ms).

Recommended thread counts:

| Goal | 2x2x2 | 3x3x3 | 4x4x4 | 5x5x5 |
| --- | --- | --- | --- | --- |
| Lowest latency of one solve | 1 | 1 (`solve`) | 4 | 8 (the P cores) |
| Throughput of many solves | one per core | `solve_batch`, 12 | one thread per solve, 12 in flight | 3 solves x 4 threads |
| Table build | 1 | 1 | 12 | 12 |

Never run solves at background QoS. The default of 12 threads is right for table builds and batch work; it is wasteful for 4x4x4 latency and wrong under concurrency.

## Where the time goes

**4x4x4 [M, report 6].** Per cube at 1 thread: phase 1 1.83 ms (sequential), phase 2 21.24, phase 3 28.00, 3x3x3 hand-off with 500 probes 21.00 (sequential), total 72.2. At 12 threads: 1.91, 10.82, 7.23, 21.49, total 41.6, so the hand-off is 52 % of wall time. 32 % of the single-thread time is sequential: the theoretical cap is 3.1x and about 2.5x (29 ms) on 8 P cores [D]. The tasks are very fine: phase 2 builds 23,645 tasks per cube and runs 13,964; phase 3 builds 6,949 and runs 3,597 (about 24 nodes each), and each task takes one mutex. Mutex wait is 0.3 ms of CPU per cube at 1 thread and 56.8 ms at 12. System time grows from 0.09 s to 76 s per 1000 cubes, 36 % of all CPU at 12 threads. Speculative work adds 82 % of nodes in phase 2 and 54 % in phase 3 at 12 threads; only about 58 % of the executed nodes are useful [D]. `par::run` creates a new `thread::scope` per call: 5.1 regions and 56 spawns per cube at 12 threads. Idle time at joins is small (6 %).

**5x5x5 [M, report 6].** No phase is sequential, but every phase stops scaling at about 8 threads. Per cube at 1 / 12 threads: phase 1 140.9 / 24.4 ms, phase 2 24.1 / 9.9, phase 3 38.2 / 7.1, phase 4 120.0 / 31.8, phase 5 84.5 / 26.2, 3x3x3 hand-off 0.55 / 0.58 ms. CPU grows from 408 to 927 ms per cube at 12 threads: nodes roughly double in phases 2, 4 and 5 (+122 %, +62 %, +99 %), system time is 52 ms and mutex wait 44.5 ms per cube, and there are 228 thread spawns (22 `thread::scope` regions, one per tried length). With one thread, each task compares its own count with the remaining quota, so it runs on after the global quota is met; Rust does 3 to 8 % more nodes than Java in phases 1 to 4 (phase 5 is identical). A node costs about 0.65 us in phase 1: random reads into 237 MB of tables, bound by DRAM latency.

## Server under load [M, report 5]

Release `cubacle-server` on `127.0.0.1` with defaults: 12 permits, 1 thread per 4x4x4 solve, 12 threads per 5x5x5 solve. The load client was a Python program on the same machine, which limits the 2x2x2 and 3x3x3 request rates.

| Size | Load | OK / 429 | Solves / s | p50 / p90 / p99 ms | Server cores |
| --- | --- | --- | --- | --- | --- |
| 2x2x2 | 12 clients, 100 requests | 100 / 0 | 6034 | 1.3 / 5.1 / 5.9 | 1.2 (client-bound) |
| 3x3x3 | 12 clients | 100 / 0 | 2031 | 2.1 / 4.7 / 31.9 | 3.5 (client-bound) |
| 4x4x4 | 12 clients | 100 / 0 | 109.4 | 85 / 166 / 325 | 10.6 |
| 5x5x5 | 12 clients | 100 / 0 | 10.3 | 970 / 1822 / 3602 | 11.8 |
| 4x4x4 | 1 client | 100 / 0 | 13.1 | 64 / 131 / 258 | 1.0 |
| 5x5x5 | 1 client | 100 / 0 | 11.5 | 82 / 131 / 279 | 10.5 |
| 4x4x4 | burst of 100 | 13 / 87 | n/a | 91 / 126 / 172 | 6.0 |
| 5x5x5 | burst of 100 | 12 / 88 | n/a | 996 / 1122 / 1139 | 11.6 |

- The 4x4x4 server matches the command-line 12 x 1 result, but a lone request gets one thread (64 ms) while 11 cores sit idle.
- The 5x5x5 server runs 12 permits x 12 threads = 144 threads, which is the oversubscribed 12 x 12 case: 10.3 per second with p50 970 ms. The measured 3 x 4 split gives 14.5 per second at p50 192 ms on the command line.
- In a burst, permits beyond the limit are rejected with 429, never queued (by design).

## Planned optimisations

These are ranked proposals from the measurement, **not done**. Gains marked [E] are estimates; [M] marks the measured basis.

| # | Proposal | Expected gain | Risk | Effort | Basis |
| --- | --- | --- | --- | --- | --- |
| A | Deterministic parallel 3x3x3 hand-off: the 6 URF searches as independent tasks, each with ceil(min_probes/6) probes and no shared state; shortest wins, ties to the lowest URF index | 4x4x4 hand-off 21.0 to 10.6 ms, total 42.2 to 32.0 ms (-24 %); moves 47.52 against 47.49 | answers differ from the serial run (fixtures change; needs an owner decision); 1.3x more probes in total | small (about 40 lines) | [M] prototype on 200 cubes; with 500 probes per URF: 52 ms, 47.11 moves |
| B | Persistent worker pool, coarse tasks, lock-free ordered merge (per-slot `OnceLock` plus an atomic prefix instead of one `Mutex`) | 4x4x4 phases 2+3 from 18 to about 7-8 ms; 5x5x5 100 to about 60-70 ms [E] | low: the ordered merge keeps answers deterministic | medium | [M] 36 % system time, 57 ms per cube mutex wait, 56 to 228 spawns per cube |
| C | Bound speculation: cancel or deprioritise tasks past the cut; give each task the global remaining quota (also fixes the 1-thread overshoot) | 5x5x5 CPU per cube 927 to about 500 ms, latency -25 to -35 %; 1-thread 5x5x5 -3 to -8 % [E] | low | medium | [M] node inflation 2x (5x5x5), +54 to +82 % (4x4x4) |
| D | 3x3x3 parity with upstream min2phase (`USE_TWIST_FLIP_PRUN`, conjugate pruning, `MAX_DEPTH2` 12) | 3x3x3 0.79 to about 0.3-0.4 ms; hand-off 21 to about 8-10 ms single-threaded [E] | changes every 3x3x3 answer and touches the search code (needs an owner decision) | medium to large | [M] 4.1x fewer nodes in Java |
| E | Thread defaults: 4x4x4 latency 4 threads, 5x5x5 8 threads; server 5x5x5 threads per solve = cores / permits (for example 3 x 4) | server 5x5x5 10.3 to 14.5 per second, p50 970 to about 190 ms; 2-3x less CPU for 4x4x4 [E] | none | trivial | [M] report 3 and 5 |
| F | Micro-optimisations: read the clock every N nodes instead of `Instant::now()` per 3x3x3 node when a budget is set; stop re-walking the root for each first-move task; prefetch pruning entries in 5x5x5 phases 1 and 4 | 2 to 10 % each [E] | low | small | [M] 0.65 us per node in 5x5x5 phase 1 |
| G | Parallel 4x4x4 phase 1 | at most 1.8 ms | low | medium | [M] |

Changing answers (A and D) needs an owner decision, because the recorded fixtures pin the solver output. Estimated latency with A to F [E]: 2x2x2 unchanged (0.002 ms); 3x3x3 about 0.3-0.4 ms (now 0.79, Java 0.57); 4x4x4 about 12-15 ms (now 41.6, Java 114); 5x5x5 about 50-65 ms (now 100, Java 576). A speed-up of 10x over Java is not reached today for either size (4x4x4: 2.74x; 5x5x5: 5.75x at 12 threads); 4x4x4 would need about 11.4 ms, and the sequential parts alone take 23 ms now [D]. For throughput it is reached for the 4x4x4 (12.1x) [D].

## Reproduce

The full measurement used a scratch harness outside the repository (instrumented copy of the code, serialised and idle-checked runs); its commands are in the report, not in the repository. What anyone can run from the repository:

```sh
cargo build --release
target/release/cubacle solve --size 4 --scramble "R U Rw Fw' 2U" -v
CUBACLE_THREADS=1 target/release/cubacle solve --size 4 --scramble "R U Rw Fw' 2U" -v
RUSTFLAGS='-C target-cpu=native' cargo run --release --example benchmark -- 100
tools/bench/run_four.sh /path/to/TPR-4x4x4-Solver /tmp/bench4 1000 1
tools/bench/run.sh /path/to/cube555 /tmp/bench 100
```

`CUBACLE_THREADS` sets the thread count. The two scripts need Java 8 or newer and a checkout of the original solver.
