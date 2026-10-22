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
| 5x5x5 | Reduction (centres and wings, then the 3x3x3 solver) | Supported, solutions are long |
| 4x4x4, 6x6x6, 7x7x7 | Reduction | Planned, not yet supported |

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

## 5x5x5: reduction

`FiveSolver` solves a 5x5x5 whose six face centres stay in place. It never turns the middle layer or a wide block, so the solved cube is the solved facelet string and no whole-cube rotation is involved.

1. It puts the 24 x-centres and the 24 t-centres on their faces, and the 24 wings next to the middle edge they belong with. Each step is a 3-cycle of three pieces: a short commutator found offline, conjugated by a setup of at most four moves. A greedy rule takes the cycle that fixes the most pieces per move.
2. A 5x5x5 has no flipped-edge parity, because the middle edges give every wing its orientation. Only the permutation parity of the 24 wings can be odd. 3-cycles cannot change it, so one inner quarter turn (`2R`) first repairs it when it is odd.
3. The corners, middle edges and centres then form a 3x3x3 (stickers in rows and columns 0, 2 and 4). `FiveSolver` hands it to the 3x3x3 solver it was built with, through the `Solver` trait, and appends that solution.
4. It replays the answer of the 3x3x3 stage on the 3x3x3 it sent, then the whole solution on the 5x5x5, before returning it.

What it does not do: it does not look for short solutions. Over 1000 random cubes, the solutions had 348 to 477 moves (median 412), of which the 3x3x3 stage adds at most 25. It does not accept cubes whose face centres have moved. Orbit64 tokens of the 5x5x5 are not decoded.

Measured in a release build on an Apple M-series laptop: the 3-cycle library has about 11,650 entries (4,042 x-centre, 3,564 t-centre and 4,044 wing cycles out of 4,048 possible each); it is built on first use or by `prepare()` in about 0.22 s, and a cold `cubacle solve` of a 5x5x5 peaks near 5 MB. With the library ready a solve takes about 1 ms (median 1.1 ms, maximum 4.6 ms over the 1000 cubes). The 3x3x3 solver is injected, so a 5x5x5 build always includes its tables.

## 4x4x4, 6x6x6, 7x7x7: planned

The input notations and the API already accept cubes of these sizes. No engine exists, so a solve request fails:

- The command-line tool exits with a non-zero status.
- The REST API answers 501 with the code `no_engine`.

The planned approach is reduction, as for the 5x5x5. The even cubes also need parity handling and have no fixed centres. See [Architecture](architecture.md).

## Limits

| Limit | Value |
| --- | --- |
| Supported sizes | 2x2x2, 3x3x3 and 5x5x5 |
| Default move limit | 21 for the 2x2x2 and 3x3x3 (CLI), none for the 5x5x5 |
| REST `maxMoves` | 1 to 25, or 1 to 255 for the 5x5x5 |
| REST `budgetMs` | 1 to 600000 |
| REST scramble length | 1 to 200 moves |
