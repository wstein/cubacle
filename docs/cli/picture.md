---
title: "Picture cubes"
description: "Solve a 3x3x3 whose face centers carry a picture: center twists, picture targets and certificates."
weight: 3
---

# Picture cubes

On a picture cube (a calendar cube, a photo cube) the six face centers show a picture that has an up direction. Turning a face turns its picture, so a cube can be solved in color and still show a picture on its side. This page describes how `cubacle solve` handles the twist of the six centers on a 3x3x3, and how to ask for a record of the proof. Everything here is opt-in: without the options below `solve` prints what it printed before.

## The twist of a center

The twist vector `t = (U, R, F, D, L, B)` has one number 0 to 3 per center: how many clockwise quarter turns, seen from outside the face, the picture is turned from its solved orientation. In the solved orientation the top of the picture of U points to the back, the top of D to the front and the top of the four side pictures up (the top of the cross net).

- A move `f` adds 1 to the twist of face `f`, `f2` adds 2, `f'` adds 3. No other center is touched by an outer face turn.
- The sum of the six numbers has the parity of the corner permutation of the cube: half of the 4096 vectors (2048) belong to a given arrangement of the pieces. An impossible pair is refused (`the center twist does not belong to this cube`).
- A whole-cube rotation (`x`, `y`, `z`) costs no move. It carries the centers to other faces and turns their pictures by a fixed amount, which the tracker computes from the geometry of the picture's up direction. A scramble with rotations or middle slices that leaves the centers off their faces is turned as a whole back into the frame where they are home (stderr names the rotation); the rotation costs no move and stays in the certificate.

The text form is six numbers separated by commas in the order `U,R,F,D,L,B`, like `0,1,2,0,3,0`. The convention has a version, `center-twist-1`, printed in certificates.

## Options

| Option | Meaning |
|---|---|
| `--picture` | The target is the solved cube with all six pictures upright (twist 0 everywhere). Not with `--to` or `--twist-target`. |
| `--center-twist T` | The twist of the six centers of the cube you give as stickers, like `0,1,2,0,3,0`. A scramble derives its own twist and refuses this option. |
| `--twist-target T` | A condition on the twists in the target: six items for `U,R,F,D,L,B`, each a twist 0 to 3, several joined by `\|` (`0\|2`), or `X` for any: `0,X,2\|3,X,X,X`. Works with or without `--to`; without it the stickers must be solved. With `--any-orientation` the conditions turn with the cube. |
| `--certificate` | Print a replayable record on standard output after the moves, also when no answer is proven. |
| `cubacle verify FILE` | Replay a certificate (`-` reads standard input). |

