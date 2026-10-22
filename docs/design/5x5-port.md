---
title: "5x5x5 port plan"
description: "Plan to replace the greedy 5x5x5 engine (now deleted) with a Rust port of the five-phase solver cs0x7f/cube555."
weight: 20
---

# 5x5x5 port plan

Status: landed. The owner decided to port `cs0x7f/cube555` (Java, five-phase
reduction) instead of designing a search from scratch; the port replaced the
greedy engine, which is deleted. This page is the plan; where the build differs,
a "Status" line in the section says how. Measured results are in
[Engines and limits](../engines).

## 1. Baseline (measured)

100 identical random cubes, both solvers replayed with `Facelets::apply`, Mac14,10
with 12 cores, Java 21, release build of ours. Harness: `tools/bench/`. The
Java solver returns the reduction only, so both columns end with our `Min2PhaseSolver`.

| | cube555 (Java) | claimed by its README | our greedy engine |
| --- | --- | --- | --- |
| reduction moves, mean (min to max) | 51.5 (48 to 54) | 51 to 52 | not separable |
| 3x3x3 moves, mean | 21.6 (20 to 23) | about 20 (task brief) | at most 25 |
| total, mean / median / p90 | 73.1 / 73 / 75 | none | 412.4 / 414 / 441 |
| solve time, mean / median / max | 865 ms / 656 ms / 5.6 s | 1 to 2 s | 1.3 ms / 1.2 ms / 3.2 ms |
| cold start (tables built and written) | 50.3 s | none | 0.25 s |
| warm start (tables read) | 1.8 s | none | 0.25 s |
| tables on disk | 237 MB, 12 files | about 240 MB | none |
| peak resident memory | 1.6 GB cold, 0.76 GB warm | at most 500 MB | 5 MB |

The memory claim is not met by the Java process (the JVM heap holds the tables
as `int[]` plus move tables). Phase shares of the 865 ms (mean of 100):
phase 1 286 ms, phase 2 35, phase 3 109, phase 4 242, phase 5 190.

Acceptance rule for the port: same reduction lengths as the Java run on the same
cubes (identical algorithm and identical move order give identical lengths),
time and memory not worse in order of magnitude.

## 2. Licence handling

The README offers GPLv3 and MIT, both by Chen Shuang (the MIT notice is dated
2023; `LICENSE-MIT` holds the 2024 notice of min2phase_rust). We take MIT.

1. Already done (commit `docs(notice)`): a NOTICE.md row marks the port as
   planned, and the older statement that the 5x5 engine is written from scratch
   now covers only the greedy engine (later deleted, and its row with it).
2. Before the first port commit: add the cube555 MIT notice (copyright line and
   permission text, verbatim) to `LICENSE-MIT` as a second block, and change the
   NOTICE row from "planned" to "landed".
3. Every ported file starts with a header naming the Java origin
   (`Ported from cs0x7f/cube555 src/Phase3Search.java, MIT, Copyright (c) 2023
   Chen Shuang`) and states the AGPL-3.0-or-later terms of the Rust file.
4. Files written new (cache format, parallel runner, tests) carry no origin line.

The deleted greedy engine leaves NOTICE with it.

## 3. What the Java solver is

Five phases over a facelet-free cubie model (`CubieCube`: 24 x-centres,
24 t-centres, 24 wings, 12 middle edges, corners). Each phase is a table-driven
IDA* with a restricted move set, followed by a best-first beam:

