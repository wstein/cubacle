---
title: "Patterns"
description: "Well-known cube patterns for the 2x2x2 to 7x7x7, defined by their structure, with generators found by our own search."
weight: 8
---

# Patterns

`cubacle patterns` lists the pretty patterns the library knows; `cubacle patterns show NAME` prints one with its generators. A pattern is defined by its structure (a colour for every sticker as a function of the cube size), never by a move list. Each generator was found by our own search from that definition and is replayed against it by the test suite (`src/patterns/tests.rs`). Patterns are sets of colour states: same-coloured inner pieces, such as centres, that look identical count as equal, and there is no strict mode.

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
patternness: 100 %
closest: Checkerboard 100 %, Solved cube 56 %, Four spots 56 %
partial: corners matches Solved cube exactly; edges differs
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
  classes: 1 identity, 3 face mirrors, 3 face half turns, 1 inversion, 6 edge mirrors, 6 edge half turns, 6 face quarter turns, 6 quarter-turn mirrors, 8 corner third turns, 8 sixth-turn mirrors
```

`identify -v` adds one line per class with the axes and the colour renaming of every operation.

### Symmetry type and antisymmetry

When some symmetry fixes the state, `identify` appends the **symmetry type** after the symmetry lines: the conjugacy class of the group of fixing symmetries among the subgroups of the cube's 48 symmetries, named by its Schoenflies symbol, with its order. The 48 symmetries have 33 such classes (the list is checked by a unit test); one symbol can cover two or three orientations of the same abstract group in the cube (a half turn about a face axis or about an edge axis), and those are told apart by `#1`, `#2`:

```text
C1:1 C2#1:2 C2#2:2 Ci:2 Cs#1:2 Cs#2:2 C3:3 C2h#1:4 C2h#2:4 C2v#1:4 C2v#2:4 C2v#3:4 C4:4 D2#1:4 D2#2:4 S4:4 C3v:6 D3:6 S6:6 C4h:8 C4v:8 D2d#1:8 D2d#2:8 D2h#1:8 D2h#2:8 D4:8 D3d:12 T:12 D4h:16 O:24 Td:24 Th:24 Oh:48
```

A state is **antisymmetric** under a symmetry when turning (or mirroring) the cube with it gives the inverse cube, with the same color renaming allowed as for symmetry. Symmetries and antisymmetries together form a subgroup of the 96 elements of the symmetries times a flag "inverted"; it has 131 classes. Such a class is written `P(K)`: `P` is the group of all operations as seen on the cube and `K` its symmetric half; `P+1'` means the cube is its own inverse as well (`P` then equals `K`). The line is printed only when an antisymmetry exists, and the symmetry type only when the stabilizer is not trivial; an asymmetric cube prints exactly what it printed before.

```sh
cubacle patterns identify "R2 L2"
cubacle patterns identify "R U R' U'"
```

```text
symmetry: 16 of 48 operations fix it (8 rotations, 8 mirrored)
...
symmetry: D4h (order 16)
antisymmetry: D4h+1' (order 32, 16 symmetries and 16 antisymmetries)
```

```text
symmetry: 1 of 48 operations fix it (1 rotations, 0 mirrored)
...
antisymmetry: C2#2(C1) (order 2, 1 symmetries and 1 antisymmetries)
```

The commutator is no symmetric cube, but a half turn about an edge axis carries it onto its own inverse. The checkerboard and the solved cube are `Oh` and `Oh+1'` (order 96); the four spots are `D4h` and `D4h+1'`; the six spots are `S6` and `D3d(S6)`. The type is told on every size from 2x2x2 to 7x7x7 by the same sticker permutations; antisymmetry needs the inverse cube, which the colors define only on the 3x3x3: its fixed centers give the frame the colors are read in. A 2x2x2 has no fixed piece, so its inverse depends on the frame chosen and antisymmetry has no meaning there, and on bigger cubes equal pieces make the inverse ambiguous. Those sizes get the symmetry type only.

The report has these parts.

