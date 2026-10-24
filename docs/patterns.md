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

## Finding a generator

`cubacle patterns find` looks for the shortest known sequence that builds a cube from the solved cube. The cube is given like the input of `solve` (or with `-s`), on the size of `--size` (3 unless given):

```sh
cubacle patterns find --size 5 --moves half-turns -s "R2 3R2 L2 U2 3U2 D2 F2 3F2 B2"
cubacle patterns find --size 7 --max-memory 200 -s "R2 3R2 5R2 L2 U2 3U2 5U2 D2 F2 3F2 5F2 B2"
```

```text
moves 6
shortest: no sequence of fewer moves exists (searched exhaustively)
2F2 4F2 4U2 2U2 4R2 2R2
moves 12
best known, a shorter one may exist (the exact search reached the memory cap (200 MB))
R2 3R2 L2 3L2 U2 3U2 D2 3D2 F2 3F2 B2 3B2
```

It first searches products of macros (from the 4x4x4 up), then an exact bidirectional search over plain moves for something shorter. `--moves SET` restricts the answer to a move set (see `solve --moves`); a macro is used only when its moves and those of its inverse are all in the set. `--max-memory MB` (default 1500) is a hard cap on the searches; a search that reaches it says so, and "nothing found within the cap" is not "no sequence exists". `-b` sets a time limit.

## Identifying a state

`cubacle patterns identify` reads the structure of a cube without searching for anything. The cube is given like the input of `solve` (an Orbit64 token, facelets, colours, a move expression or `-s`), on the size of `--size` (3 unless given), 2x2x2 to 7x7x7. It never replaces the search: it explains a state and names the family it belongs to.

```sh
cubacle patterns identify "U2 D2 R2 L2 F2 B2"
```

```text
pattern: Checkerboard (checkerboard), exactly
fingerprint: C0t0 E12f0(2.2.2.2.2.2)
  corners: 8 pieces, 0 displaced, 0 twisted
  edges: 12 pieces, 12 displaced, 0 flipped, cycles of length 2, 2, 2, 2, 2, 2
in corners only: no: an edge or centre piece is moved
in edges only: yes
in G1 domino group, U/D axis: yes
in G1 domino group, F/B axis: yes
in G1 domino group, R/L axis: yes
in half-turn group <U2,D2,R2,L2,F2,B2>: yes
in <R,U>: no: an edge these faces never touch is moved
in <R,U,F>: no: an edge these faces never touch is moved
symmetry: 48 of 48 operations fix it (24 rotations, 24 mirrored)
  1 of them keep every colour, 47 rename colours
  ...
```

The report has four parts.

- **pattern**: the state is put into canonical form, the smallest sticker string under the 48 cube symmetries (24 rotations and their mirror images) and every renaming of the six colours, and looked up among the library patterns and the solved cube of that size. The text after the name says how the state differs from the library's colouring: `exactly`, `turned`, `mirrored`, `with colours renamed`, or a combination. A state that is no library pattern prints `no library match`; a random cube does not match.
- **fingerprint**: for every piece orbit (corners, edges, wings by depth, centres) the number of displaced pieces, of twisted corners or flipped edges, and, where pieces can be told apart (corners, and the edges of the 3x3x3 and the middle wings of odd cubes), the cycle type of their permutation. The compact code is `C` corners with `t` twisted, `E` edges and `W<depth>:` wings with `f` flipped, `X` centre orbits (the number of misplaced stickers), each preceded by the number displaced and followed by the cycle lengths in brackets: `C0t0 E0f12` is the superflip. The code is read in every colour frame and axis order that fits and the least is kept, so a cube, its turned or mirrored image and any renaming of its colours give the same code. On an even cube, which has no fixed centres, the frame is the best matching of colours to faces.
- **subgroups**: exact tests on the pieces. `corners only`, `edges only` and (4x4x4 up) `centres only` say that nothing else moved. On the 3x3x3, `G1 domino group` (corners untwisted, edges unflipped, the middle-layer edges in the middle layer; one line per axis) and the half-turn group are exact too: the half-turn test checks the classes of four corners and four edges and a table of 648 projected arrangements, and a test walks all 663 552 states of the group to confirm it. `<R,U>` and `<R,U,F>` have no description by pieces, so the test only refutes (a piece those faces never touch is moved) or answers `possible`; `cubacle solve --moves "<R,U>"` is the proof. The tests assume a legal cube.
- **symmetry**: how many of the 48 symmetries fix the state when the colours may be renamed, how many of those keep every colour literally, and which operations they are with the renaming each needs (a superflip is fixed by all 48 but only after renaming colours; the identity keeps them).

