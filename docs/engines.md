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
| 4x4x4 to 7x7x7 | Reduction | Planned, not yet supported |

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

## 4x4x4 to 7x7x7: planned

The input notations and the API already accept cubes of these sizes. No engine exists, so a solve request fails:

- The command-line tool exits with a non-zero status.
- The REST API answers 501 with the code `no_engine`.

The planned approach is reduction: solve the centres and edges to reduce the cube to a 3x3x3, then hand it to the 3x3x3 solver. See [Architecture](architecture.md).

## Limits

| Limit | Value |
| --- | --- |
| Supported sizes | 2x2x2 and 3x3x3 |
| Default move limit | 21 (CLI) |
| REST `maxMoves` | 1 to 25 |
| REST `budgetMs` | 1 to 600000 |
| REST scramble length | 1 to 200 moves |