- **pattern**: the state is put into canonical form, the smallest sticker string under the 48 cube symmetries (24 rotations and their mirror images) and every renaming of the six colours, and looked up among the library patterns and the solved cube of that size. The text after the name says how the state differs from the library's colouring: `exactly`, `turned`, `mirrored`, `with colours renamed`, or a combination. A state that is no library pattern prints `no library match`; a random cube does not match.
- **patternness** and **closest**: see below; `closest` names the three library patterns (the solved cube included) with the share of stickers that agree after the best of the 48 symmetries and colour renamings, and a `partial:` line says in which orbits the state equals the closest pattern that matches in some (`partial: corners, edges match Solved cube exactly; wings (depth 1) differ`).
- **fingerprint**: for every piece orbit (corners, edges, wings by depth, centres) the number of displaced pieces, of twisted corners or flipped edges, and, where pieces can be told apart (corners, and the edges of the 3x3x3 and the middle wings of odd cubes), the cycle type of their permutation. The compact code is `C` corners with `t` twisted, `E` edges and `W<depth>:` wings with `f` flipped, `X` (x-centres, on the diagonals of a face), `+` (+-centres, on its mid-lines) and `O` (oblique) centre orbits (the number of misplaced stickers; a ring number follows where a shape has more than one ring, `X2:`), each preceded by the number displaced and followed by the cycle lengths in brackets: `C0t0 E0f12` is the superflip. The code is read in every colour frame and axis order that fits and the least is kept, so a cube, its turned or mirrored image and any renaming of its colours give the same code. On an even cube, which has no fixed centres, the frame is the best matching of colours to faces.
- **subgroups**: exact tests on the pieces. `corners only`, `edges only` and (4x4x4 up) `centres only` say that nothing else moved. On the 3x3x3, `G1 domino group` (corners untwisted, edges unflipped, the middle-layer edges in the middle layer; one line per axis) and the half-turn group are exact too: the half-turn test checks the classes of four corners and four edges and a table of 648 projected arrangements, and a test walks all 663 552 states of the group to confirm it. `<R,U>` and `<R,U,F>` have no description by pieces, so the test only refutes (a piece those faces never touch is moved) or answers `possible`; `cubacle solve --moves "<R,U>"` is the proof. The tests assume a legal cube.
- **symmetry**: how many of the 48 symmetries fix the state when the colours may be renamed, how many of those keep every colour literally, and how many operations each class has; `-v` lists the operations with the renaming each needs (a superflip is fixed by all 48 but only after renaming colours; the identity keeps them).

Centre orbits are named by where their stickers sit, never by colour, and the same names are used by `identify` and `cycles`. On the 5x5x5 the two orbits are `x-centres (corner side)` and `+-centres (edge side)`; the 6x6x6 and 7x7x7 add `x-centres (middle side)`, `+-centres (middle side)` and `oblique centres (1 and 2 cells in)`, whose two mirror images are told apart by `a` and `b`. For example (5x5x5):

```text
fingerprint: C0t0 W1:0f12 E0f0 X0 +12
  ...
  x-centres (corner side): 24 centre stickers, 0 misplaced
  +-centres (edge side): 24 centre stickers, 12 misplaced
```

On a 7x7x7 `identify` takes about 10 ms.

### Patternness

`patternness: NN %` says how much a state looks like a pattern, from 0 (irregular) to 100 (the solved cube). It is a measure of regularity, not a probability, and it is computed by `cubacle::patterns::patternness`. `identify`, `cycles`, `show` and `find` print it.

`patternness = 100 * (0.6 S + 0.4 U)`, rounded, with two parts from 0 to 1.

- **S, symmetry**: `(ln order + ln(keep) / 2) / ln 2`, at most 1, where `order` is the number of the 48 operations that fix the state when colours may be renamed and `keep` the number that fix it with every colour kept. Any symmetry beyond the identity gives 1 (colour-keeping ones count extra); a random cube has none.
- **U, uniformity**: the mean over the piece orbits of the share of tests an orbit passes: the displaced count is a simple share of the orbit (none, 1/6, 1/4, 1/3, 1/2, 2/3, 3/4, 5/6 or all), so is the twisted or flipped count, and the cycles of the pieces, where known, all have one length.

Calibration, measured with `random_state` (1000 cubes per size) and the library:

| size | random mean | random p95 | random max |
|------|-------------|------------|------------|
| 2x2x2 | 22.1 | 40 | 100 (the 2x2x2 has few states, some are symmetric) |
| 3x3x3 | 23.4 | 33 | 40 |
| 4x4x4 | 16.5 | 33 | 40 |
| 5x5x5 | 18.4 | 31 | 36 |
| 6x6x6 | 16.3 | 29 | 37 |
| 7x7x7 | 16.0 | 25 | 35 |

The solved cube and every library pattern of every size score 100 (a test checks at least 95). A state with no symmetry cannot score above 40, however regular its orbits are.

