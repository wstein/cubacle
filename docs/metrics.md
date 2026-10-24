---
title: "Move metrics"
description: "How cubacle counts moves: HTM, QTM, STM, ETM, OBTM, BTM and ATM, what each engine minimises, and the metrics lines on stderr."
weight: 5
---

# Move metrics

The length of a solution depends on how moves are counted. `R2` is one move to a person who turns a face, two to a person who counts quarter turns, and `M` on a 3x3x3 is one slice turn or two face turns. cubacle prints its own count (`moves N`) and, since the metrics were added, the counts in the metrics people compare solutions by. This page defines each one for the move model of the tool, says where the usual sources differ and what is chosen here, and states honestly which of them each solver works on.

## What the commands print

Every command that prints moves adds one line to standard error after `moves N`. Standard output is not changed.

```text
$ cubacle solve -s "R U R' U' F2 D L2 B'"
moves 8
metrics: htm 8, qtm 10, stm 8
B L2 D' F2 U R U' R'
```

The line shows the four counts most useful for the cube: `htm qtm stm` for the 2x2x2 and 3x3x3, and `obtm qtm stm btm` from the 4x4x4, where wide turns and inner layers make the block counts differ. `moves N` is unchanged.

| Command | Where the line appears |
| --- | --- |
| `solve` | after `moves N`; with `--optimise ranges` a second line `compressed metrics: ...` follows `compressed moves M` (not with `--group phases`) |
| `solve --all`, `--count` | once, before `solutions N`: `metrics over the listed solutions: htm 20, qtm 24-26, stm 20`, the smallest and the largest count of the solutions listed (several solutions of one length differ in QTM) |
| `recover` | `fresh metrics: ...` for the fresh solution, `recover route metrics: ...` for the recommended way back (with `--fresh` a plain `metrics:` line for the solution) |
| `patterns find` | after `moves N` |
| `patterns show` | one line after each generator it prints |
| `moves` | after the sequence, counted on the words as typed (see [Words, not expansions](#words-not-expansions)) |
| `moves --metrics`, `convert --metrics` | all seven counts: `metrics: htm 4, qtm 5, stm 3, etm 4, obtm 4, btm 3, atm 1` |

The REST solve answer always carries all seven counts as `metrics`; see [Move counts in the API](api.md#move-counts).

## The model

All metrics count turns of a block: a set of contiguous layers on one axis, turned together by one quarter turn or one half turn. The move forms of cubacle map onto blocks like this (3x3x3 examples; layers are counted from the face named):

| Written | Block |
| --- | --- |
| `R`, `R'`, `R2` | the outer layer of R |
| `2R` (4x4x4 and up) | the second layer from R, alone |
| `Rw`, `r` | the two outer layers of R |
| `3Rw` | the three outer layers |
| `2-3Rw`, `2-3r` | layers 2 and 3 |
| `M`, `E`, `S` (odd cubes) | the middle layer of the L-R, U-D or F-B axis |
| `x`, `y`, `z` | every layer of the axis: a whole-cube rotation |

A block that holds every layer is a rotation. A block that holds an outer layer of its axis "touches the outside"; any other block is "inner" (`M`, `2R`, `2-3Rw` on a 5x5x5).

## The metrics

`q` is 1 for a quarter turn and 2 for a half turn. A rotation is free in every metric except ETM.

| Metric | Counts | Rotation | `R2` | `Rw` (3x3x3) | `M` (3x3x3) |
| --- | --- | --- | --- | --- | --- |
| HTM (also FTM) | a block that touches the outside is 1, an inner block is 2; the angle does not matter | 0 | 1 | 1 | 2 |
| QTM | the HTM count times `q`: a half turn is two quarter turns | 0 | 2 | 1 | 2 |
| STM (also LTM) | the number of layers in the block; the angle does not matter | 0 | 1 | 2 | 1 |
| ETM | every block turn and every rotation is 1, the way a hand executes it | 1 | 1 | 1 | 1 |
| OBTM | as HTM, under the name used for big cubes | 0 | 1 | 1 | 2 |
| BTM | every block is 1, the angle does not matter | 0 | 1 | 1 | 1 |
| ATM | a run of consecutive turns about one axis is 1, whatever the layers and directions | 0 | 1 | 1 | 1 |

HTM and OBTM are the same rule here and always give the same number; `htm` is the name people use on the 2x2x2 and 3x3x3, `obtm` the one on bigger cubes, and both are accepted. `ftm` and `ltm` are accepted as aliases of `htm` and `stm`.

ATM: a rotation about the axis of the surrounding run joins that run; a rotation about another axis ends it. So `R x R'` is 1 (x turns about the R-L axis), `R y R'` is 2.

### Verified examples

Produced with `cubacle moves --metrics --size N "..."` (output of the commands, not computed by hand):

| Size | Moves | htm | qtm | stm | etm | obtm | btm | atm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 3 | `R U R' U'` | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| 3 | `R2` | 1 | 2 | 1 | 1 | 1 | 1 | 1 |
| 3 | `R L'` | 2 | 2 | 2 | 2 | 2 | 2 | 1 |
| 3 | `R L' R2` | 3 | 4 | 3 | 3 | 3 | 3 | 1 |
| 3 | `Rw` | 1 | 1 | 2 | 1 | 1 | 1 | 1 |
| 3 | `M` | 2 | 2 | 1 | 1 | 2 | 1 | 1 |
| 3 | `M2` | 2 | 4 | 1 | 1 | 2 | 1 | 1 |
| 3 | `x y2 R` | 1 | 1 | 1 | 3 | 1 | 1 | 1 |
| 3 | `R x R'` | 2 | 2 | 2 | 3 | 2 | 2 | 1 |
| 3 | `R y R'` | 2 | 2 | 2 | 3 | 2 | 2 | 2 |
| 4 | `Rw L'` | 2 | 2 | 3 | 2 | 2 | 2 | 1 |
| 5 | `2R` | 2 | 2 | 1 | 1 | 2 | 1 | 1 |
| 5 | `3Rw` | 1 | 1 | 3 | 1 | 1 | 1 | 1 |
| 5 | `2-3Rw` | 2 | 2 | 2 | 1 | 2 | 1 | 1 |
| 5 | `2-3Rw2` | 2 | 4 | 2 | 1 | 2 | 1 | 1 |
| 5 | `2-4Rw` | 2 | 2 | 3 | 1 | 2 | 1 | 1 |
| 5 | `M E` | 4 | 4 | 2 | 2 | 4 | 2 | 2 |
| 5 | `R 2L 3Lw'` | 4 | 4 | 5 | 3 | 4 | 3 | 1 |

### Where sources differ, and the choice made

- **Half turns in QTM.** All sources agree a half turn is two quarter turns. Kept.
- **Inner blocks in HTM and OBTM.** On the 3x3x3 every source counts a slice turn (`M`) as two face turns. For a bigger cube the rule is generalised: an inner block is two outer-block turns (the outer `b` layers, then the outer `a-1` layers back). Block moves that touch the outside count 1.
- **Wide turns in STM.** Some sources count `Rw` as one slice turn, others as the number of layers it moves. The layer count is used (2 for `Rw`, 3 for `3Rw`): it never undercounts, and a turn of `n-1` layers is not shortened to the single opposite layer, because that would be a change of orientation the move as written does not make.
- **Rotations.** Metrics that exist for the shortest-path question (HTM, QTM, STM, OBTM, BTM) ignore rotations; ETM, which models execution, counts them as one each. ATM ignores them as moves but lets them sit in a run of their own axis.
- **Middle slices on even cubes.** `M`, `E` and `S` are refused on even cubes, as everywhere in the tool.
- **Whole-cube-equivalent blocks.** A block of `n-1` layers turns like the single opposite layer plus a rotation. It is counted as written, never as that equivalent.
- **Names.** HTM = FTM = OBTM here, STM = LTM; the aliases `ftm` and `ltm` are accepted on input only.

### Words, not expansions

The expression reader turns `M`, `x` and a range such as `2-3Rw` into several plain layer moves (`x` on a 3x3x3 is `Rw L'`). Counting those would count one turn as two. `moves` and `convert --metrics` therefore count the words as typed whenever the input is a flat list of WCA or SiGN words, middle slices, rotations and ranges. Text with groups, repeats, commutators or SSE words is counted after expansion into plain moves, which can count rotations and slices as the layer moves they stand for.

The output of a solver never contains rotations, slices or ranges unless `--optimise ranges` is given, so its counts need no care.

## What each engine works on

`moves N` is the number of plain moves of the answer with no merging: each of `R`, `R2`, `2R`, `Rw` counts 1. On solver answers that equals BTM (and ETM, as an answer holds no rotation). It equals OBTM and HTM only where the answer has no single inner layer such as `2R`, which counts 2 there: the 4x4x4 writes none unless `--slices` is given, the 5x5x5 and larger write second layers by default. The solvers do not know the other metrics.

| Engine | What it works on | Measured (mean over cubes) |
| --- | --- | --- |
| 2x2x2 | optimal in HTM (exact distance table) | 20 cubes: htm 8.55, qtm 11.00 |
| 3x3x3 | two-phase search over the 18 face turns; shortest answers come from `--improve`, but none is proven HTM-optimal | 20 cubes: default htm 20.70, qtm 30.90; `--improve -b 300`: htm 18.30, qtm 25.40 |
| 4x4x4 | reduction; counts blocks and the 3x3x3 tail, no metric is minimised | 4x4x4, 20 cubes: moves 44.00, obtm 44.00, btm 44.00, qtm 62.30, stm 54.10 |
| 5x5x5 | reduction, as above | 8 cubes: moves 67.88, btm 67.88, obtm 68.75, qtm 104.25, stm 92.75 |
| `--moves qtm` | the default answer written as quarter turns, merged, not proven shortest | e.g. 25 quarter turns where `--improve` gives 11 |

So `solve` never claims "shortest in QTM" or "shortest in STM". Where it says shortest (the 2x2x2, `--moves` sets other than `qtm`, `--all`) it means the fewest moves of that engine's own alphabet, and the guide says so there; the 2x2x2 alphabet is the half-turn one, so that answer is HTM-optimal. A solution that is short in HTM can be long in QTM: the 3x3x3 mean above is 20.70 HTM and 30.90 QTM.

Measured with `cubacle solve --size N -s "<30 scramble moves>"` on fixed pseudo-random scrambles; the `metrics:` line gave the counts.

## Writing a sequence for a metric

| Form | Command | Best for |
| --- | --- | --- |
| plain moves (default) | `moves`, `solve` | HTM and OBTM (a half turn is one move) |
| every half turn as two quarter turns | `moves --expand quarter` | QTM: every written move is one count, so the number of words is the QTM count (R2 becomes R R) |
| runs of one axis as layer ranges | `moves --optimise ranges`, `solve --optimise ranges` | BTM and ETM on big cubes: `3Rw R'` becomes `2-3Rw`, one block turn |

`--expand quarter` is exact and costs nothing: `R2` and `R R` turn the same layer by the same angle. It writes only quarter turns, so the HTM count of the output is its QTM count. It does not combine with `--optimise ranges`, which writes half turns of a block as one move (`2-3Rw2`). `convert` prints a cube, not moves, so it has nothing to rewrite; its `--metrics` counts an input expression. `solve` has no `--expand` (follow-up).

Neither form searches: the sequence stays the one the solver found, so no form makes a solution shorter in a metric it was not found in.

## For library users

`cubacle::metrics` holds `Metric` (`FromStr`, `Display`), `count(&MoveSequence, PuzzleKind, Metric)`, `count_layer_moves` for compressed sequences, `count_text` and `count_all_text` for move words, `count_all` and the `summary` helpers. It depends on `puzzle` only.
