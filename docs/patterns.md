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

The formula was checked once against the pattern tables of the CubeTwister project (Walter Randelshofer; the files were read from a temporary directory and are not part of this repository, only the numbers below are kept). The first version (`ln 6` saturation, weights 0.7 and 0.3) gave 46 % of the 3x3x3 patterns at least 95, because many well-known patterns have only two to four symmetries; saturating at one symmetry beyond the identity and weighting 0.6 and 0.4 gave the numbers below. The patterns that stay low have no symmetry at all (asymmetric L shapes and the like): patternness measures regularity, not whether a pattern has a name. Patterns whose moves did not parse are left out.

| size | patterns | mean | min | p5 | median | at least 95 |
|------|----------|------|-----|----|--------|-------------|
| 2x2x2 | 108 | 92.6 | 20 | 40 | 100 | 75 % |
| 3x3x3 | 1122 | 93.0 | 20 | 33 | 100 | 78 % |
| 4x4x4 | 1165 | 94.9 | 13 | 71 | 100 | 79 % |
| 5x5x5 | 1142 | 96.5 | 13 | 84 | 100 | 86 % |
| 6x6x6 | 1257 | 95.1 | 6 | 77 | 100 | 75 % |
| 7x7x7 | 1112 | 97.1 | 20 | 84 | 100 | 89 % |

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

**The layout.** One line is one generator of one pattern; fields are separated by tabs, and the text may be UTF-8 or a legacy 8-bit encoding (read as Latin-1 when it is not UTF-8). The fields are:

1. the pattern code: a letter, three digits, a dot and two digits (for example `A200.01`; any code of 1 to 9 letters, digits, dots and hyphens is accepted, anything else is reported and skipped, never shortened);
2. the number of the generator: the same code on several lines means alternative generators of the same pattern;
3. to 8. three levels of grouping (category, subgroup, sub-subgroup), each with a German and an English name; the last level names the pattern;
9. the generator;
10. to 13. four counts, probably block turns, face turns, quarter turns and the order of the sequence, where a `*` seems to mark a proven minimum (unverified; not imported);
14. the cycles of the parts (not imported, cubacle computes its own);
15. to 17. author and year, and notes. The author text is kept as provenance.

The cube size is not in the file. It is read from the name of the folder (`2xPocket`, `3xRubik`, `4xRevenge`, `5xProfessor`, `6xVCube6`, `7xVCube7` are the six folders) or given with `--size N`.

**The moves.** The plain spelling is a space-separated list of face letters with `2` for a half turn and `-` (or `'`) for the inverse. When that does not parse, the field is read by the expression reader of `solve` (see [Command-line guide](cli.md)) after turning a `-` behind a move or bracket into `'`. Brackets with a repeat count, `'` on a bracket, wide turns and layer prefixes therefore read as in our other commands; small letters read as SiGN wide turns (`r` is `Rw`). What cubacle cannot read is reported per line: a layer or wide turn that does not exist on the folder's cube, and any construct not listed here. The moves of the 2x2x2 sample were all plain face turns; how the bigger folders write wide and slice turns has not been verified.

**Checking.** Every sequence is replayed on a solved cube of that size. A line that does not have 9 fields, does not parse, does not apply to the cube or builds another state than the first line of its pattern is reported with its line number on standard error and skipped; the import does not stop. `--dry-run` also recounts every accepted line (block, face and quarter turns with `src/metrics`, the order as in `patterns cycles`) and prints each claimed count that differs, as a check of our counting and of the file. A table with synthetic lines (invented, in this layout):

```sh
cubacle patterns import tables/3xDemo/demo.txt --dry-run
```

```text
3x3x3: 4 lines, 4 verified by replay, 3 patterns, 0 problems
counts: 0 of 16 claimed values differ from ours
```