The formula was checked against the pattern tables of the CubeTwister project (Walter Randelshofer; the files were read from a temporary directory and are not part of this repository, only the numbers below are kept). The first version (`ln 6` saturation, weights 0.7 and 0.3) gave 46 % of the 3x3x3 patterns at least 95, because many well-known patterns have only two to four symmetries; saturating at one symmetry beyond the identity and weighting 0.6 and 0.4 gave the numbers below. The patterns that stay low have no symmetry at all (asymmetric L shapes and the like): patternness measures regularity, not whether a pattern has a name. The measurement was first made when only a quarter of the lines could be read; the numbers below repeat it on the fully decoded tables, 110 patterns for the 2x2x2 and 1119 to 1262 for the other sizes, split by the share of patterns in each band of patternness (in %). Lines that cannot be read are skipped and counted in the last column; a pattern with no readable line is missing.

| size | patterns | 100 | 98-99 | 95-97 | 90-94 | 80-89 | 60-79 | 40-59 | below 40 | unreadable lines |
|------|----------|-----|-------|-------|-------|-------|-------|-------|----------|------------------|
| 2x2x2 | 110 | 73.6 | 0.0 | 0.0 | 0.0 | 17.3 | 3.6 | 1.8 | 3.6 | 0 of 274 |
| 3x3x3 | 1188 | 74.7 | 0.0 | 0.0 | 11.2 | 2.5 | 0.0 | 4.8 | 6.7 | 30 of 4372 |
| 4x4x4 | 1171 | 78.0 | 0.0 | 0.8 | 8.6 | 6.2 | 1.9 | 2.9 | 1.6 | 11 of 2915 |
| 5x5x5 | 1153 | 82.5 | 0.0 | 3.4 | 7.9 | 2.2 | 0.2 | 2.8 | 1.1 | 2 of 2410 |
| 6x6x6 | 1262 | 70.8 | 0.2 | 4.0 | 7.8 | 10.9 | 3.2 | 2.3 | 0.8 | 9 of 2494 |
| 7x7x7 | 1119 | 80.9 | 2.9 | 5.5 | 4.6 | 2.8 | 0.3 | 2.8 | 0.2 | 11 of 2255 |

Cumulatively, the mean is 92.3 (2x2x2), 91.3, 95.1, 96.4, 95.2 and 97.1 (7x7x7), and the median is 100 for every size. At least 95 reach 74, 75, 79, 86, 75 and 89 % of the patterns, at least 90 reach 74, 86, 87, 94, 83 and 94 %. The built-in patterns of every size score 100.

### Pattern kinds

Patternness stays the one score. The **kind** says which sort of regularity a state has, so a list can be filtered and a reader sees why a state scored as it did. `cubacle::patterns::kind(&Facelets)` returns a `PatternKind`; `identify` prints it with the measures it was judged by:

```sh
cubacle patterns identify "R"
```

```text
kind: MOSTLY (mostly in place, the moved part is symmetric)
measures: fixed 0.78, defect symmetry 0.54, moved 8 of 26 pieces, symmetry 1.00 (order 4), faces 0.67, simple faces 1.00, orbits 1.00
```

```sh
cubacle patterns identify "R U R' U'"
```

```text
kind: IRREG (no regularity of the kinds above)
measures: fixed 0.78, defect symmetry 0.00, moved 7 of 26 pieces, symmetry 0.00 (order 1), faces 0.33, simple faces 0.00, orbits 1.00
```

The measures: **fixed** is the share of stickers that agree with the solved cube after the whole-cube turn that makes most of them agree; the other stickers are the defect, **defect symmetry** is `ln(symmetries that map the defect onto itself) / ln 48`, and **moved** counts the pieces (corners, edge pieces, centre stickers) the defect touches. **faces** is the share of the six faces that show the most common arrangement (up to turning, mirroring and renaming colours), **simple faces** the share of faces that are one colour, stripes, a chequer, a cross, rings or similar. **symmetry** and **orbits** are the two parts of patternness.

The kinds are tried in order and the first that fits wins:

| Kind | Fits when |
|------|-----------|
| `SOLVED` | no piece is moved: the cube is solved up to turning it |
| `FEW` | at most 3 pieces, or at most 15 %, are moved |
| `MOSTLY` | fixed is at least 0.6 and defect symmetry at least 0.5 |
| `SYM` | the symmetry part is at least 0.9 and the number of symmetries reaches the size's minimum |
| `FACE` | the faces repeat one arrangement, or most are simple |
| `IRREG` | none of the above |

The thresholds depend on the size and come from `cargo run --release --example kind_rates`, which counts how many deterministic **short scrambles** (4 to 12 turns of random layers, 30 per length) and 200 long scrambles pass each criterion. Long scrambles are an easy negative (almost nothing passes), short ones are the hard negative, so they decide:

| size | SYM: symmetries at least | FACE: same arrangement at least | FACE: simple faces at least |
|------|------|------|------|
| 2x2x2 | 8 | 1.00 | never |
| 3x3x3 | 4 | 0.67 | 0.67 |
| 4x4x4 to 7x7x7 | 4 | 0.50 | 0.67 |