| Phase | Goal | Moves | Coordinates | Prune tables (states, file) |
| --- | --- | --- | --- | --- |
| 1 | two opposite face colours of both centre types onto their two faces | 36 (18 face, 18 two-layer) | t and x centre C(24,8) = 735,471 each, 16-fold symmetry (46,935 and 46,371 classes) | t-sym, x-sym (23 KB each), joint x-sym by t-raw, depth-limited and hashed (2^26 entries, 33.5 MB) |
| 2 | remaining centres of the two colours onto UD, edge parity fixed | 22 | t and x centre C(16,8) = 12,870 each, plus wing parity | two tables of 25,740 states (13 KB each) |
| 3 | wings into four slots per layer, middle edges placed, centres into L/R-F/B halves | 24 | centre 1,225, middle edge 2,048, wing 2,704,156 raw / 86,048 sym classes, 32 symmetries | centre by middle edge 2.5 M states (1.25 MB); wing by middle edge 176 M states (88.1 MB) |
| 4 | pair the high and low wings, finish UD centres | 16 | middle-low edge, UD and RL centre, high edge 70 x 1,680 | edge-sym 49.8 M states (24.9 MB); mid-low-edge by UD centre 145 M states (72.6 MB); by RL centre 6.4 M (3.2 MB); centre 1.06 M (0.5 MB) |
| 5 | complete centres and pairing, leaving a 3x3x3 | 14 | centre, low edge | centre 176 k (88 KB); low-edge-sym by centre 25.9 M states (13 MB) |

(Table sizes are the measured file sizes; the sum is 237 MB. The goal column
is my reading of the code and the first four rows should be confirmed while
porting phase by phase, by checking each intermediate state.)

Search driver (`PhaseSearch`, `Search.solveReduction`): the input cube, in
three axis conjugations, enters phase 1, which collects up to 200 solutions;
each is the start of phase 2 (up to 500), then phase 3 (500, filtered by a
bit-mask test), phase 4 (500), phase 5 (1). In every phase the start nodes are
ordered by `cost so far + pruning value` (ties keep insertion order) and
searched with IDA* at rising total length; the first node reaching the goal at
the current length yields a solution, and the search continues until the phase
has its quota. The beam is what makes the result about 52 moves: later phases
see many candidates, and the cheapest total wins. The Java code is single
threaded and has no cancellation.

## 4. Mapping to Rust

Status: landed with other file names than the table below plans (`mv`, `cubie`,
`cubie_sym`, `build`, `spaces`, `p1` to `p5` with `p4_model` and `p5_model`,
`tables`, `solving`, `reduction`, `solver`); the generic table builder, cache,
parallel runner and IDA* live in `src/reduction/`.

Planned layout under `src/engines/five/` (the greedy engine was deleted instead
of moved; see the status above):

| Module | Content | Java origin | est. lines |
| --- | --- | --- | --- |
| `cubie.rs`, `cubie_moves.rs`, `cubie_sym.rs` | cubie cube, move and symmetry tables, facelet conversion to our `Facelets` | CubieCube, SolvingCube | 650 in three files |
| `coord.rs` | `Coord`/`SymCoord`/`RawCoord` as traits or plain structs of move tables | Util | 200 |
| `prun.rs`, `prun_build.rs` | 4-bit pruning table, BFS builder (full, symmetric, depth-limited), parallel layers | Util.PruningTable | 450 in two files |
| `p1.rs` to `p5.rs` plus `p3_edge.rs`, `p4_edge.rs`, `p4_center.rs` | per phase: coordinates, moves, prune lookups, search node | Phase1..5 (Center, Edge, Search) | about 1,800 in about 10 files |
| `beam.rs` | the best-first beam and IDA* (generic over a phase trait) | PhaseSearch, Search | 350 |
| `tables.rs` | owner of all tables, `Arc<OnceLock<Tables>>`, build order, cache | Search.init | 200 |
| `cache.rs` | on-disk format and I/O | Tools | 150 |
| `solver.rs` | `FiveSolver::new(Arc<dyn Solver + Send + Sync>)`, hand-off to the 3x3 | Search, example | 250 |

Java total is 4,378 lines; I expect 4,500 to 5,500 lines of Rust, every file
under 500 lines, `pub(crate)` except `FiveSolver`. The bundled `twophase.jar`
is replaced by the injected `Solver`; the input is `three_by_three` (as today).
Table state is owned by the solver instance behind `Arc<OnceLock<Tables>>`
exactly as `FiveSolver` does now; no statics. `prepare()` builds or loads the
tables and the 3x3 tables.