On a 7x7x7 `identify` takes about 10 ms.

## Cycles and the order of a sequence

`cubacle patterns cycles` prints a state as part cycles with orientation marks and the order of the permutation:

```sh
cubacle patterns cycles "R U R' U'"
```

```text
corners: (UBL UBR) +2 0; (DFR UFR) +2 +2; 4 in place
edges: (UB FR UR); 9 in place
centres: solved
order: 6
```

```sh
cubacle patterns cycles --size 4 "R U 2R' Fw"
```

```text
corners: (DFL UFL UBL UBR DBR) +2 0 0 +2 +2; (DFR UFR); 1 in place
wings 1: ...
centres 1: ...
order: 780
order without the centres: 60
```

The spelling:

- A **slot** is named by the faces that meet there in the order U/D, F/B, R/L: `UFR` for a corner, `UF` for an edge of the 3x3x3, `UF2` for the second wing of that edge on a bigger cube (wings are counted from L to R, from D to U and from B to F), `Ur2c3` for the centre sticker in row 2, column 3 of the U face in the usual net.
- A **cycle** `(UFR UBR DBR)` means that the piece that belongs at UFR sits at UBR, the one that belongs at UBR sits at DBR, and the one that belongs at DBR sits at UFR. Pieces in place and not turned are not listed; `N in place` counts them.
- The **marks** after a cycle give the orientation of each of its pieces in the same order. A corner mark is the number of clockwise thirds, seen from outside the cube, from the slot's U/D face to the face that shows the piece's own U/D colour: `0`, `+1` or `+2`. An edge or wing mark is `0` if the piece shows its first-named colour (U/D before F/B before R/L) on the slot's first-named face and `+1` if it is flipped. Centres have no marks.
- The **order** is how many times the sequence must be repeated for every piece to be back in place and orientation: the least common multiple, over all cycles, of the cycle length times 3 (corners) or 2 (wings) when the marks of the cycle do not sum to a multiple of that number. The second line leaves the centres out. A move sequence (also `-s`) is followed piece by piece and the order is exact; the tests repeat sequences until the cube returns. A cube given as colours defines an order only on the 2x2x2 and 3x3x3, where all pieces differ; on bigger cubes equal pieces are matched in index order, the cycles still rebuild the colours, and the order line says it is not defined. The cycles are read in a colour frame (the centres, or the piece at the D, B, L corner), so colour input may come back with renamed colours.

Reading this spelling back as input is not implemented.

## How the generators were found

`examples/pattern_search` runs a bidirectional search over cost levels from the solved cube and from the pattern (in every orientation on even cubes) and replays its answer with the library before printing it:

```sh
cargo run --release --example pattern_search -- checkerboard 3 range half 8 60
```

Modes: `outer`, `range`, the symmetric macro modes `sym-outer` and `sym-range` for the big cubes, and `face` (single outer-layer turns, replayed on the bigger cubes). `examples/pattern_solve` inverts a solver's solution for 3x3x3 to 5x5x5 (`best-known`). A search stops at `MAX_STATES` stored states (default 5,000,000); with 2,000,000 the 5x5x5 range search for six spots used 0.45 GB of resident memory (`/usr/bin/time -l`), so deep searches stop and leave the pattern at `best-known` rather than grow.

The generators were found from the definitions only; no move list of any collection was read or copied.