Share of the 270 short scrambles per size that end up in each kind, with the same set of long scrambles and the built-in library (in %; the 2x2x2 has one library pattern):

| size | | SOLVED | FEW | MOSTLY | SYM | FACE | IRREG |
|------|---|------|-----|--------|-----|------|-------|
| 2x2x2 | short scrambles | 0.4 | 0.0 | 4.4 | 2.2 | 5.9 | 87.0 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 100 | 0 | 0 | 0 | 0 |
| 3x3x3 | short scrambles | 0.0 | 0.0 | 3.3 | 2.6 | 0.0 | 94.1 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 0 | 40 | 60 | 0 | 0 |
| 4x4x4 | short scrambles | 0.0 | 0.0 | 0.7 | 0.7 | 0.7 | 97.7 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 0 | 57.1 | 42.9 | 0 | 0 |
| 5x5x5 | short scrambles | 0.0 | 0.0 | 1.9 | 0.0 | 0.0 | 98.1 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 22.2 | 44.4 | 33.3 | 0 | 0 |
| 6x6x6 | short scrambles | 0.0 | 0.0 | 1.1 | 0.4 | 0.0 | 98.5 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 11.1 | 33.3 | 55.6 | 0 | 0 |
| 7x7x7 | short scrambles | 0.0 | 0.0 | 2.2 | 0.4 | 0.4 | 97.0 |
| | long scrambles | 0 | 0 | 0 | 0 | 0 | 100 |
| | library | 0 | 16.7 | 33.3 | 50.0 | 0 | 0 |

Read the table with care. A short scramble of a few turns of a big cube is genuinely close to a pattern (one turn is `MOSTLY`), so some of these "false positives" are real structure; the point is that `IRREG` is the answer for 87 to 98 % of them. `FACE` decides no library pattern. The 2x2x2 has only 3.7 million states, so its symmetric and face-regular shares are high for every threshold. These kinds describe the structure of one state; they are not claimed to reproduce the categories of any pattern collection.

There used to be a seventh kind, `ORBIT` (every orbit of pieces moved in a regular way, 4x4x4 and bigger). It was removed after measuring it on a fully decoded collection of 1110 to 1260 imported patterns per size: it decided only 0.3 to 1.6 % of them, and in at most 0.4 % of them no other measure reached 90 (patternness at least 90, or any other kind), while it caught 6 to 16 % of the short scrambles of 4 to 12 turns on the big cubes. Its threshold was already the highest possible (a perfect orbit score), so it could not be made stricter; those states are `IRREG` now.

**Kinds on the imported collection.** The same tables (see above) give the share of each kind among the imported patterns (in %), and how many patterns reach 90 in at least one measure, either patternness at least 90 or a kind other than `IRREG`:

| size | SOLVED | FEW | MOSTLY | SYM | FACE | IRREG | patternness at least 90 | kind not IRREG | either |
|------|--------|-----|--------|-----|------|-------|-------------------------|----------------|--------|
| 2x2x2 | 0.9 | 29.1 | 12.7 | 11.8 | 20.0 | 25.5 | 73.6 | 74.5 | 92.7 |
| 3x3x3 | 3.0 | 2.4 | 27.9 | 34.5 | 15.2 | 16.8 | 85.9 | 83.2 | 93.4 |
| 4x4x4 | 2.9 | 18.0 | 24.1 | 29.8 | 22.1 | 3.1 | 87.4 | 96.9 | 99.0 |
| 5x5x5 | 2.9 | 29.7 | 34.1 | 19.2 | 13.1 | 1.0 | 93.8 | 99.0 | 99.9 |
| 6x6x6 | 4.4 | 31.5 | 26.8 | 19.1 | 16.2 | 2.1 | 82.8 | 97.9 | 99.5 |
| 7x7x7 | 0.0 | 33.5 | 27.7 | 27.4 | 9.7 | 1.6 | 94.0 | 98.4 | 99.6 |

So the goal "most patterns reach 90 in at least one measure" holds for every size, with 93 to 99.9 %; the kinds carry the 4x4x4 and bigger cubes (patternness alone reaches 83 to 94 %), and on the 2x2x2 and 3x3x3 the two measures together add 19 and 7.5 points to patternness alone.

