---
title: "Picture cubes"
description: "Solve a 3x3x3 whose face centers carry a picture: center twists, upright centers and twist targets."
weight: 3
---

# Picture cubes

On a picture cube (a calendar cube, a photo cube) the six face centers show a picture that has an up direction. Turning a face turns its picture, so a cube can be solved in color and still show a picture on its side. This page describes how `cubacle solve` handles the twist of the six centers on a 3x3x3. Everything here is opt-in: without the options below `solve` prints what it printed before.

## The twist of a center

The twist vector `t = (U, R, F, D, L, B)` has one number 0 to 3 per center: how many clockwise quarter turns, seen from outside the face, the picture is turned from its solved orientation. In the solved orientation the top of the picture of U points to the back, the top of D to the front and the top of the four side pictures up (the top of the cross net).

- A move `f` adds 1 to the twist of face `f`, `f2` adds 2, `f'` adds 3. No other center is touched by an outer face turn.
- The sum of the six numbers has the parity of the corner permutation of the cube. An impossible pair is refused (`the center twist does not belong to this cube`).
- A whole-cube rotation (`x`, `y`, `z`) costs no move. A scramble whose centers end off their faces is turned as a whole back into the frame where they are home; stderr names the rotation (`note: ... turned as a whole (x x x) first`).

The text form is six numbers separated by commas in the order `U,R,F,D,L,B`, like `0,1,2,0,3,0`.

## Options

| Option | Meaning |
|---|---|
| `--upright-centers` | The target is the solved cube with all six pictures upright (twist 0 everywhere). Not with `--to` or `--twist-target`. |
| `--center-twist T` | The twist of the six centers of the cube you give as stickers, like `0,1,2,0,3,0`. A scramble derives its own twist and refuses this option. |
| `--twist-target T` | A condition on the twists in the target: six items for `U,R,F,D,L,B`, each a twist 0 to 3, several joined by `\|` (`0\|2`), or `X` for any: `0,X,2\|3,X,X,X`. Works with or without `--to`; without it the stickers must be solved. With `--any-orientation` the conditions turn with the cube. |

A cube given as stickers does not say how its pictures sit, so `--upright-centers` and `--twist-target` need `--center-twist` for it; a scramble gives the twist by itself. The search is the exact search of [target sets](solve.md#target-sets-proven-shortest-answers) with the twists as one more coordinate; `-b`, `-m` and `--max-memory` apply, and the table build is not counted in `-b`. With a twist condition, `--moves`, `--all`, `--count`, `--now` and `--within` are refused, and so are `--json`, `--icons`, `--improve` and `--nodes` (as for other target sets).

```sh
cubacle solve -s "R U2 F' D L2 B" --upright-centers
cubacle solve -s "x R U" --upright-centers
cubacle solve -s R --twist-target X,0,X,X,X,X
```

The answers are `B' L2 D' F U2 R'` (the pictures start at `2,1,3,1,2,1`), `F' R'` (the `x` is turned back first; the start twist is `0,1,1,0,0,0`) and `R'`. Stderr shows the twist at the start and at the end and the usual `shortest: proved by ...` line. Each answer is replayed by the command itself, apart from the engine: the stickers through the facelet model and the twists through the center tracker.

## How far the proof reaches

The answer is proven shortest, or the result is an error that names the depth proven empty (`not proven: no solution has fewer than N moves, but the search of N moves did not finish in ... ms`). A random picture cube is far beyond the search, because the pictures add a long tail to the moves the pieces need: expect `not proven` unless the scramble is short, or give much more time with `-b`.

## Limits

3x3x3 only. A picture on a center is the only extra state; edges and corners have no picture. A twist condition needs a start whose centers are on their faces (a scramble with rotations or slices is turned home first). Sticker-level identities (the calendar day on a front face) are not expressed by the command line. The replayable record of the proof (`--certificate`) is not built yet.
