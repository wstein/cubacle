---
title: "Patterns"
description: "Well-known cube patterns for the 2x2x2 to 7x7x7, defined by their structure, with generators found by our own search."
weight: 8
---

# Patterns

`cubacle patterns` lists the pretty patterns the library knows; `cubacle patterns show NAME` prints one with its generators. A pattern is defined by its structure (a colour for every sticker as a function of the cube size), never by a move list. Each generator was found by our own search from that definition and is replayed against it by the test suite (`src/patterns/tests.rs`).

```sh
cubacle patterns --size 3
```

```text
id	name	sizes (outer/range moves)
checkerboard	Checkerboard	3:6/3
six-spots	Six spots	3:8/6
four-spots	Four spots	3:8/6
cube-in-cube	Cube in a cube	3:15/15
superflip	Superflip	3:20/20
```

```sh
cubacle patterns show chessboard --size 3
```

```text
Checkerboard (checkerboard)
also: chessboard, chess pattern
Every face alternates its own colour and the opposite colour; the corners stay solved.
3x3x3
  outer metric, 6 moves, optimal:
    F2 Fw2 Rw2 R2 Uw2 U2
  range metric, 3 moves, optimal:
    2U2 2R2 2F2
```

Names, spaces, case and hyphens are not significant (`Cube-in-a-Cube` finds `cube-in-cube`).

## Metrics and status

- **outer**: every move turns the outer `k` layers of one face (`R`, `Rw`, `3Rw`); a slice costs two moves.
- **range**: any block of adjacent layers is one move (`2-3Rw`, `2R`). Every outer sequence is also a range sequence, so the range length is never above the outer length.

| Status | Meaning |
| --- | --- |
| `optimal` | An exhaustive bidirectional search over quarter and half turns found nothing shorter in that metric. |
| `half-turn-optimal` | Exhaustive over half turns only. |
| `best-known` | The best found: a search inside a smaller move set (mirror-symmetric layer sets, face turns only), or the inverse of a solver's answer. Shorter ones may exist. |

On even cubes, which have no fixed centres, a pattern is accepted in any orientation.

## The library

Lengths are `outer/range`; `-` means no generator of that kind was found. All lengths are move counts in the metric above.

| Pattern | Definition |
| --- | --- |
| Checkerboard | The colour of a sticker is its own on cubies whose distances from the nearest face sum to an even number, the opposite colour otherwise. |
| Six spots | Every face keeps a block of its own colour in the middle inside a border of the next colour round a corner. |
| Six centre spots | The same with the spot reduced to the middle (one sticker, or 2x2 on even cubes). |
| Four spots | Spots on four faces round an axis on a border of the opposite colour, two faces solid. |
| Four centre spots | The same with the spot reduced to the middle. |
| Cube in a cube | At two opposite corners every face has a block of its own colour; the L-shaped rest shows the next colour round a corner. |
| Cube in a cube in a cube | Three nested shells. |
| Superflip | Every edge flipped in place, everything else solved. |
| Opposite centres | Everything solved except the centres, which show the opposite colour. |
| Cycled centres | Everything solved except the centres, which show the next colour round a corner. |

The generated list (`cubacle patterns`):

```text
id	name	sizes (outer/range moves)
checkerboard	Checkerboard	3:6/3 4:6/3 5:12/6 6:12/6 7:-/9
six-spots	Six spots	3:8/6 4:8/4 5:8/6 6:8/4 7:8/8
six-centre-spots	Six centre spots	5:8/6 6:8/4 7:-/8
four-spots	Four spots	3:8/6 4:8/4 5:8/6 6:8/4 7:8/6
four-centre-spots	Four centre spots	5:8/6 6:8/4
cube-in-cube	Cube in a cube	2:10/10 3:15/15 4:15/15 5:15/15 6:15/15 7:15/15
cube-in-cube-in-cube	Cube in a cube in a cube	5:44/44
superflip	Superflip	3:20/20 4:20/20 5:20/20 6:20/20 7:20/20
centres-opposite	Opposite centres	4:14/6 6:-/8
centres-cycled	Cycled centres	4:8/4 5:8/4 6:8/4
```

Not every pattern exists on every size or was found within the search budget; the missing entries are open, not impossible. The nested cube in a cube has only a long solver-derived generator (5x5x5); the 6x6x6 and 7x7x7 versions, the opposite centres on the 5x5x5 and 7x7x7 and the 2x2x2 analogues other than the cube in a cube were not found. Lists such as Tetris, Anaconda or Pons asinorum are not included because we could not state their structure with certainty.

## How the generators were found

`examples/pattern_search` runs a bidirectional search over cost levels from the solved cube and from the pattern (in every orientation on even cubes) and replays its answer with the library before printing it:

```sh
cargo run --release --example pattern_search -- checkerboard 3 range half 8 60
```

Modes: `outer`, `range`, the symmetric macro modes `sym-outer` and `sym-range` for the big cubes, and `face` (single outer-layer turns, replayed on the bigger cubes). `examples/pattern_solve` inverts a solver's solution for 3x3x3 to 5x5x5 (`best-known`). A search stops at `MAX_STATES` stored states (default 5,000,000); with 2,000,000 the 5x5x5 range search for six spots used 0.45 GB of resident memory (`/usr/bin/time -l`), so deep searches stop and leave the pattern at `best-known` rather than grow.

The generators were found from the definitions only; no move list of any collection was read or copied.