**Recognition.** `identify` against the built-in patterns and 70 % of the imported ones (chosen by a hash of the id), for the other 30 %, given as states: an exact match (up to turning, mirroring and renaming colours) is found for 73 % of the 2x2x2 patterns, 46 % of the 3x3x3, 35, 28, 24 and 16 % of the 4x4x4 to 7x7x7 patterns. Exact matches need a twin of the same state in the library, which the bigger tables have less often; for the rest the closest pattern agrees on at least 90 % of the stickers for 65 % (3x3x3) and 74 to 80 % (4x4x4 to 7x7x7) of all held-out patterns, and the median best similarity is 93 to 96 % (2x2x2: 100). As a baseline, 50 scrambles of 8 and of 12 turns have a median best similarity of 43 to 56 % on the 3x3x3 to 7x7x7 (2x2x2: 67 and 63 %), and none reaches 90 % (the 2x2x2 does for 16 % and 2 %, because a few turns of it are often a pattern). A similarity of 90 % or more is therefore a strong hint on the bigger cubes, and 80 % is where a few unrelated scrambles start to appear (up to 6 % on the 7x7x7).

**Kind in lists.** `patterns --kind NAME` (case does not matter) keeps the library and imported patterns of that kind, judged at the size with the highest patternness (the smallest on a tie), and adds a `kind` column to the flat list; it combines with `--size`, `--sort` and `--min-patternness`, and filters the `--tree` too (which has no kind column).

```sh
cubacle patterns --kind MOSTLY --sort patternness --size 3
```

```text
id	name	sizes (outer/range moves)	patternness	kind
six-spots	Six spots	3:8/6	100 %	MOSTLY
four-spots	Four spots	3:8/6	100 %	MOSTLY
```

```sh
cubacle patterns --kind FEW
```

```text
id	name	sizes (outer/range moves)	kind
six-centre-spots	Six centre spots	5:8/6 6:8/4 7:-/8	FEW
four-centre-spots	Four centre spots	5:8/6 6:8/4	FEW
cube-in-cube	Cube in a cube	2:10/10 3:15/15 4:15/15 5:15/15 6:15/15 7:15/15	FEW
```

An unknown name is refused with the list of kinds. `patterns find` is unchanged.

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
x-centres (corner side): (Ur2c2 Fr3c3 Dr3c3 ...); (Ur3c3 Rr2c2 ...); 5 in place
order: 780
order without the centres: 60
```

The spelling:

- A **slot** is named by the faces that meet there in the order U/D, F/B, R/L: `UFR` for a corner, `UF` for an edge of the 3x3x3, `UF2` for the second wing of that edge on a bigger cube (wings are counted from L to R, from D to U and from B to F), `Ur2c3` for the centre sticker in row 2, column 3 of the U face in the usual net.
- A **cycle** `(UFR UBR DBR)` means that the piece that belongs at UFR sits at UBR, the one that belongs at UBR sits at DBR, and the one that belongs at DBR sits at UFR. Pieces in place and not turned are not listed; `N in place` counts them.
- The **marks** after a cycle give the orientation of each of its pieces in the same order. A corner mark is the number of clockwise thirds, seen from outside the cube, from the slot's U/D face to the face that shows the piece's own U/D colour: `0`, `+1` or `+2`. An edge or wing mark is `0` if the piece shows its first-named colour (U/D before F/B before R/L) on the slot's first-named face and `+1` if it is flipped. Centres have no marks.
- The **order** is how many times the sequence must be repeated for every piece to be back in place and orientation: the least common multiple, over all cycles, of the cycle length times 3 (corners) or 2 (wings) when the marks of the cycle do not sum to a multiple of that number. The second line leaves the centres out. A move sequence (also `-s`) is followed piece by piece and the order is exact; the tests repeat sequences until the cube returns. A cube given as colours defines an order only on the 2x2x2 and 3x3x3, where all pieces differ; on bigger cubes equal pieces are matched in index order, the cycles still rebuild the colours, and the order line says it is not defined. The cycles are read in a colour frame (the centres, or the piece at the D, B, L corner), so colour input may come back with renamed colours.

Reading this spelling back as input is not implemented.

## Importing pattern files

`cubacle patterns import` reads a tab-separated pattern table, in the layout of the `PatternExportTab.txt` files of Werner Randelshofer's CubeTwister, and keeps the verified patterns next to the built-in library. cubacle ships none of that data. His pattern database is all rights reserved; it is not part of this repository, is never bundled and is never downloaded by cubacle. If you use it, you use it under the terms of its author, which you have to check yourself. The importer was written from the shape of one file, in our own words below, and its tests use invented lines.

**The layout.** One line is one generator of one pattern; fields are separated by tabs, and the text is UTF-8 or, as in the files exported on a Mac, Mac Roman. Bytes that are valid UTF-8 are read as UTF-8; any other file is read as Mac Roman, where the group separator `·` is the single byte 0xE1. A byte order mark is dropped, `\r\n` line ends are accepted, and `--dry-run` prints which decoding was used (`decoded as Mac Roman`). The fields are:

1. the pattern code: a letter, three digits, a dot and two digits (for example `A200.01`; any code of 1 to 9 letters, digits, dots and hyphens is accepted, anything else is reported and skipped, never shortened);
2. the number of the generator: the same code on several lines means alternative generators of the same pattern;
3. to 8. three levels of grouping (category, subgroup, sub-subgroup), each with a German and an English name; the last level names the pattern;
9. the generator;
10. to 13. four counts, probably block turns, face turns, quarter turns and the order of the sequence, where a `*` seems to mark a proven minimum (unverified; not imported);
14. the cycles of the parts (not imported, cubacle computes its own);
15. to 17. author and year, and notes. The author text is kept as provenance.

The cube size is not in the file. It is read from the name of the folder (`2xPocket`, `3xRubik`, `4xRevenge`, `5xProfessor`, `6xVCube6`, `7xVCube7` are the six folders) or given with `--size N`.

**The moves.** The plain spelling is a space-separated list of face letters with `2` for a half turn and `-` (or `'`) for the inverse. When that does not parse, the field is read by the expression reader of `solve` (see [Command-line guide](cli.md)) after turning a `-` behind a move or bracket into `'`. Brackets with a repeat count, `'` on a bracket, wide turns and layer prefixes therefore read as in our other commands; small letters read as SiGN wide turns (`r` is `Rw`). The group separator `·` (byte 0xE1 in Mac Roman) is the same separator `solve` and `moves` accept; the layer prefixes of the SSE spelling (`T`, `N`, `V`, `M`, `S`, `W`, `C`) are read by the same reader as `moves`, not by a second one. A vertical tab (0x0B) between moves is white space. In the exports a vertical tab also precedes a bracketed note naming the pieces the pattern moves, such as `(uf,dr)` or `(+lb)`; a final bracket made only of lower-case letters, commas, `+` and `-` behind a vertical tab is that note and is dropped, while any other bracket is read as moves. What cubacle cannot read is reported per line: a layer or wide turn that does not exist on the folder's cube, and any construct not listed here. A few lines of the bigger folders still fail and are listed with their line number; other lines are skipped because their variants build different states.

