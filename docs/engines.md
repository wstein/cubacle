---
title: "Engines and limits"
description: "Supported cube sizes, engines and limits."
weight: 5
---

# Engines and limits

An engine solves one cube size. cubacle picks the engine from the size of the cube you give it.

| Size | Engine | Status |
| --- | --- | --- |
| 2x2x2 | Optimal table solver | Supported |
| 3x3x3 | Kociemba two-phase | Supported |
| 4x4x4 | Reduction (centres and wings, then the 3x3x3 solver) | Supported, about 47 moves |
| 5x5x5 | Reduction (centres and wings, then the 3x3x3 solver) | Supported, about 73 moves |
| 6x6x6, 7x7x7 | Reduction | Planned, not yet supported |

## 2x2x2: optimal table solver

The solver keeps one corner fixed and searches an exact distance table of all 3,674,160 states. Its solutions are optimal in the half-turn metric: U, R and F turns by 90 or 180 degrees.

- The table is built on first use, in well under a second in a release build.
- A cube that is solved in any orientation needs no moves.
- With a ready table, a solve takes microseconds.
- The first solve in a process includes the one-off table build. A time budget cannot interrupt the build, and the build counts against the budget.

Because the 2x2x2 has no centres, colour input needs a [colour scheme](cli.md#colour-schemes).

## 3x3x3: Kociemba two-phase

The 3x3x3 solver uses an optimized implementation of the two-phase algorithm, derived from [min2phase](https://github.com/cs0x7f/min2phase). It returns a solution of at most the move limit you give. A limit of 20 or 21 moves is a good choice. The default is 21.

- A `Min2PhaseSolver` builds its move and pruning tables on first use or in `prepare()`. The tables belong to the solver instance and are shared by its clones, so create one solver at startup and clone it. `Min2PhaseSolver::shared()` returns a process-wide default instance.
- A search without a time budget is deterministic for a fixed cube, move limit and build.
- The parallel searches (`solve_parallel*`) share a best-length bound between workers, so they are not guaranteed to return the same solution on every run; the result is always a valid solution within the move limit.
- A search with a time budget is not deterministic. CPU scheduling and the moment the search sees the time limit can change the best result.
- The command-line tool checks each 3x3x3 solution by replaying it on the cube.

## 4x4x4: reduction

`FourSolver` solves any 4x4x4. A 4x4x4 has no fixed centres, so the solver treats the cube as solved when every face shows one colour in the standard scheme (U white, R red, F green, D yellow, L orange, B blue in the default notation), and the answer ends with the whole-cube rotation that gets there. The rotation is written as wide turns: `Rw Lw'` is the rotation about the R axis, and at most four such turns bring any cube to the standard orientation. The solution therefore solves the cube exactly as `Facelets::apply` replays it, with layer turns only (`R`, `Rw`, never a slice turn).

1. A port of [TPR-4x4x4-Solver](https://github.com/cs0x7f/TPR-4x4x4-Solver) (Chen Shuang, MIT option, see `NOTICE.md`), Tsai's three-phase reduction. Phase 1 brings the centres into a state where every colour pair sits on one axis; it enumerates up to 10,000 solutions and keeps the best 500 in the same priority queue as the original. Phase 2 finishes the centres without breaking the pairing of the wings, until 100 results exist; phase 3 pairs the wings and solves the centres (the original's third and fourth steps in one). The tables (about 12 MB: centre moves and pruning, and a 2-bit wing pruning table of 31 million states) are built on first use or in `prepare()`, in about 0.2 s on 12 cores and 0.4 s on one; they have no cache file.
2. The corners, wings and centres then form a 3x3x3. `FourSolver` hands it to the 3x3x3 solver it was built with, through the `Solver` trait (a solver of another kind is an internal error). It asks that solver for at least 500 probes (`SolveOptions::min_probes`, counted, not timed; the original does the same), which gives shorter tails and the same answer on every run.
3. It maps the model's moves back to real moves, appends the rotation, and replays the whole solution on the 4x4x4 before returning it; a mismatch is `Failure::Internal`, never a wrong answer. A cube that is not a 4x4x4, uses a colour scheme other than the standard one, has the wrong number of stickers of a colour on the centres, repeats or lacks a wing, or has a corner twisted alone (twists must sum to a multiple of 3) is `InvalidInput`. Two swapped corners or wings are a valid 4x4x4 and are solved.
4. The searches use `CUBACLE_THREADS` threads (default all cores). Phase 1 is sequential (it takes about 2 ms); phases 2 and 3 search several (candidate, first move) pairs at once and join the results in the order a single thread would find them, so without a time budget the solution does not depend on the number of threads. The server uses one thread per solve.

What it does not do: it does not look for shorter answers than the original does, and `maxMoves` bounds the whole solution including the rotation. If the time budget runs out the solve fails with `BudgetExhausted`; the message says so when the tables were still being built. Without a budget the 3x3x3 stage is limited to 10 s. With a move limit it is limited to 1 s, whatever the budget: the reduction is fixed, so a limit that the 3x3x3 tail cannot meet ends as `Exhausted` (the REST API answers 422 `no_solution_within_limit`, not 504) instead of searching until the budget ends; a tail that can meet it is found in milliseconds.

Measured on one 12-core macOS machine (Apple M2 Pro), release build, the same 1000 random cubes for both solvers (drawn by the original's generator), every answer replayed on the cube:

| | TPR-4x4x4-Solver (Java) | `FourSolver` 1 thread | `FourSolver` 4 threads | `FourSolver` 12 threads |
| --- | --- | --- | --- | --- |
| Moves without the rotation, mean / median / p90 / max | 44.59 / 45 / 46 / 47 | 44.34 / 44 / 46 / 47 | 44.34 / 44 / 46 / 47 | 44.34 / 44 / 46 / 47 |
| Moves with the rotation, mean / median / p90 / max | 47.71 / 48 / 50 / 51 | 47.47 / 48 / 49 / 51 | 47.47 / 48 / 49 / 51 | 47.47 / 48 / 49 / 51 |
| Solve time, mean / median / p90 | 115 / 99 / 197 ms | 72 / 62 / 117 ms | 40.0 / 37.8 / 56.8 ms | 39.5 / 38.1 / 53.4 ms |
| Table build | 0.8 s (3x3x3 included) | 0.37 s | 0.19 s | 0.17 s |
| Peak memory | not measured | 65 MB | 71 MB | 66 MB |

The Java time is one warm JVM thread. The answers of 1, 2, 4, 8 and 12 threads were identical for all 1000 cubes (also after the first-move split of the final version). The first three phases (the part of the solution before the 3x3x3 tail) are identical to the original's on all 1000 cubes; the whole answer without the rotation is identical on 665, because the 3x3x3 tails come from different solvers (ours is `Min2PhaseSolver`).

Four threads cut the mean time to 0.56 of the single-thread time (40.0 against 71.9 ms), not to the 0.5 that was the aim. The 3x3x3 stage (21 ms of the 73 ms on average) runs on one thread, and phase 1 (2 ms) too; with those fixed, four threads cannot go below 0.49 even if phases 2 and 3 scaled perfectly, and phases 2 and 3 together scale by about 2.9 on four threads.

## 5x5x5: reduction

`FiveSolver` solves a 5x5x5 whose six face centres stay in place. It turns the two outer layers of a face (`R`, `2R`, and the two-layer turns `Rw`, which appear in its solutions), never the middle layer (`3R`), so the solved cube is the solved facelet string and no whole-cube rotation is involved.

1. A port of [cube555](https://github.com/cs0x7f/cube555) (Chen Shuang, MIT option, see `NOTICE.md`): five searches in a row bring the centres and then the wing edges into the state of a 3x3x3. Each search is an iterative-deepening search guided by a pruning table, and it keeps several candidates for the next one (200, 500, 500, 500 and 1 for phases 1 to 5, as in the reference). Phase 1 searches from three conjugated copies of the cube, and phase 2 hands three conjugated copies of each of its solutions to phase 3.
2. The corners, middle edges and centres then form a 3x3x3 (stickers in rows and columns 0, 2 and 4). `FiveSolver` hands it to the 3x3x3 solver it was built with, through the `Solver` trait (a solver of another kind is an internal error), and appends that solution. A turn that ends one part and begins the next on the same face is merged or cancelled.
3. It checks that the 3x3x3 answer uses only single-layer turns, then replays the whole solution on the 5x5x5 before returning it.
4. The pruning tables hold 237 MB (12 files). They are built by a parallel breadth-first search (the number of threads is `CUBACLE_THREADS`, default all cores) on the first solve or in `prepare()`, and saved in the cache directory (`CUBACLE_CACHE_DIR`, else `$XDG_CACHE_HOME/cubacle`, else `~/.cache/cubacle`) with a checksum and a format version. A missing, damaged or foreign file is rebuilt. The tables are identical for any thread count, and the search merges its results in a fixed order, so without a time budget the solution does not depend on the thread count.

What it does not do: it does not accept cubes whose face centres have moved, and `maxMoves` bounds the whole solution. If the time budget runs out the solve fails with `BudgetExhausted`; the message says so when the tables were still being built. Without a budget the 3x3x3 stage is limited to 10 s, so a tight `maxMoves` ends as `Exhausted` instead of searching for minutes.

Measured on one 12-core macOS machine, release build, 100 random cubes, against the Java reference (same cubes; both solutions finished by our 3x3x3 solver):

| | cube555 (Java) | `FiveSolver` 12 threads | `FiveSolver` 1 thread |
| --- | --- | --- | --- |
| Total moves, mean / median / p90 | 73.1 / 73 / 75 | 73.1 / 73 / 75 | 73.1 / 73 / 75 |
| Solve time, mean / median / p90 | 865 / 656 / 1776 ms | 92 / 77 / 187 ms | 444 / 404 / 723 ms |
| Table build, cold | about 50 s | 3.3 s | 18.3 s |
| Table load, warm | 1.8 s | about 1.0 s | about 1.0 s |
| Peak memory, warm | 0.76 GB | 335 MB | 327 MB |

The peak memory is above the 300 MB aim: 237 MB are pruning tables, about 70 MB the move and symmetry tables built next to them (measured with `vmmap`), and the rest the process and its libraries. Reading the cache streams the files, which avoids a second copy of the largest table.

## 6x6x6, 7x7x7: planned

The input notations and the API already accept cubes of these sizes. No engine exists, so a solve request fails:

- The command-line tool exits with a non-zero status.
- The REST API answers 501 with the code `no_engine`.

The planned approach is reduction, as for the 4x4x4 and 5x5x5. The 6x6x6 also has no fixed centres and needs parity handling. See [Architecture](architecture.md).

## Limits

| Limit | Value |
| --- | --- |
| Supported sizes | 2x2x2, 3x3x3, 4x4x4 and 5x5x5 |
| Default move limit | 21 for the 2x2x2 and 3x3x3 (CLI), none for the 4x4x4 and 5x5x5 |
| REST `maxMoves` | 1 to 25, or 1 to 255 for the 4x4x4 and 5x5x5 |
| REST `budgetMs` | 1 to 600000 |
| REST scramble length | 1 to 200 moves |
