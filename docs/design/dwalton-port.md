---
title: "6x6x6 and 7x7x7 port"
description: "How the 6x6x6 and 7x7x7 engine follows the reduction stages of dwalton76/rubiks-cube-NxNxN-solver, and where it differs."
weight: 30
---

# 6x6x6 and 7x7x7 port

Status: landed (`src/engines/dwalton`). The engine follows the stage order of
[dwalton76/rubiks-cube-NxNxN-solver](https://github.com/dwalton76/rubiks-cube-NxNxN-solver)
(MIT, Copyright (c) 2019 Daniel Walton; see `NOTICE.md`): the centers are
reduced in stages, the wings are paired through a fake 4x4x4 or 5x5x5, and a
3x3x3 solver finishes. Everything else is new code: no Python, no subprocess,
no lookup-table files of the reference.

## Method

1. **Center stages.** Each stage is an iterative-deepening search (the generic
   `reduction::ida`, thread-count independent) over pruning tables. A node is
   one bit mask per group of stickers (bit set: the sticker shows a marked
   color); a move permutes the bits through byte lookup tables.
   - Stages that the reference hands to its 5x5x5 center tables (6x6x6: stages 2
     and 4; 7x7x7: the inner and outer staging of both axes) use two dense
     tables of 735,471 states (8 of 24 stickers marked), built by breadth-first
     search with 4-bit entries and cached like the 5x5x5 tables.
   - The steps over many small tables (6x6x6: steps 50 and 60; 7x7x7: steps 40,
     50 and 60) use hash tables built from the goal states the reference lists
     (343,000 states each, or 72 or 2 for the daisy tables). They are built in
     memory when the stage starts and not cached.
   - The oblique pairing stages of the 7x7x7 have no table: the heuristic is the
     reference's count of unpaired obliques, mapped to a move estimate by the
     reference's own table (inadmissible, as in the reference).
2. **Wings.** The inner wings of the 6x6x6 go through a 4x4x4 cut from the
   corners and the inner wings; the outer wings of the 6x6x6 and both orbits of
   the 7x7x7 (orbit 1 first) through 5x5x5 cubes. The engines are used only
   through the `Solver` trait with `reduce_only`; a move of the fake cube is
   lifted to the layers it stands for (a slice of the fake cube is a block of
   layers of the big one, so it becomes a wide turn followed by an outer turn
   back). The 4x4x4 engine keeps the frame of the cube in `reduce_only`, because
   the centers of the big cube must not turn as a whole.
3. **Finish.** The corners and one wing of each edge form a 3x3x3 that
   `Min2PhaseSolver` solves.

The solver ignores `maxMoves`, `budgetMs` and `slices`: there is no search for a
shorter answer yet.

## Validation (measured)

20 reference cubes per size (`tests/dwalton_test.rs`, run with
`--ignored`; every solution is replayed and ends solved). Mean moves per stage,
with the reference's mean over the same cubes:

| 6x6x6 stage | here | reference |
| --- | --- | --- |
| LR inner x-centers staged, obliques paired | 9.60 | 9.65 |
| LR centers staged | 10.20 | 10.0 |
| UD inner x-centers staged, obliques paired | 9.00 | 9.1 |
| UD centers staged | 9.55 | 9.6 |
| step 50 (LR reducible) | 6.55 | 8.55 |
| step 60 | 13.25 | 13.6 |
| inner wings (4x4x4) | 19.05 | 16.05 |
| outer wings (5x5x5) | 37.15 | 37.8 |
| 3x3x3 | 20.55 | 20.9 |
| total | 133.6 | 135.2 |

| 7x7x7 stage | here | reference |
| --- | --- | --- |
| LR inner centers staged | 9.85 | 9.85 |
| LR obliques paired | 12.55 | 12.2 |
| LR centers staged | 10.30 | 10.1 |
| UD inner centers staged | 9.80 | 9.65 |
| UD obliques paired | 10.70 | 11.05 |
| UD centers staged | 9.10 | 9.7 |
| step 40 (LR vertical bars) | 10.10 | 9.55 |
| step 50 | 11.00 | 11.3 |
| step 60 | 12.95 | 12.75 |
| inner wings (5x5x5, orbit 1) | 37.20 | 36.95 |
| outer wings (5x5x5, orbit 0) | 37.20 | 38.05 |
| 3x3x3 | 20.60 | 20.7 |
| total | 189.05 | 191.8 |

Step 50 of the 6x6x6 is shorter here because the edge orientation of the inside
edges is not required (the wings are paired by an engine); the later stages are
not longer for it.

## Stages in the answer

`SolveResult::stages` carries six stages that group the steps above (the table
is in [Move icons](../icons.md#phases)). The engine records the length of the
raw move list at the end of each step, and the ends of the six stages are then
matched to the normalized answer by the cube state they leave. The match is
by colors, so a stage can end a few moves before its raw end when the
rewriting has shortened the moves before it. On eight random 6x6x6 and eight
7x7x7 cubes, 79 of the 80 ends found a state of the final answer equal to the
engine's; the last took the closest state (see `stage_ends`). On three cubes of
each size the state-based phases ended at the end of the wing stages
(`inner wings paired` or `middle wings paired`, and `outer wings paired`).

## Phases

The six stages are named `LR centers on L/R`, `UD/FB centers on their axes`, `inner centers solved` (7x7x7: `centers solved`), `inner wings paired` (`middle wings paired`), `outer wings paired, centers solved` (`outer wings paired`) and `3x3 stage`. A stage over 20 moves is cut at the ends of the reference's steps; the wing stages are cut at the stages of the 4x4x4 or 5x5x5 engine that makes them, lifted to the big cube. The ends are matched to the normalized answer by cube state; a stage end is moved at most four moves later where normalizing merged moves across it, and the end before the 3x3 stage is moved behind the last slice or wide turn. Before, the repair ran on to the end of the answer when no state matched, and a 7-move scramble of a 6x6x6 had all 52 moves in one phase (`tests/dwalton_test.rs` checks it). The step lengths in the table above are unchanged.

## Differences from the reference

- Tables are searched, not read: no 7.3 GB of lookup files. The 5x5x5 edge
  tables of the reference (about 4.5 GB) are not ported; the 5x5x5 engine of this
  repository pairs the wings.
- The reference counts wide quarter turns to control OLL parity in some center
  stages. No cube of the 40 validation cubes needed it, and the center stages
  here do not track it.
- Moves are the repository's: outer, wide (two layers) and three-layer wide
  turns; the 7x7x7 results are normalized by `normalize`.

## Open

- Times, memory and table sizes on an idle machine (not yet measured).
- Inner wings of the 6x6x6: 3 moves over the reference, from the closing
  rotation of the 4x4x4 reduction.
- A shorter search for the centers (beam instead of IDA*), and `improve`.