**Checking.** Every sequence is replayed on a solved cube of that size. A line that does not have 9 fields, does not parse, does not apply to the cube or builds another state than the first line of its pattern is reported with its line number on standard error and skipped; the import does not stop. `--dry-run` also recounts every accepted line and prints each claimed count that differs, as a check of the file. The block, face and quarter turns are counted the way the table counts them, not the way `moves` does: a prefixed word (`M`, `N`, `S`, `V`, `W`) is one block turn and worth two face turns, a plain or tier (`T`) turn is worth one, a whole-cube rotation (`C`) none, and the quarter turns are the face turns, doubled for a half turn. Our own counts are not changed by this; only the comparison is. With that rule about 99.9 % of the claimed counts of the six folders agree. The order is compared with ours (`patterns cycles`); it differs on more lines, and no rule for it has been derived. A table with synthetic lines (invented, in this layout):

```sh
cubacle patterns import tables/3xDemo/demo.txt --dry-run
```

```text
3x3x3: 4 lines, 4 verified by replay, 3 patterns, 0 problems
counts: 0 of 16 claimed values differ from ours
```

Without `--dry-run` the verified patterns are written in our own format to `~/.cache/cubacle/patterns/imported/` as `NAME.tsv`: a tab-separated file whose first line is `#cubacle-patterns 1 patternness-v1` (format version and patternness formula version), then `#source LABEL`, then one line per generator with `id`, `catalogue`, `size`, `number`, `name`, `aliases`, `groups`, `provenance`, `description`, `patternness` (an integer 0 to 100, or `-`) and the `moves` as the last field. The patternness is computed at import time; when the version in a file differs from the program's, the stored values are ignored, computed again and the file is rewritten the next time it is loaded (`CUBACLE_CACHE_DIR` and `XDG_CACHE_HOME` apply as for the table caches; `--out FILE` writes elsewhere). `--name-prefix P` goes in front of the ids and names, and `--source XX` sets the two-letter tag of the catalogue ids (default `ct`). A URL is refused with a hint: list it under `[patterns] files` in the config and run `cubacle patterns update`.

