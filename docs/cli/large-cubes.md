---
title: "Solve a 4x4x4 or 5x5x5"
description: "Solving the 4x4x4 and 5x5x5 from the command line."
weight: 3
---

# Solve a 4x4x4 or 5x5x5

## Solve a 4x4x4

Give `solve` a 96-sticker cube, or a scramble with `--size 4`. A 4x4x4 scramble may turn the first two layers of a face (`R`, `2R`) and use wide turns (`Rw`); the 4x4x4 has no fixed centers, so the solver brings the cube to the standard color scheme and the solution may end with whole-cube rotations.

```sh
cubacle solve --size 4 --scramble "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw"
```

```text
Uw' Rw' Uw L' Fw U' B2 Rw' D B Uw2 Rw' Fw2 R2 Rw2 U F Fw2 L2 U' D' Rw2 F' Uw2 L U L' D' B L U2 L' D' F2 B2 U' L2 F2 U2 F2 D' L2 B Rw' Lw Fw Bw'
```

The solution has 43 moves and needs no rotation. Some answers end with up to four whole-cube turns such as `Rw' Lw Fw Bw'`: each pair of opposite wide turns, such as `Rw' Lw`, turns the whole cube, and at most four turns bring any cube to the standard orientation. The solutions of 1000 random cubes have 47.5 moves on average (44.3 without the rotation). The tool replays every 4x4x4 solution on the cube and exits with an error if it does not solve it. With a move limit the solution cannot meet, `solve` fails:

```sh
cubacle solve --size 4 -s "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" -m 30
```

```text
error: no solution within the move limit (3x3x3 stage: the search ended without a solution (time budget or move limit reached))
```

The first 4x4x4 solve prints `preparing the 4x4x4 tables (built now, about 12 MB)` on stderr and builds the tables, which takes about 0.2 s on 12 cores and 0.4 s on one; there is no cache directory. The preparation happens before the time budget starts, so `-b 1` fails with `the time budget ran out (the reduction ran out of time before the 3x3x3 stage)`. `CUBACLE_THREADS` sets the number of threads (at most 256); the solution does not depend on it unless a time budget cuts the search short. `-v` prints the time and the number of probes of the 3x3x3 stage (500: that stage is asked for at least 500 probes, as in the original solver).

## Solve a 5x5x5

Give `solve` a 150-sticker cube, or a scramble with `--size 5`. A 5x5x5 scramble may turn the first two layers of a face (`R`, `2R`); the middle layer (`3R`, `3Rw`) moves the face centers, and the cube is then solved by the color of its centers (see [Face centers off their faces](solve-options.md#face-centers-off-their-faces)). Two-layer turns (`Rw`) never move them.

```sh
cubacle solve --size 5 --scramble "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F" | wc -w
```

```text
67
```

The solutions have 73 moves on average: 70 to 76 over 100 random cubes and 69 to 77 over 300 others (see [Engines and limits](../engines/_index.md)). It starts like this:

```text
Fw' D F' Rw U B' Uw' Lw' U Bw R2 Fw' Uw2 D Fw'
```

The tool replays every 5x5x5 solution on the cube and exits with an error if it does not solve it. With a move limit the solution cannot meet, `solve` fails:

```sh
cubacle solve --size 5 -s "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F" -m 20
```

```text
error: no solution within the move limit (the reduction alone is longer than the move limit)
```

The first 5x5x5 solve prints `preparing the 5x5x5 tables (cache directory ...)` on stderr and then reads the tables from the cache directory, or builds them and saves them there (3.3 s on 12 cores, 18 s on one, about 1 s to read them again). If the tables cannot be saved it prints `warning: the tables could not be saved; they are built again next time`. The preparation happens before the time budget starts, so a later `-b 1` fails with `the time budget ran out`, followed by `(the reduction ran out of time before the 3x3x3 stage)` or `(the reduction used up the time budget before the 3x3x3 stage)`, depending on where the clock ran out. `CUBACLE_THREADS` sets the number of threads (at most 256). Without `-b`, the 3x3x3 stage gets at most 10 s; a tight `-m` that it cannot meet in that time ends with `no solution within the move limit (no 3x3x3 solution within the move limit was found in 10 s ...)`.