A cube given as stickers does not say how its pictures sit, so `--picture` and `--twist-target` need `--center-twist` for it; a scramble gives the twist by itself. The search is the exact search of [target sets](solve.md#target-sets-proven-shortest-answers) with the twists as one more coordinate; `-b`, `-m` and `--max-memory` apply, and the table build is not counted in `-b`.

```sh
cubacle solve -s "R U2 F' D L2 B" --picture
cubacle solve -s "x R U" --picture
cubacle solve -s R --twist-target X,0,X,X,X,X
```

The answers are `B' L2 D' F U2 R'` (the pictures start at `2,1,3,1,2,1`), `F' R'` (the `x` is turned back first; stderr says so, and the start twist is `0,1,1,0,0,0`) and `R'`. Stderr shows the twist at the start and at the end and the usual `shortest: proved by ...` line.

## How far the proof reaches

The answer is proven shortest, or the result is an error that names the depth proven empty (`not proven: no solution has fewer than 15 moves, but the search of 15 moves did not finish in 20000 ms ...`). A random picture cube is far beyond the search: its pieces alone need about 18 moves, and fixing the pictures of a cube whose pieces are home costs a long tail (a twist vector that needs no piece to move takes 10 moves or more in the measurement spike), so a random state needs about 37 moves in all. Measured in release on one core of an Apple M2 Pro, one process at a time, 109 MB of tables (corners, six edges and the 4096 twist states), 123 MB peak, 5.6 s of table build outside `-b`:

| Scramble | Result | Nodes | Search time |
|---|---|---|---|
| 8 moves (3 cubes) | the scramble undone in 8 | 9 to 56 | under 0.1 s |
| 12 moves (3 cubes) | 12 | 18,000 to 52,000 | under 0.2 s |
| 13 moves | 13 | 370,000 | under 0.5 s |
| 14 moves | 13 | 121,000 | under 0.5 s |
| 14 moves | 14 | 4.9 million | 2.4 s |
| 15 moves (2 cubes) | 15 and 15 | 28.5 and 91.4 million | 12 s and 42 s |
| 16 moves (2 cubes, `-b 120000`) | not proven, nothing under 16 moves | 241 and 248 million | 120 s |
| random, 25-move scramble (`-b 20000`) | not proven, nothing under 15 moves | 45.6 million | 20 s |

So a picture cube is solved with a proof when the scramble has up to about 15 moves; beyond that give much more time with `-b` or accept `not proven`. The search runs about two million nodes per second. The twist table saves 1 to 2 percent of the nodes at 12 to 14 moves (it is 4 KB and costs nothing); the piece tables decide the search. A plain `solve` of the same cubes with `--center-twist` ignored is not an answer for a picture cube: it solves the colors only. A fallback that finishes a plain solution with the shortest tail is not built: the table of tails of the measurement spike took hours to compute and cannot be rebuilt from the committed code.

A target that cares about few centers is cheap. The calendar use case puts a date on the front face and asks for the F picture upright. As a stand-in for the date (one view of colors; the sticker identities of a real date are not on the command line) take the F face of a spike cube (stickers `UUUUUULFURRRFRRRRBUUFFFRFFFDDDDDDDDRLLFLLLLLLBBBBBBDBB`, start twist `1,2,0,0,3,0`). The plain target-set search (`--optimal --to` the F face) answers `R2 U2 R' U B' R'`, 6 moves, 217 nodes. With `--twist-target X,X,T,X,X,X` the answer has 6 moves for T = 0 (and `0|2`), the same moves and nodes, and 7 moves for T = 1, 2 and 3 (1990, 854 and 1990 nodes). Every run took 2.2 to 2.3 s, almost all of it the table build, and 55 MB peak; the twist condition costs nothing measurable.

## The certificate

`--certificate` prints, after the moves, plain text of `key: value` lines (format `cubacle-certificate 1`):

```text
cubacle-certificate 1
convention: center-twist-1 (t=URFDLB, clockwise quarters seen from outside, up: U back, D front, sides up)
puzzle: 3x3x3
start: LRFDUUBRRBLLDRBUFLRDDBFFDLFFDRFDLBUUUBUULLDRRDUFRBFBBL
start-twist: 2,1,3,1,2,1
target: UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
target-twist: 0,0,0,0,0,0
orientation: fixed
moves: B' L2 D' F U2 R'
length: 6
final-rotation:
end: UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
end-twist: 0,0,0,0,0,0
result: shortest, searched out below its length
views: 1
lower-bound: 6
found: 6 7
table: corners 88179840 fnv1a64:9ca955289cfde19b
table: edges6 42577920 fnv1a64:5a214e51159ac5b7
table: twist 4096 fnv1a64:93c9db04a3b11fe5
```

- `frame:` (only if present) is the zero-cost rotation applied to the given cube before the search; `start` is the cube after it. `final-rotation:` is the zero-cost rotation after the last move (non-empty only with `--any-orientation`). Rotations are never moves, so `length` counts the face turns only.
- `lower-bound` is the largest entry of the pruning tables at the start. `searched-out: D N` lines (one per depth from the bound up to `length - 1`) give the nodes of every iteration that found nothing, `found: D N` the iteration that found the answer. The table digests are FNV-1a 64 of the table contents, so a second program that builds the same tables can compare them.
- A failed search (`not proven`) prints the same record with `result: not proven, ...`, the lower bound and the depths searched out so far, and no moves; the exit status is the usual one for a limit.

`cubacle verify` replays the stickers through the facelet model and the twists through the center tracker, and compares the end state, the target and the claimed length; it also checks that the searched-out depths run without a gap from the lower bound to the length. It does not rebuild the tables, so shortness is checked for consistency only. An independent checker is expected to recompute the state, the twists and the moves itself from the `start`, `start-twist`, `target`, `target-twist`, `moves` and `final-rotation` lines.

```sh
cubacle solve -s "R U2 F' D L2 B" --picture --certificate | tail -n +2 | cubacle verify -
```

prints `valid: 6 moves reach the target, twist 2,1,3,1,2,1 to 0,0,0,0,0,0; ...`.

## Limits

3x3x3 only. A picture on a center is the only extra state; edges and corners have no picture. Sticker-level identities (the calendar day on a front face) are not expressed by the command line: the engine takes a list of views with the twists each center may show, and the command line gives one view of colors plus the twist condition. `--moves`, `--all`, `--count`, `--json`, `--icons`, `--improve` and `--nodes` are refused with a target that has open stickers or a twist condition, as for other target sets.