Representation: tables are `Vec<u32>` with 8 entries of 4 bits per word like
Java (an `u64` variant is possible later). Move tables as flat `Vec<u16>` or
`Vec<u32>` indexed `state * moves + move` instead of arrays of arrays. Memory
estimate: about 237 MB prune tables, 25 to 35 MB move/sym tables, so 270 MB in
total at rest, with no heap overhead of a runtime. This is an estimate, not a
measurement.

## 5. Tables: building, sharing, persistence

Options:

| Option | Cold start | Warm start | Verdict |
| --- | --- | --- | --- |
| Build at every start | 10 to 30 s (estimate: Java 50 s, of which most is the BFS of the two 150 M-state tables; Rust with parallel build should be several times faster; must be measured) | same | not acceptable for the CLI as the only way |
| Disk cache (`CUBACLE_CACHE_DIR`, default the platform cache directory) | as above, once | 0.3 to 1 s to read 237 MB (Java 1.8 s) | recommended |
| `build.rs` | adds the build time to every compile and 237 MB to the binary or `OUT_DIR` | | rejected |
| Download at first use | needs a network and a trust anchor | | rejected |

Recommendation: build on demand and cache. `prepare()` loads the cache if all
files pass validation, otherwise builds (in parallel, section 6) and writes the
files atomically (write to a temporary name, rename). Cache file: magic,
format version, table id, state count, FNV-1a or CRC of the payload, little
endian words. A corrupt or foreign file is rebuilt, never trusted. A failure
to write the cache is not an error (read-only home, container): the tables stay
in memory.

The server (stateless, long-running) calls `prepare()` at start, as it does
now (`crates/server/src/lib.rs` lines 95-106); the 5x5 tables then live for the
process. With a cold cache the server start is delayed by the build; a
deployment should provide a writable cache volume or accept one slow start.
The CLI pays the cold cost once per machine; it must print one line to standard
error before a long build ("building 5x5x5 tables, once, about N s") so it does
not look hung.

There is no fallback engine (section 8): a request that arrives before the
tables are ready builds them first.

## 6. Multi-core

The Java code uses one core. Opportunities, by value:

1. Table construction. The 12 tables are independent. Java cold times
   (cumulative, from the build log) were about 19 s for the 176 M-state
   wing table, 11 s for the 145 M-state table, 6 s and 5 s for two more, the
   rest small. With one task per table the critical path is the largest table
   (about 19 s in Java), the sum about 50 s; layer-parallel BFS inside the
   large tables (split the state range among workers, with disjoint nibble
   writes per chunk, merge by depth layer) lowers the path further. Expected
   (estimate, unmeasured for Rust): 3x to 6x on 12 cores for the full build.
2. The search. The phases 2 to 5 start from up to 200 to 500 independent nodes.
   At a given total length, entries are independent: run them in parallel and
   merge the found solutions in entry order, then cut at the quota. This yields
   exactly the serial result (same solutions, same order) at the price of some
   extra work when the quota fills early. Phase 1 has one root; partition by
   the first move (36 roots) the same way. Phases 1, 4 and 5 are 83% of the
   measured time, so the ceiling by Amdahl is about 5 to 8 times on 12 cores
   (estimate: the time per solve could fall from 0.87 s to 0.15 to 0.25 s).
3. Pipelining phases (start phase 3 on phase 2 solutions as they appear) is
   less deterministic and adds little; not planned.

Determinism: results are identical to the serial run for any thread count,
because merging is ordered by (length, entry index) and quotas are applied
after the merge. If a time budget stops the search, the result depends on timing
and is documented as such (the 3x3 does the same).

Status: the worker pool was not moved; `src/reduction/par.rs` uses scoped
threads, with `CUBACLE_THREADS` for both the table build and the search.

Original plan, not followed: the pool (`src/engines/three/pool.rs`) is private to
the 3x3 engine and holds a process-wide static, and engines may not import each
other, so a first commit would move it to a neutral module. Built instead:
`src/reduction/par.rs` runs scoped threads, the count comes from
`CUBACLE_THREADS` or `FiveSolver::with_threads(n)`, and there is no global
(`tests/architecture_test.rs` checks `src/reduction/` for statics).