Without `--dry-run` the verified patterns are written in our own format (`id | catalogue | size | number | name | aliases | groups | provenance | description | moves`, one line per generator) to `~/.cache/cubacle/patterns/imported/` (`CUBACLE_CACHE_DIR` and `XDG_CACHE_HOME` apply as for the table caches; `--out FILE` writes elsewhere). `--name-prefix P` goes in front of the ids and names, and `--source XX` sets the two-letter tag of the catalogue ids (default `ct`). A URL is refused with a hint: cubacle has no HTTP client and does not download. Fetch the file yourself with `curl -o FILE URL`.

**Catalogue ids.** Every pattern has an id of at most 12 characters, `xx:code`: a two-letter source and a code of up to 9 characters from letters, digits, `.` and `-`. The built-ins use `cu:` with a short readable code (`cu:checker`, `cu:sixspot`, `cu:sixcentre`, `cu:foursp`, `cu:fourcen`, `cu:cubecube`, `cu:ccc`, `cu:superflip`, `cu:cenopp`, `cu:cencyc`); imported patterns keep the author's code (`ct:A200.01`). `show` takes a name, an alias, the short id or the catalogue id (case is not significant), and `#n` after it selects the generator with that number: `cubacle patterns show ct:X1.01#2`. The group path is separate from the id.

**Configuration.** `cubacle.toml` lists the pattern sources. It is looked up as `--config FILE`, then `$CUBACLE_CONFIG`, then `./cubacle.toml`, then `~/.config/cubacle/cubacle.toml` (`$XDG_CONFIG_HOME/cubacle/` when set). Without any file the list is the built-in library alone. [`cubacle.example.toml`](https://github.com/wstein/cubacle/blob/develop/cubacle.example.toml) in the repository root has the built-in entry active and the CubeTwister URLs commented out; uncomment them only after you have checked the terms and downloaded the files.

```toml
[patterns]
files = ["builtin", "tables/3xDemo/demo.txt"]   # paths are relative to this file
# cache_dir = "~/.cache/cubacle/patterns"
auto_fetch = false   # read but without effect: downloading is not built in
```

An entry is a string, or `{ path = "...", size = 3, name_prefix = "x-" }`. A file starting with our header line (the output of `import`) is read as it is; any other file is read as a table and verified again each time. A URL entry is looked up in `<cache_dir>/downloads/`; when it is not there, `cubacle patterns sources` and every other `patterns` command print a warning with the `curl -o` line that puts it there. A broken entry is a warning and never hides the rest. Errors in the TOML name the file and the line.

```sh
cubacle patterns sources
```

```text
config: cubacle.toml
data directory: ~/.cache/cubacle/patterns
builtin: the built-in library
tables/3xDemo/demo.txt: 3 patterns, 0 of 4 lines skipped (see `patterns import --dry-run`)
```

With imports configured, `patterns` appends `imported from FILE` to the rows of imported patterns, `show` prints the catalogue id, the group path, the author text and every generator by number, and `identify` adds an `imported:` line for every imported pattern the state equals up to turning, mirroring and recolouring (equal canonical forms). Without imports every output is unchanged.

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

## How the generators were found

`examples/pattern_search` runs a bidirectional search over cost levels from the solved cube and from the pattern (in every orientation on even cubes) and replays its answer with the library before printing it:

```sh
cargo run --release --example pattern_search -- checkerboard 3 range half 8 60
```

Modes: `outer`, `range`, the symmetric macro modes `sym-outer` and `sym-range` for the big cubes, and `face` (single outer-layer turns, replayed on the bigger cubes). `examples/pattern_solve` inverts a solver's solution for 3x3x3 to 5x5x5 (`best-known`). A search stops at `MAX_STATES` stored states (default 5,000,000); with 2,000,000 the 5x5x5 range search for six spots used 0.45 GB of resident memory (`/usr/bin/time -l`), so deep searches stop and leave the pattern at `best-known` rather than grow.

The generators were found from the definitions only; no move list of any collection was read or copied.
