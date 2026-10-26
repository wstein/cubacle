---
title: "Solve a cube"
description: "Solve a cube, with targets and shorter or shortest solutions."
weight: 1
---

# Solve a cube

## Solve a cube

`solve` reads a cube, prints a solution on standard output and exits with status 0.

```sh
cubacle solve A4ib8aejxj4Y
```

```text
B R' D' R B U R U' B2 U' D B' R' B U
```

You can also solve the cube produced by a scramble:

```sh
cubacle solve --scramble "R U R' U'" --max-moves 20 --budget-ms 500 --verbose
```

```text
elapsed 69.316417ms, probes 1
moves 15
U R U R F2 R F2 R' L D2 R' D2 L' U2 R'
```

The first two lines go to standard error. `moves N` is always printed there, `elapsed` only with `--verbose`. The solution goes to standard output. The elapsed time varies between runs.

| Option | Meaning |
| --- | --- |
| `-s`, `--scramble <MOVES>` | Solve the cube produced by this scramble instead of reading one. |
| `--size <N>` | Cube size for `--scramble`: 2 to 7, default 3. A cube given as text has its size already. |
| `--scheme <SCHEME>` | Color scheme for color input. See [Color schemes](input-notations.md#color-schemes). |
| `--to <TARGET>` | Solve to a state other than the solved cube, or to any state that matches a target with open stickers (`X`). See [Solve to a target](#solve-to-a-target) and [Targets with open stickers](#targets-with-open-stickers). |
| `-m`, `--max-moves <N>` | Longest solution to accept. Default: 21 for the 2x2x2 and 3x3x3, none for the 4x4x4 and 5x5x5. Also caps the default 3000 probes. |
| `-b`, `--budget-ms <MS>` | Hard time limit in milliseconds. Given on a 3x3x3 it also restores the first-answer search (no default 3000 probes). Reaching it is the same error as `--max-time` (exit status 3). |
| `--max-time <MS>` | Time limit of the whole solve in milliseconds, 0 for none. Default 120000 for the 4x4x4 to 7x7x7, none for the 2x2x2 and 3x3x3; with `--moves` none. The 4x4x4 and 5x5x5 look at the clock inside every search stage, the 6x6x6 and 7x7x7 inside the center searches and between the wing stages. Reaching it is an error, see [Failure](solve-options.md#failure). Not with `-b`, `--improve` or `--parallel`. With `--now` it limits the search for the mistake (default 8000). |
| `--fast` | 3x3x3 only. Print the first solution found, about 20.6 moves on average, in a few milliseconds. Without it a plain `solve` of a 3x3x3 makes 3000 probes (see [Shorter solutions](#shorter-solutions)). Not with `--probes`, `--improve` or `--parallel`. |
| `--probes <N>` | 3x3x3 to 7x7x7 (not the 2x2x2). Keep improving the solution until N probes were made. Counts work, not time, so the answer is the same on every machine. On the 4x4x4 to 7x7x7 it sets the probe minimum of the 3x3x3 stage; the 6x6x6 and 7x7x7 also keep N reductions of the wing stages (see [Shorter solutions](#shorter-solutions)). |
| `--improve` | 3x3x3 to 7x7x7 (not the 2x2x2). Keep improving until the time budget (`-b`, default 1000 ms) is spent, then print the best solution found. |
| `--parallel` | 3x3x3 only, with `--improve` and `-b`: search the six orientations at once. Measured no better than plain `--improve`. |
| `--now <CUBE>` | The cube as it is now, after following the solution of the cube given above and going wrong: `solve` then explains the mistake and the way back instead of solving. See [Recover a mistake](recover.md#recover-a-mistake) for this mode and its options (`--solution`, `--at-move`, `--sure-until`, `--fresh`, `--strays`, `--one-mistake`, `--thorough`, `--max-time`, and `--depth` with another meaning). It cannot be combined with the options that change how a plain solve searches. |
| `--within[=K]` | An optimal route of at most K moves (1 to 7, default 6) for a nearly solved cube of any size, searched from both ends; works for every `--to` state. Exit code 2 when there is none, 3 at a limit. See [A nearly solved cube](solve-options.md#a-nearly-solved-cube-within). |
| `--blocks` | With `--within`: also the blocks of two and three layers from either side and the layers 2-3. |
| `--then-solve` | With `--within`: when no route of at most K moves exists, solve with the normal solver. |
| `--slices` | 4x4x4 and 5x5x5: allow single inner layers (`2R`) in the answer where they make it shorter. Default: outer layers and two-layer turns only. See [the move metric](../performance.md#move-metric). |
| `--notation <wca\|sign\|sse>` | How the solution is written. Default `wca` (`Rw`, `3Rw`); `sign` writes `r`, `3r`; `sse` writes `TR`, `T3R`, `MR` (see [Move notations](move-notations.md#move-notations)). With `--icons` it also sets the move lines and the last line; the icon labels stay WCA. |
| `--optimize ranges` | After the solve, with no search: adjacent turns of one axis are written as layer ranges where that is shorter (`2-3r` in sign, `2-3Rw` in wca, `V2R`, `M2R` or `N2-4R` in sse; only contiguous layers). Off by default. The solution is the same state; stderr reports `moves N` (as solved), `compressed moves M` and, after the metrics line of the plain answer, `compressed metrics: ...`; stdout holds the compressed text. With `--icons` the pictures show the compressed solution: each range or block move is one picture with one arrow over the layers it turns (see [Ranges in the pictures](../icons.md#ranges-in-the-pictures)). On 20 random 4x4x4 to 7x7x7 solutions it saved 0 to 2 moves (2 of 20 saved any, both 7x7x7), on 14 human 4x4x4+ states 0 to 5, because the solvers already merge most axis runs. |
| `--group <none\|phases\|axis>` | `phases` writes the solution with ` · ` between the solver phases that the engine reports (see [Hand units](moves.md#hand-units)); `axis` writes ` · ` between hand units by axis (turns about one axis on disjoint layers); both are given as `--group phases,axis`; `none` (default) is the plain list. Not with `--icons` (refused with the reason: the pictures are headed by the solver phases already). With `--optimize ranges` each phase is compressed on its own, so no range crosses a dot. |
| `--icons` | Show each move as an icon. The output begins with `Start state: TOKEN`, then the start state as a picture (on rows of its own, one and a half times as large as a move icon, with a frame), has a line `State after phase N as token: TOKEN` after each phase but the last, and ends with `End state: TOKEN`, the end picture and `Moves: …`, so that a checkpoint can be compared with your cube. See [Move icons](../icons.md#checkpoints). |
| `--icon-size <PIXELS>` | With `--icons`: icon height in pixels. Default: 32 per cube layer, at least 96. |
| `--background <dark\|light>` | With `--icons`: what the icons are seen on. It sets the notation color and the dimming. Default: the terminal is asked (`OSC 11`), dark when it does not answer. |
| `--graphics <auto\|kitty\|ansi\|plain>` | With `--icons`: how the pictures are drawn. Default `auto` (variables, then a question to the terminal). An explicit value is used as given, also when output is not a terminal. The variable `CUBACLE_GRAPHICS` has the same values; the flag wins. Also accepted by `moves` and by `solve --now`. See [Forcing the graphics](../icons.md#forcing-the-graphics-and-terminals). |
| `--columns <N>` | With `--icons` on a graphics terminal: icons per row. Default: as many as fit the terminal width. See [Move icons](../icons.md#how-the-terminal-is-asked). |
| `--views <folded\|both\|urf\|net\|all>` | With `--icons`: how the whole-cube state pictures are drawn. `folded` (default) draws the Up-Right-Front corner in perspective with L, D and B unfolded flat around it, every sticker once; `both` draws each state from the Up-Right-Front and from the opposite Down-Left-Back corner, side by side, so all six faces show; `urf` only the first; `net` the flat cross net (U above F, L F R B in a row, D below), every sticker once; `all` the net and both corners. See [The folded view](../icons.md#the-folded-view), [Two views of a state](../icons.md#two-views-of-a-state) and [The flat net](../icons.md#the-flat-net). |
| `--numbers <auto\|on\|off>` | With `--icons`: the move number in the top-left corner of every icon (`on`) or none (`off`); `auto` (default) is `on`, except that SVG files written by `moves` have no numbers. The number is the position in the move list, the N of `solve --now --at-move N`. See [Move numbers](../icons.md#move-numbers). |
| `-v`, `--verbose` | Print timing and search statistics to standard error. With `--probes` or `--improve` also `improved from L1 to L2 moves`. Also one line per phase of the solution, such as `phase 1: to the G1 subgroup, moves 1-14` or `phase 2: wings staged, moves 20-25`; the 4x4x4 to 7x7x7 name the stages of their engine (see [Move icons](../icons.md#phases)). |

A cube argument of `-` reads the cube from standard input:

```sh
cubacle random | cubacle solve -
```

### What solved means

`solve` works on colors: a cube is solved when every face shows one color. Inner pieces of one color that cannot be told apart by looking, such as the centers of the 4x4x4 to 7x7x7, are interchangeable, so two cubes that differ only by a swap of same-colored centers are the same cube and get the same answer. There is no strict mode that tells them apart. A `--to` target on the 4x4x4 and larger may still ask for a scrambled arrangement of same-colored centers, that is for centers of one kind on one face in different colors (`Rw` moves half of the centers of the front face away): the route then ends in a center fix-up, see [Targets with mixed centers](#targets-with-mixed-centers).

```sh
cubacle solve --size 4 "R U" --to Rw
```

```text
moves 3
U' R' Rw
```

### Solve to a target

`--to <TARGET>` makes `solve` print the moves that take the start cube to the target instead of to the solved cube. The target is one argument (quote it) and is read like the start: an Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colors (with `--scheme`), or, when the text is no cube, a move expression applied to the solved cube of the start's size. A text that is exactly `superflip`, `checkerboard` or `cube-in-cube` is the pattern of that name instead; these are 3x3x3 patterns, and other sizes get `--to: superflip is a 3x3x3 pattern, not a 4x4x4 one`. Longer texts such as `superflip R` are not names.

```sh
cubacle solve "R U F2 D' L" --to superflip
cubacle solve --scramble "R U R' U'" --to "F2 B2"
```

```text
moves 20
L' U' F B' R U D' F L2 D2 F' U2 F' R2 L2 F' R2 F2 U2 L
moves 15
U R U R F2 R L' U2 R' B2 R' D2 R' L F2
```

Start followed by the printed moves shows the target; the tool replays every answer and exits with an error if it does not. No engine changes: the tool builds the state that is left of the start once the target is taken out and solves that. A target of `UUUU...` (the solved cube) gives the same moves as no `--to`. On 20 random 3x3x3 pairs the answers had 20.6 moves on average against 20.75 for 20 plain solves of other random cubes; on the 4x4x4 (6 pairs) 45.2 against 44.5 and on the 5x5x5 (4 pairs) 68.8 against 68.3, where the targets were 40 random outer turns and the plain 4x4x4 answer includes its final whole-cube rotation.

Rules and limits:

- Start and target must have the same size: `the start is a 3x3x3 but the target is a 2x2x2`.
- An impossible start or target is rejected: `the start or the target state is impossible`.
- 4x4x4 and larger: the target holds exactly, with orientation, and the colors of its centers count. Equal pieces are paired fixed points first, so the centers may sit on other faces: `solve --size 4 "R U" --to "Rw2 Lw2"` prints `U' R' Rw2 Lw2`. A target whose centers of one kind show different colors on one face is reached by a center fix-up (next section). A target with center colors no cube has (a color on more or fewer than four places of an orbit) is `impossible`.
- 2x2x2: it has no centers to fix its frame, and its solver ends in any of the 24 orientations, so the target counts as reached when the cube shows it after a rotation of the whole cube.
- With `--icons` the output has `Target state: TOKEN` below `Start state: TOKEN`, and the move pictures end in the target.
- A target is one argument, so `--to` does not read standard input.

### Targets with mixed centers

On the 4x4x4 to 7x7x7 the solvers work on colors: they bring every center to its face but leave the centers of a face in any order. A target built with inner slices (`2R`, SSE `VB`), wide turns with slices or an imported pattern can show different colors on one face, and the solver cannot aim for that directly. `solve --to` then plans the route in three steps: find moves `w` that make the target's centers uniform, solve the start to that nearby target as for any uniform target, and append `w` undone, the center fix-up. `w` is the shortest sequence of layer moves when a search finds it within a fixed work limit (so the answer is the same on every machine), else a short one from a beam search, else the shortest prefix of the engine's own solution of the target that leaves the centers uniform. When the target is given as moves, undoing those moves bounds the fix-up, so the route is never longer than solving to the solved cube and then making the target's moves. The fix-up is its own phase, `center fix-up`, the last one in `-v`, `--group phases` and `--icons`.

The owner's SSE slice target on a 7x7x7, from a 40-move scramble (`B L2 3R' 3U' ... B' 3D2`; 220 moves, 188 for the start and 32 for the fix-up, 35 s on a busy machine):

```sh
cubacle solve --size 7 -s "B L2 3R' 3U' 3L 3R2 2F2 U 3R' 3D' 2R U 3F 2R2 3B2 2L 3U2 3R' 2B' 2R2 3D2 3F' 2D' 3F' 2D' 2R' 3D' U' 3R' 2B 2U2 2B' F2 3L 3R F2 L 3U' B' 3D2" \
  --to "VB VD VL VD2 VL' VD VB2 VD' · VL' VR' · VU' VB VU2 VB' VU2 VF'" --notation sse -v
```

```text
center fix-up: 32 moves (the moves that built the target, undone)
moves 220
phase 6: 3x3 stage, moves 168-188
phase 7: center fix-up, moves 189-220
```

Measured routes (random 40-move starts, targets of 3, 5 and 8 random inner-slice turns; moves and seconds on a busy machine). Naive is the plain solve of the start plus the target's own moves:

| Cube | Naive | Route from moves | Route from facelets (search) |
| --- | --- | --- | --- |
| 4x4x4, 3 / 5 / 8 slices | 47 / 49 / 52 | 45 / 47 / 51 | 46 / 47 / 51 |
| 5x5x5, 3 / 5 / 8 slices | 72 / 74 / 77 | 72 / 74 / 76 | 72 / 74 / 76 |
| 6x6x6, 3 / 5 / 8 slices | 135 / 137 / 140 | 135 / 137 / 140 | 135 / 137 / 140 |
| 7x7x7, 3 / 5 / 8 slices | 191 / 193 / 196 | 191 / 193 / 195 | 191 / 193 / 195 |
| 7x7x7, the owner's target | 220 | 220 | 225 |

So the route is as short as the naive one and sometimes one or two moves shorter; a target of many slice turns given only as facelets costs a few moves more, because no short sequence is found for it. Rules:

- The odd cubes keep their face centers: a target built with the middle slice (`3F` on a 5x5x5) is first put into the start's frame by middle-layer turns, which count in the fix-up.
- `-m` counts the whole route: `the center fix-up alone takes 5 moves, over the limit of 5`; `-b` covers the planning too (`the budget went into planning the center fix-up`).
- `--probes` and `--improve` act on the solve to the nearby target; `--parallel` stays 3x3x3 only. `--moves` (a move set) is refused with such a target. `--optimize ranges`, `--notation`, `--group` and `--icons` work as for any target.
- Every route is replayed against the target before it is printed.

#### A target without a cube

With `--to` and no cube (and no `--scramble`) the start is the solved cube, so the answer is the route from the solved cube to the target: how to build the pattern. A target that is a cube state gives its own size (2x2x2 to 7x7x7); a move expression or a pattern name uses `--size` (default 3). An explicit `--size` that disagrees with a state target is an error. Standard error says `start: the solved cube (no cube given; routing to the target)`; standard output stays the plain solution, and every other option (`--icons`, `--notation`, `-m`, `-b`, `--group`, ...) works as with a given start. `--now` needs a real cube and cannot be combined with `--to`.

```sh
cubacle solve --to "R U F2 D'"
cubacle solve --to ADHIm3Ila4c0-D5-oM1NpD4OGcpMmAORicPxlSSZMsw
cubacle solve --to ADHIm3Ila4c0-D5-oM1NpD4OGcpMmAORicPxlSSZMsw --size 3
cubacle solve
```

```text
moves 13
R U R2 U F2 U2 D2 B2 U' L2 B2 U2 D
moves 46
Dw Uw B F Lw Rw Dw Uw' B' F Lw Rw U2 Fw2 2D2 F2 Uw2 Lw2 Dw2 L2 Fw2 R2 U2 L2 Bw2 Uw2 Fw2 D2 R2 B2 L2 U L2 R2 U2 B2 F2 U' B F D B2 F2 L2 R2 U'
error: --to: the target is a 5x5x5 but --size is 3
error: no cube given; give a cube state or --scramble (or use --to TARGET to start from the solved cube)
```

### Targets with open stickers

A `--to` target may leave stickers open: the printed moves then reach any state that shows the right color on every sticker that is fixed. An open sticker is written `X` (a `?` is read as `X`). Lowercase `x` stays the cube rotation and is never a sticker. A text is a target with open stickers when it has the length of a cube string (24, 54, 96, ...) and an `X` or `?` among face letters (`URFDLB`) or color letters (`WRGYOB`, read with `--scheme`); every other text is read as above, and a target without an open sticker keeps the old code path and the old output, byte for byte. Orbit64 tokens cannot express open stickers.

Two names stand for a 3x3x3 subset over the solved cube: `cross` (the four edges of the down face, both stickers each, and the centers) and `cross-u`, `cross-r`, `cross-f`, `cross-d`, `cross-l`, `cross-b` for the cross of another face. A name is only taken as the whole text.

```sh
cubacle solve --size 2 "R U F2 R' U' F R2 U F'" --to UUUURRRRXXXXXXXXXXXXXXXX
cubacle solve "R U2 F' D L2 B R' U F2 D' L B2 U' R2 F D2 L' U B' R" --to cross
cubacle solve "R U2 F' D L2 B R' U F2 D' L B2 U' R2 F D2 L' U B' R" --to cross-u
```

```text
moves 7
metrics: htm 7, qtm 9, stm 7
shortest: proved by a search from the whole goal set of 24 states
U' R2 F2 R U F U
moves 4
metrics: htm 4, qtm 6, stm 4
shortest: proved by an exact search over the 4 edges the target names
L2 D2 L' F'
moves 5
metrics: htm 5, qtm 5, stm 5
shortest: proved by an exact search over the 4 edges the target names
R U R' B U
```

The line after the metrics says what the answer is. `shortest` is printed only where the search proves it; otherwise the line says `best found`, and a target the engine cannot handle is refused, never answered with a longer or unproven solution under the word shortest.

| Size | Targets with open stickers | Answer |
|---|---|---|
| 2x2x2 | any mask | shortest, by a search from the whole goal set (24 rotations of the cube count, as for complete targets) |
| 3x3x3 | up to five whole edges and the centers (`cross`, `cross-u`, ...) | shortest, by an exact search over those edges |
| 3x3x3 | any mask that allows at most 24 complete cubes (for example one corner open) | best found: the shortest of the two-phase answers to each cube; `--probes N` makes them shorter, 16 moves instead of 20 in the example below |
| 3x3x3 | more (a face, F2L) | refused with `unsupported: the target allows more than 24 complete cubes` |
| 4x4x4 and larger | any | refused with `unsupported: the 4x4x4 engine cannot solve targets with open stickers yet` |

```sh
cubacle solve "R U2 F' D L2 B R' U F2 D' L B2 U' R2 F D2 L' U B' R" --probes 3000 --to UUUUUUUUXXRRRRRRRRFFXFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

```text
moves 16
metrics: htm 16, qtm 22, stm 16
best found: the shortest of the two-phase answers to 1 complete cube(s) the target allows, not proven shortest
U2 B U' D R' L2 U2 R F2 D2 B' R2 B D' L' F'
```

Errors name the case: `unreachable: no state of the 2x2x2 meets the target (all 3674160 states tested)` for three stickers of one corner all `U`; `the target needs color U on 12 stickers but a cube has only 4 of it; the target is contradictory` for too many of a color; `contradictory: no edge of the cube has the colors U and D`; `unreachable: the target wants another color on a center, and the centers of a 3x3x3 never move`; `--to: cross is a 3x3x3 subset, not a 2x2x2 one`. With a target that has open stickers `--moves`, `--all`, `--count`, `--icons`, `--improve` and `--parallel` are refused for now. Measured on a quiet machine in release (fixed scrambles, one process at a time): any 2x2x2 mask 0.25 to 0.3 s and about 30 MB (a plain 2x2x2 solve is 0.08 s and 21 MB), the 3x3x3 cross 0.03 s and 4.5 MB, a cross and one more edge 0.7 s and 25 MB, one or two corners open 0.04 s.

### Shorter solutions

A plain `solve` of a 3x3x3 makes at least 3000 probes, a counted improvement: the same answer on every machine, never longer than the first solution, 19.0 moves on average for random cubes (20.7 for the first solution), 90 ms on average and under 0.2 s at most on 30 fixed random cubes (about 0.6 s at worst on a loaded machine). `--fast` returns the first solution instead, in about 1 ms of search. The default is replaced by an explicit `--probes`, `--improve`, `--moves`, `--to` or `-b`, which keep their own meaning; the 2x2x2 is unchanged, and so are the 4x4x4 and larger (their 3x3x3 stage asks for 500 probes). `solve --now` and `patterns` do not use it. Two modes look for a still shorter one:

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'" --probes 1000 -v
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'" --improve -b 100 -v
```

`--probes N` is repeatable: the same cube and N give the same solution everywhere. `--improve` is timed: it runs for the whole budget and the result depends on the machine. Reaching the budget after a solution was found is not an error. Measured on 500 random cubes (mean moves): 20.83 for the first solution (`--fast`); 19.16 for `--probes 1000` (42 ms per cube); 18.59 for `--probes 10000` (466 ms); 18.99 for `--improve -b 100`; 18.48 for `--improve -b 1000`. The full table and the commands are in [3x3x3 performance](../engines/3x3.md#shorter-solutions-moves-against-probes-and-time).

Without `--improve`, `--parallel` is refused, and so is `--parallel` without `-b`: it would never return. The 2x2x2 is already optimal and refuses these options with a one-line error; `--parallel` works on the 3x3x3 only.

On the 6x6x6 and 7x7x7 the first solution is probe 1 and is what a plain `solve` prints, byte for byte. `--probes N` then repeats the wing stages and the 3x3x3 stage on the same reduced centers: the wing stages keep N reductions each (at most 48) and the 3x3x3 stage makes 1000 probes per N, and the shorter total wins, the first on a tie, so the answer is never longer. The center stages, which make most of the moves, are not varied, so the gain is small. `--improve -b MS` takes the first solution without a time limit and then spends the budget on the same repeat with N from the budget as on the 4x4x4 and 5x5x5 (at most 48); the result depends on the machine. `--max-time` bounds the first solution and the counted repeat (a repeat that runs out of time keeps the first solution). The moves are merged and canceled per axis by the engine's usual cleanup, as in every solution. Measured on fixed seeded scrambles (`tools/bench/probes_big.rs`, tables cached, mean over 20 cubes of the 6x6x6 and 15 of the 7x7x7, OBTM/BTM per cube): 6x6x6 134.3/132.1 first, 132.2/129.9 with `--probes 5` (2.6 s against 2.3 s), 132.1/129.5 with 20, 132.1/129.2 with 48 (3.4 s); 7x7x7 189.9/186.7 first, 187.9/184.2 with 5 (3.6 s against 3.3 s), 187.6/184.0 with 20, 187.5/183.7 with 48 (4.6 s). That is about 1.5 percent fewer moves; a tenfold probe count buys almost nothing more. For the target `FxnPH-jRPlq7IufcH-uQYFlfnhxdBAeZGz6ONPglYfRsWaqD8wxhAFR7zJ-nheZxFv2Gw_5ahskca3-5Xz288Kj7Lw` on the 7x7x7 (`--size 7 --to`): 194/193 first, 192/191 with `--probes 5`, 20 and 48. Only the 21 moves of the solve after the center fix-up improve (to 19); the 172 moves of the fix-up are the prefix of the plain solution and are not varied. On a single 6x6x6 scramble `--improve -b 1000` gave 133/128 against 135/131 and `--improve -b 5000` 132/127. With a target that needs a center fix-up, `--improve -b` counts the fix-up in its budget as it does for the 4x4x4 and 5x5x5, so a small budget can end in `error: the time budget ran out while fixing the target's centers`.

### Shortest solutions

`--optimal` finds a provably shortest solution (half-turn metric) of a 3x3x3. The default engine answers first, with the same 3000 probes as a plain `solve`; it is the fallback and the upper bound. Then an iterative deepening search, guided by pattern databases that are built once and kept in the cache directory (`CUBACLE_CACHE_DIR`, else `~/.cache/cubacle`), looks for a shorter one, starting at the lower bound and going up one move at a time. A solution found this way is the shortest, because no shorter length was left. `-b` (default 60000 ms) and `--nodes N` (a count, the same on every machine) end the search. If the search ends first, the default answer is printed and stderr says how far the proof got:

```sh
cubacle solve --optimal -s "R U F' D2 L B' R2 U F2 D L2"
cubacle solve --optimal -b 3000 -s "R U F' D2 L B' R2 U F2 D L2 B' U2 R F D' L2 U' B2 R' F2 U D2"
```

```text
optimal search with the large pruning tables (555 MB)
optimal: no solution is shorter than 11 moves
moves 11
metrics: htm 11, qtm 15, stm 11
L2 D' F2 U' R2 B L' D2 F U' R'
```

```text
optimal search with the large pruning tables (555 MB)
not proven shortest: the default engine's 18 moves; no solution is shorter than 14 (stopped after 3000 ms)
moves 18
metrics: htm 18, qtm 24, stm 18
L' U2 F' L F U' D F B2 L B' D' L2 B2 D R2 B2 U
```

The second run proves only that no solution has fewer than 14 moves, and prints the default engine's 18. The answer is never longer than the default one, and every answer is replayed on the cube before it is printed. The first run in a cache directory builds the tables, which takes a few minutes (Small: about 50 s of CPU; Large: about 6 minutes of CPU) and prints its progress on stderr.

The tables depend on the memory this machine really has, read at run time (macOS, Linux and Windows alike, never assumed from the platform): without `--max-memory` the budget is half of the available memory (the total where the system does not report it), at most 16000 MB. The large set (555 MB) is used when it fits the budget, else the small one (86 MB), else an error. An explicit `--max-memory MB` wins. If the memory cannot be read, the small set is used and stderr says so. The first stderr line names the choice and the reason, for example `optimal search with the large pruning tables (555 MB): this machine has 17179 MB of memory, available: 9533 MB; budget 4766 MB (half of the available memory, at most 16000)`. `--optimal` needs the 3x3x3 and cannot be combined with `--fast`, `--probes`, `--improve`, `--parallel`, `--moves`, `--to`, `--all` or `--count`. **Random cubes are mostly out of reach on one core of a laptop:** a cube that needs 17 or 18 moves takes billions of nodes, minutes to hours. Measured numbers, by length, are on [3x3x3 performance](../engines/3x3.md#optimal-solutions).