Measured speedup (100 cubes, 12 cores): mean solve time 444 ms on one thread and
92 ms on twelve; see [Engines and limits](../engines).

## 7. Shared framework with the 4x4

The owner plans to port `cs0x7f/TPR-4x4x4-Solver` (README: 44.39 moves, 250 ms,
at most 30 MB, MIT). I looked at its file list and class names only. It is a
min2phase-style three-phase solver with symmetry-reduced centre and edge
coordinates (Center1/2/3, Edge3) and one IDA search, not the beam of
`PhaseSearch`. So the two code bases share primitives and not the driver:

- symmetry coordinate tables (raw to sym, sym move table, self-symmetry masks),
- 4-bit pruning tables with the BFS builder and the on-disk cache,
- the worker pool and a parallel-for with ordered merge,
- the generic IDA* node loop (a trait with `do_move_prune`, move-skip masks).

Proposal: a neutral module `src/reduction/` (crate-private, `pub(crate)`)
holding exactly those four, written once for the 5x5 port and reused by the
4x4. Dependency rule: `puzzle <- notation <- reduction <- engines`; the
architecture test (`tests/architecture_test.rs`) needs one added rule. The beam
driver stays in `five/` and moves to `reduction/` only if the 4x4 port needs it.
This claim rests on file names and class names and must be confirmed by the 4x4
author.

## 8. Greedy engine: dropped

Owner decision: the greedy engine (1,236 lines, 412 moves) is deleted, with its
tests, its golden fixture `tests/fixtures/five.txt` and its documentation; there
is no fallback and no `five/greedy/`. While the tables are missing, `solve()`
builds them (`prepare()`), so a first request pays the cold start; a time budget
shorter than the build fails with `BudgetExhausted` and a message that names the
table build. The golden fixture of the port is the Java transfer record of
section 9. The public name `FiveSolver` and its constructor stay.

## 9. Correctness and tests

1. Each phase checks its goal on the cubie state it reached (debug assertion
   plus a test) and the final answer is replayed with `Facelets::apply`
   against the input before it is returned, as the engine does today.
2. Golden transfer: the Java solver is deterministic. The first 20 to 30 cubes
   of the benchmark with the Java reduction moves are stored as a fixture
   (data only: facelets and move strings). The single-threaded Rust port must
   reproduce the move strings exactly; this proves the port, not just that the
   output is valid. The multi-threaded run must reproduce them too.
3. Unit tests per phase on small fixtures: coordinate round trips, move table
   vs cubie move, symmetry conjugation consistency, on the small tables only.
4. Heavy tests. The full tables are about 237 MB and 10 to 30 s to build; the
   GitHub runner (ubuntu-latest: 4 cores, 16 GB for public repositories, 7 GB
   otherwise; jobs have 30 min timeouts in `ci.yml`) can afford it but
   `cargo test --workspace --release` runs many binaries in parallel and
   should stay fast. Plan:
   - tables accept a depth cap (`TableDepth::Capped(n)`): an unknown entry is
     treated as `n + 1`, still admissible, so search is correct but slower. Unit
     and end-to-end tests use capped tables (a few MB, under a second) on cubes
     scrambled with few moves per phase.
   - one test with the real tables, `#[ignore]` by default, run by a separate CI
     job (`cargo test --release -- --ignored five_full`) that caches the table
     directory with `actions/cache` keyed by the format version. No timing
     assertions anywhere.
   - the golden-transfer test of point 2 is part of the ignored suite and of the
     release checklist.
5. Memory: the ignored test records the peak table size in bytes (computed, not
   measured by the OS) and asserts it stays below a limit (300 MB).

## 10. Time budget and cancellation

`SolveOptions::time_budget` and `max_moves` stay. The search checks a deadline
every few thousand nodes. As built: on expiry the solve stops and fails with
`Failure::BudgetExhausted`; there is no "best candidate continues" mode. If the
tables were still being built, the message says so. `max_moves` bounds the whole
solution (reduction plus the 3x3 tail, which gets `min(25, max - used)`). With a
budget the result is not deterministic, without it the result is.