**Catalogue ids.** Every pattern has an id of at most 12 characters, `xx:code`: a two-letter source and a code of up to 9 characters from letters, digits, `.` and `-`. The built-ins use `cu:` with a short readable code (`cu:checker`, `cu:sixspot`, `cu:sixcentre`, `cu:foursp`, `cu:fourcen`, `cu:cubecube`, `cu:ccc`, `cu:superflip`, `cu:cenopp`, `cu:cencyc`); imported patterns keep the author's code (`ct:A200.01`). `show` takes a name, an alias, the short id or the catalogue id (case is not significant), and `#n` after it selects the generator with that number: `cubacle patterns show ct:X1.01#2`. The group path is separate from the id.

**Configuration.** `cubacle.toml` lists the pattern sources. It is looked up as `--config FILE`, then `$CUBACLE_CONFIG`, then `./cubacle.toml`, then the global file `~/.config/cubacle/config.toml` (`$XDG_CONFIG_HOME/cubacle/config.toml` when set). On the first run, when none of these exists, cubacle creates the global file from [`cubacle.example.toml`](https://github.com/wstein/cubacle/blob/develop/cubacle.example.toml) and prints one line on stderr saying so; if the location is not writable it says that instead and carries on with the built-in library. The created file has the built-in entry active and the six CubeTwister URLs (folders `2xPocket` to `7xVCube7`) commented out: that data belongs to its author, so check his terms first, then uncomment the entries you want. Enabling an entry is your consent to download it: cubacle fetches only URLs that are listed (uncommented) in your config, never a built-in list, and nothing at all until you run `cubacle patterns update` or set `auto_fetch = true`.

```text
$ cubacle patterns sources        # first run, XDG_CONFIG_HOME=/tmp/cbx/h
cubacle: created /tmp/cbx/h/cubacle/config.toml (built-in library only; third-party sources are commented out, read the notes inside)
config: /tmp/cbx/h/cubacle/config.toml
data directory: ~/.cache/cubacle/patterns
builtin: the built-in library
```

When the location cannot be written, the line reads `cubacle: could not create PATH: REASON; continuing with the built-in library` and `patterns sources` prints `config: none (default: the built-in library only)`.

```toml
[patterns]
files = ["builtin", "tables/3xDemo/demo.txt"]   # paths are relative to this file
# cache_dir = "~/.cache/cubacle/patterns"
auto_fetch = false   # true: a patterns command downloads missing URL sources by itself
```

An entry is a string, or `{ path = "...", size = 3, name_prefix = "x-" }`. A file starting with our header line (the output of `import`) is read as it is; any other file is read as a table and verified again each time. A URL entry is looked up in `<cache_dir>/downloads/`; when it is not there, `cubacle patterns sources` says `not downloaded` for it and every other `patterns` command prints one line on stderr, `1 source is not downloaded; run `cubacle patterns update``, with `-v` followed by the list. A broken entry is a warning and never hides the rest. Errors in the TOML name the file and the line.

```sh
cubacle patterns sources
```

```text
config: cubacle.toml
data directory: ~/.cache/cubacle/patterns
builtin: the built-in library
tables/3xDemo/demo.txt: 3 patterns, 0 of 4 lines skipped (see `patterns import --dry-run`)
```

With imports configured, `patterns` appends `imported from FILE` to the rows of imported patterns, `show` prints the catalogue id, the group path, the author text and every generator by number, and `identify` matches and ranks against the active library, the built-in patterns plus every imported source of the config. A state that equals an imported pattern up to turning, mirroring and recolouring (equal canonical forms) gets a `pattern:` line and an `imported:` line, and imported patterns appear in `closest:` and `partial:` with their catalogue id; `--closest N` lists N patterns instead of three. With a 3x3 table of two invented patterns:

```text
$ cubacle patterns identify -s "U2 D2"
pattern: Slice (ct:D1.02), turned, with colours renamed
closest: Slice (ct:D1.02) 100 %, Four spots 85 %, Solved cube 78 %
imported: ct:D1.02 Slice (imported from demo.txt)
$ cubacle patterns identify -s "R2 L2 U"
pattern: no library match
closest: Slice (ct:D1.02) 70 %, Four spots 67 %, Solved cube 59 %
```

Without imports every output is unchanged.

**Patternness in lists.** `patterns --sort patternness` puts the most pattern-like first and adds a `patternness` column (the highest over the sizes shown); `--min-patternness N` keeps the patterns at or above N. Both work with `--tree`, and without them the output is unchanged. `show` prints `3x3x3 patternness: 33 %` per size, and `import --dry-run` prints one line per pattern and a summary:

```text
ct:D1.01	patternness 87 %
ct:D1.02	patternness 100 %
ct:D1.03	patternness 33 %
patternness: mean 73.3 %, lowest 33 %, 1 of 3 at least 95 %
```

**The tree.** `cubacle patterns --tree` is the overview by category: headings for the category levels, then per pattern its catalogue id, name, number of generators and the shortest generator length per size. The flat list stays the default.

```sh
cubacle patterns --tree --size 3
```

```text
Simple patterns
  cu:checker	Checkerboard	2 generators	3:3
  cu:sixspot	Six spots	2 generators	3:6
  cu:foursp	Four spots	2 generators	3:6
Cube in a cube
  cu:cubecube	Cube in a cube	1 generator	3:15
Flips
  cu:superflip	Superflip	1 generator	3:20
Demo patterns
  Slices
    ct:X1.01	Two slices	2 generators	3:2
    ct:X1.02	Pair	1 generator	3:2
    ct:X1.03	Loop	1 generator	3:6
```

The built-ins have one level: Simple patterns (checkerboard, six spots, four spots and their centre variants), Cube in a cube (cube in a cube, cube in a cube in a cube), Flips (superflip) and Centres (the two centre patterns). The length counts moves as written in the generator (for a built-in the shorter of the outer and range metrics, for an import the number of turns).

## Using the library in `solve`

`cubacle solve --pattern-aware` ([CLI guide](cli.md)) indexes the generators of the active library (the built-in patterns and the imported files) under the 48 symmetries of the cube, looks the cube up exactly, and answers with the pattern's route only when it is strictly shorter than the solver's answer. A large imported collection therefore shortens the answer for every cube that is in it, in any turned or mirrored position, and costs nothing for any other cube.

## How the generators were found

`examples/pattern_search` runs a bidirectional search over cost levels from the solved cube and from the pattern (in every orientation on even cubes) and replays its answer with the library before printing it:

```sh
cargo run --release --example pattern_search -- checkerboard 3 range half 8 60
```

Modes: `outer`, `range`, the symmetric macro modes `sym-outer` and `sym-range` for the big cubes, and `face` (single outer-layer turns, replayed on the bigger cubes). `examples/pattern_solve` inverts a solver's solution for 3x3x3 to 5x5x5 (`best-known`). A search stops at `MAX_STATES` stored states (default 5,000,000); with 2,000,000 the 5x5x5 range search for six spots used 0.45 GB of resident memory (`/usr/bin/time -l`), so deep searches stop and leave the pattern at `best-known` rather than grow.

The generators were found from the definitions only; no move list of any collection was read or copied.

**Downloading.** `cubacle patterns update` (alias `download`) fetches every `http(s)` source of the active config into `<cache_dir>/downloads/` and prints one line per source: `URL: downloaded, 225 bytes, 2 patterns`, `URL: up to date` or `URL: failed: REASON` (the reasons also go to stderr). The exit status is 0 when all sources are fine and 1 when any failed. Next to each file sits `NAME.meta` with the `ETag` and `Last-Modified` the server sent; the next update asks with `If-None-Match` / `If-Modified-Since` and a `304` is `up to date` without a transfer. `--force` downloads anyway. A download is written to `NAME.part`, read with the table reader, and replaces the old file only when it gives at least one pattern; a truncated, garbage or empty answer leaves the old file untouched and removes the `.part`.

```text
$ cubacle patterns update
http://127.0.0.1:8765/3xDemo/PatternExportTab.txt: downloaded, 225 bytes, 2 patterns
$ cubacle patterns update
http://127.0.0.1:8765/3xDemo/PatternExportTab.txt: up to date
$ cubacle patterns --offline update
http://127.0.0.1:8765/3xDemo/PatternExportTab.txt: failed: offline (--offline or CUBACLE_OFFLINE is set); nothing was fetched
```

Limits: https only (plain http is accepted only for `127.0.0.1` and `localhost`, which the tests use; any other host gets `plain http is refused`), at most 50 MB, 15 s to connect and 120 s in all, at most 5 redirects and each only to https. A download is only ever read as a tab table, never run. Proxies are taken from `HTTPS_PROXY`, `https_proxy` and `ALL_PROXY` (and the `HTTP_PROXY` forms). `--offline`, or `CUBACLE_OFFLINE=1` in the environment, makes every `patterns` command skip the network, `update` included (it then fails with `offline`).

`auto_fetch = true` in the config makes a `patterns` command (except `update`, `import`, `sources`, `find` and `cycles`) download the URL sources that are missing from the cache before it starts, with one stderr line `downloading URL ...` each. A failed download is a `warning:` and the command goes on. The default and the created config keep `auto_fetch = false`. `sources` shows for each downloaded URL its date, size and pattern count (`URL: downloaded 2026-10-25, 225 bytes, 2 patterns, ...`), or `not downloaded`.