## 11. Commit plan (as landed)

1. `docs`: this plan, the NOTICE row and the cube555 MIT notice.
2. `feat(reduction)`: shared building blocks (tables, BFS, cache, parallel
   runner, IDA*); then a `refactor` with tagged moves, depth caps and streamed
   cache loading.
3. `feat(five)`: piece model and symmetries; table builders; phases 1 to 5, one
   commit each; the chain of phases with the Java golden.
4. `feat(puzzle)`: `Failure::Internal`, HTTP 500.
5. `feat(five)!`: wiring, greedy engine deleted, CLI notice, private cache
   directories in tests.
6. `docs`, `ci`: docs, notes, architecture rules and the full-table CI job.

## 12. Risks and open questions

- The Java run is the only oracle. Its claims (memory 500 MB, 51 to 52 moves)
  were measured only partly: moves yes, memory no.
- The port must keep Java's tie-breaking (stable order of equal estimates) for
  golden equality; Java's `TreeSet` returns "greater" on ties, which equals
  insertion order.
- Table build time in Rust is an estimate; if it exceeds 30 s on one core, ship
  a pre-built cache as a release asset (SHA-256 pinned) as an option.
- A 270 MB resident set is large for the server container; the owner should
  state the memory limit. Option: memory-map the cache file (needs a dependency
  or `unsafe`; not planned).
- Decided: the default cache directory (`CUBACLE_CACHE_DIR`, else
  `$XDG_CACHE_HOME/cubacle`, else `~/.cache/cubacle`), no greedy fallback,
  `CUBACLE_THREADS` governs table builds too. Open: 4x4 author agreement on
  `src/reduction/`.
- Not verified: the phase goals in section 3 (read quickly); Rust build time;
  multi-core speedups; the 3x3 tail with a minimum search time.

## Result

Measured on 2026-10-22, Apple M2 Pro (12 cores), release build, the same 100
random cubes for both solvers, every answer replayed with `Facelets::apply`. The
harness is `tools/bench/run.sh`. The sections above are the plan; this section is
what was built and measured, and it replaces the estimates in sections 5, 6 and 12.

| Criterion | Aim | Java (cube555) | Rust |
| --- | --- | --- | --- |
| Total moves, mean / median / p90 | about 73 | 73.1 / 73 / 75 | 73.1 / 73 / 75 |
| Reduction moves, mean | 51.5 | 51.5 | identical (the Java reduction is a prefix of every answer on the golden cubes) |
| Solve time, mean / median / p90, 12 threads | | 865 / 656 / 1776 ms | 92 / 77 / 187 ms |
| Solve time, mean / median / p90, one thread | | 865 / 656 / 1776 ms | 444 / 404 / 723 ms |
| Cold table build | | about 50 s | 3.3 s on 12 threads, 18.3 s on one |
| Warm table load | | 1.8 s | about 1.0 s |
| Memory, warm | at most 300 MB | 0.76 GB | 328 MB (one thread), 335 MB (12 threads): **missed** |
| Answers with 1, 4, 12 threads | identical | | identical on 300 cubes |
| Pruning tables | | 237 MB | 237 MB, byte-identical to Java's |

Notes on the differences from the plan.

- **Memory.** 237 MB are pruning tables and about 70 MB are move and symmetry tables.
  Storing small tables as 16-bit values saves about 3 MB; dropping the 10 MB
  wing table and recomputing it made solving about five times slower and was
  reverted. Going below 300 MB needs a smaller pruning-table format (see the note
  `5x5 memory above target`).
- **No greedy fallback.** The greedy engine was deleted; `solve` builds the tables
  first when they are missing, and the budget message says so.
- **Java's skip-mask shift bug** is reproduced on purpose, because without it the
  answers differ from Java's (note `5x5 skip masks keep a Java shift bug`).
- **Worker pool.** The search uses scoped threads in `reduction::par`; the pool
  of the 3x3x3 engine was not moved.
- **Move limit.** A tight `-m` gives the 3x3x3 hand-off a 10 s default budget and
  ends in `Exhausted` instead of running on.
