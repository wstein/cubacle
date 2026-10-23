---
title: "Command-line guide"
description: "Use the cubacle command-line tool."
weight: 3
---

# Command-line guide

The `cubacle` tool has six subcommands: `solve`, `convert`, `scramble`, `random`, `facelets` and `icon`. Run `cubacle <command> --help` for the full option list.

## Solve a cube

`solve` reads a cube, prints a solution on standard output and exits with status 0.

```sh
cubacle solve A4ib8aejxj4Y
```

```text
B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R
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
| `--scheme <SCHEME>` | Colour scheme for colour input. See [Colour schemes](#colour-schemes). |
| `-m`, `--max-moves <N>` | Longest solution to accept. Default: 21 for the 2x2x2 and 3x3x3, none for the 4x4x4 and 5x5x5. |
| `-b`, `--budget-ms <MS>` | Hard time limit in milliseconds. |
| `--probes <N>` | 3x3x3 only. Keep improving the solution until N probes were made. Counts work, not time, so the answer is the same on every machine. |
| `--improve` | 3x3x3 only. Keep improving until the time budget (`-b`, default 1000 ms) is spent, then print the best solution found. |
| `--parallel` | 3x3x3 only, with `--improve` and `-b`: search the six orientations at once. Measured no better than plain `--improve`. |
| `--slices` | 4x4x4 and 5x5x5: allow single inner layers (`2R`) in the answer where they make it shorter. Default: outer layers and two-layer turns only. See [the move metric](performance.md#move-metric). |
| `--icons` | Show each step as an icon. The output begins with `Start state: TOKEN` and has a line `State after phase N as token: TOKEN` after each phase, so that a checkpoint can be compared with your cube. See [Move icons](icons.md#checkpoints). |
| `--icon-size <PIXELS>` | With `--icons`: icon height in pixels. Default: 32 per cube layer, at least 96. |
| `--background <dark\|light>` | With `--icons`: what the icons are seen on. It sets the notation colour and the dimming. Default: the terminal is asked (`OSC 11`), dark when it does not answer. |
| `--columns <N>` | With `--icons` on a graphics terminal: icons per row. Default: as many as fit the terminal width. See [Move icons](icons.md#how-the-terminal-is-asked). |
| `-v`, `--verbose` | Print timing and search statistics to standard error. With `--probes` or `--improve` also `improved from L1 to L2 moves`. Also one line per phase of the solution, such as `phase 1: to the G1 subgroup, moves 1-14` or `phase 2: wings staged, moves 20-25`; the 4x4x4 to 7x7x7 name the stages of their engine (see [Move icons](icons.md#phases)). |

A cube argument of `-` reads the cube from standard input:

```sh
cubacle random | cubacle solve -
```

### Shorter solutions

By default `solve` stops at the first solution, about 20.8 moves on average for a 3x3x3. Two modes look for a shorter one:

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'" --probes 1000 -v
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B' R2 U F D' L' B2 R U2 F L D B'" --improve -b 100 -v
```

`--probes N` is repeatable: the same cube and N give the same solution everywhere. `--improve` is timed: it runs for the whole budget and the result depends on the machine. Reaching the budget after a solution was found is not an error. Measured on 500 random cubes (mean moves): 20.83 by default; 19.16 for `--probes 1000` (42 ms per cube); 18.59 for `--probes 10000` (466 ms); 18.99 for `--improve -b 100`; 18.48 for `--improve -b 1000`. The full table and the commands are in [3x3x3 performance](engines/3x3.md#shorter-solutions-moves-against-probes-and-time).

Without `--improve`, `--parallel` is refused, and so is `--parallel` without `-b`: it would never return. The 2x2x2 is already optimal and the 4x4x4 and 5x5x5 have no improve modes yet; all three refuse these options with a one-line error.

### Failure

`solve` prints a message to standard error and exits with a non-zero status when the cube is impossible, the limit is too tight, or the time budget runs out. For example:

```sh
cubacle solve -s "R U F B L D R U" --max-moves 2
```

```text
error: no solution within the move limit (the search ended without a solution (time budget or move limit reached))
```

The 6x6x6 and 7x7x7 solve by reduction; see [Engines and limits](engines/_index.md).

## Solve a 4x4x4

Give `solve` a 96-sticker cube, or a scramble with `--size 4`. A 4x4x4 scramble may turn the first two layers of a face (`R`, `2R`) and use wide turns (`Rw`); the 4x4x4 has no fixed centres, so the solver brings the cube to the standard colour scheme and the solution may end with whole-cube rotations.

```sh
cubacle solve --size 4 --scramble "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw"
```

```text
Uw' Rw' Uw L' Fw U' B2 Rw' D B Uw2 Rw' Fw2 R2 Rw2 U F Fw2 L2 U' D' Rw2 F' Uw2 L U L' D' B L U2 L' D' F2 B2 U' L2 F2 U2 F2 D' L2 B Rw' Lw Fw Bw'
```

The solution has 47 moves. The last four, `Rw' Lw Fw Bw'`, are the rotation: each pair of opposite wide turns, such as `Rw' Lw`, turns the whole cube, and at most four turns bring any cube to the standard orientation. The solutions of 1000 random cubes have 47.5 moves on average (44.3 without the rotation). The tool replays every 4x4x4 solution on the cube and exits with an error if it does not solve it. With a move limit the solution cannot meet, `solve` fails:

```sh
cubacle solve --size 4 -s "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" -m 30
```

```text
error: no solution within the move limit (3x3x3 stage: the search ended without a solution (time budget or move limit reached))
```

The first 4x4x4 solve prints `preparing the 4x4x4 tables (built now, about 12 MB)` on stderr and builds the tables, which takes about 0.2 s on 12 cores and 0.4 s on one; there is no cache directory. The preparation happens before the time budget starts, so `-b 1` fails with `the time budget ran out (the reduction ran out of time before the 3x3x3 stage)`. `CUBACLE_THREADS` sets the number of threads (at most 256); the solution does not depend on it unless a time budget cuts the search short. `-v` prints the time and the number of probes of the 3x3x3 stage (500: that stage is asked for at least 500 probes, as in the original solver).

## Solve a 5x5x5

Give `solve` a 150-sticker cube, or a scramble with `--size 5`. A 5x5x5 scramble may turn the first two layers of a face (`R`, `2R`); the middle layer (`3R`, `3Rw`) is not allowed, because it moves the face centres, which the solver keeps fixed. Two-layer turns (`Rw`) are fine.

```sh
cubacle solve --size 5 --scramble "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F" | wc -w
```

```text
67
```

The solutions have 73 moves on average: 70 to 76 over 100 random cubes and 69 to 77 over 300 others (see [Engines and limits](engines/_index.md)). It starts like this:

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

## Input notations

Wherever a command takes a cube (`solve`, `convert`), you can write it in one of three notations. Spaces are allowed between any groups and need no quoting.

### Orbit64 tokens

An [Orbit64](https://github.com/wstein/flix-orbit64) token is a short string that encodes a cube state. Its length tells the cube size, so no flag is needed. This command solves a 3x3x3 token:

```sh
cubacle solve A4ib8aejxj4Y
```

This 2x2x2 token converts to facelets:

```sh
cubacle convert CFJnI
```

```text
UUUUBFRRFRFFDDDDLLLLRBBB
```

`convert --to token` writes a token instead. It accepts facelets, colours or a token:

```sh
cubacle convert --to token LFDRFLDBFURLBBLUURUDDFRB
cubacle convert --to token "$(cubacle facelets "R U 2R 2U' F" --size 6)"
```

```text
ASj79
A3DZCufPmdSpgSdAXL5x0iJAczTlcP4refzTMJv_VgBoTFSReBHZMvFpjDdnSJzhHA
```

Tokens exist for every size from 2x2x2 to 7x7x7. `solve` and `convert` read all of them.

| Cube | Token length | Checked against |
| --- | ---: | --- |
| 2x2x2 | 5 | 20 official examples |
| 3x3x3 | 12 | 20 official examples, the format's compatibility vectors |
| 4x4x4 | 27 | 20 official examples |
| 5x5x5 | 43 | 20 official examples |
| 6x6x6 | 66 | the official solved cube, and round trips only |
| 7x7x7 | 90 | the official solved cube, and round trips only |

"Round trips only" means the 6x6x6 and 7x7x7 slot numbering follows the rules of the format but has no official scrambled vector to compare with: tests scramble cubes with outer, wide and inner moves, and decode random points of the state space, and each time `encode` and `decode` agree and the token has the length above. A token made by another program for these sizes may use a different slot numbering; see the format's own caveat. A token with a wrong length, a character outside the base64url alphabet, a number above the state count, or (when encoding) pieces that no cube has, is refused with a one-sentence error.

### Facelets (URFDLB)

Each sticker is named by the face it belongs to: `U`, `R`, `F`, `D`, `L` or `B`. Stickers are listed face by face in the order U, R, F, D, L, B. A face has 4 stickers on a 2x2x2 and 9 on a 3x3x3. The solved 3x3x3 is:

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

You can write facelets compactly or spaced in any grouping. These two commands solve the same cube:

```sh
cubacle solve FUFUUFFFF UUUURRURU RRRFFRRFR BDBBDDBBB DLDDLLDDD LBLBBLLLL
cubacle solve FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL
```

A 2x2x2 works the same way:

```sh
cubacle solve UUUU BFRR FRFF DDDD LLLL RBBB
```

```text
U R U' R F2 R' U R F2 R2
```

### Colours

Each sticker is a colour letter: `W`, `R`, `G`, `Y`, `O` or `B`. Compact and spaced forms both work.

```sh
cubacle solve GWGWWGGGG WWWWRRWRW RRRGGRRGR BYBBYYBBB YOYYOOYYY OBOBBOOOO
```

This prints the same solution as the Orbit64 example above. On a cube with fixed centres (3x3x3), cubacle reads the scheme from the centres, so western and Japanese cubes need no flag.

### Colour schemes

A cube without centres, such as the 2x2x2, needs a scheme to map colours to faces. Use `--scheme`.

| Scheme | U R F D L B | Opposite pairs |
| --- | --- | --- |
| `boy` (default) | W R G Y O B | white-yellow, green-blue, red-orange |
| `jp` | W R G B O Y | white-blue, green-yellow, red-orange |

You can also give six letters in URFDLB order, such as `--scheme WRGYOB`. The longer names `western`, `western-boy` (for `boy`) and `japanese`, `japanese-bl-wt` (for `jp`) work as aliases.

```sh
cubacle convert --scheme jp WWWW RRRR GGGG BBBB OOOO YYYY
cubacle convert --scheme WRGYOB WWWWRRRRGGGGYYYYOOOOBBBB
```

Both commands print:

```text
UUUURRRRFFFFDDDDLLLLBBBB
```

## Convert a cube

`convert` translates any accepted notation into canonical URFDLB facelets.

```sh
cubacle convert A4ib8aejxj4Y
```

```text
FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL
```

`--to` picks the output: `facelets` (default), `token` or `color`. `color` (also spelled `colour`) writes one W/R/G/Y/O/B letter per sticker, in the same order as the facelets output, through the colour scheme named by `--scheme` (default: `boy`, the western scheme, white up and yellow down). `--scheme western` is another name for `boy`; `jp` and six letters in URFDLB order work too. The text reads back with `convert --scheme <same>`.

```sh
cubacle convert A4ib8aejxj4Y --to color
cubacle convert A4ib8aejxj4Y --to color --scheme jp
```

```text
GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO
GWGWWGGGGWWWWRRWRWRRRGGRRGRYBYYBBYYYBOBBOOBBBOYOYYOOOO
```

`convert` also takes a move expression (see [Apply a scramble](#apply-a-scramble)); `--size` picks the cube. This nested expression of slices, rotations and repeats on a 7x7x7 gives 294 letters, one per sticker:

```sh
cubacle convert --size 7 "((M' U)*4 y' z')*3" --to color --scheme western
```

```text
WWWBWWWWWWBWWWWWWBWWWOWWWWWRWWWBWWWWWWBW...
```

The output is shortened here; it ends with `...BBBYBBBBBBYBBBBBBYBBB`.

## Generate scrambles and cubes

`scramble` prints a random scramble. `-n` sets the number of moves (default: 25) and `--size` the cube size (default: 3). On a 4x4x4 or 5x5x5 the scramble also turns second layers.

```sh
cubacle scramble -n 8
```

```text
L' U F' R2 F L2 F' D2
```

The output is random; moves are separated by single spaces. `random` prints a random state as facelets, uniform for the 2x2x2 and 3x3x3. `--size` takes 2, 3, 4 or 5 (default: 3); the 4x4x4 and 5x5x5 states are uniform shuffles of corners, wings and centres:

```sh
cubacle random
cubacle random --size 5 | cubacle solve - | wc -w
```

The second command prints the length of the solution of a random 5x5x5, between 69 and 77 in the 300 cubes measured.

```sh
cubacle random --size 4 | cubacle solve - | wc -w
```

This prints the length of the solution of a random 4x4x4 (rotation included), between 43 and 50 in 100 cubes measured (47.5 on average); the 1000 cubes drawn by the original solver's generator gave 42 to 51 (see [Engines and limits](engines/_index.md)).

## Apply a scramble

A scramble is standard move notation with the moves separated by whitespace. Besides the canonical forms (`R`, `R2`, `R'`, and wide or inner-layer moves such as `Rw` or `2R`), these whole-token aliases are accepted:

| Written | Read as |
| --- | --- |
| `R1` | `R` |
| `R3` | `R'` |
| `R2'`, `R'2` | `R2` |

Moves must be separated: a run such as `RUR'` and a lone `-` are rejected. A 3x3x3 whose centres a wide or inner-layer move has displaced cannot be solved; `solve` reports that instead of a wrong solution.

### Repeated groups

`( ... )` groups moves and `*N` repeats the group, or a single move, right before it N times. Groups nest, and parentheses need no spaces around them; `*N` must follow its group or move directly (`R *2` and `R*0` are errors, as are unbalanced parentheses). An expression may expand to at most 100000 moves. The expression is expanded into plain moves before it is applied, so `(R U)*3` is `R U R U R U`. Six repetitions of the sexy move return a 3x3x3 to the solved state:

```sh
cubacle facelets "(R U R' U')*6"
```

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

### Middle slices and whole-cube rotations

| Written | Meaning |
| --- | --- |
| `M`, `E`, `S` | The middle layer of an odd cube, turned like `L`, like `D` and like `F`. On a 3x3x3 `M` is `2L`; on a 7x7x7 it is `4L`. |
| `x`, `y`, `z` | The whole cube turned like `R`, like `U` and like `F`. |

All of them take `'`, `2` and the aliases above, and `*N`. An even cube (2x2x2, 4x4x4, 6x6x6) has no middle layer and refuses `M`, `E` and `S` with an error; `x`, `y` and `z` work on every size. Wide forms such as `Mw` and prefixed forms such as `2M` are not accepted. The expression is expanded into plain layer turns: a rotation becomes a wide turn of all layers but one plus the opposite outer face turned the same way round the axis, so `x` on a 3x3x3 is `2Rw L'`. The standard identities hold and are tested: `x = R M' L'`, `y = U E' D'`, `z = F S B'`.

A rotation moves the centres, so the state after it is a rotated cube, not a solved-looking one: after `x` a solved 3x3x3 shows `F` on the up face.

```sh
cubacle facelets "x"
```

```text
FFFFFFFFFRRRRRRRRRDDDDDDDDDBBBBBBBBBLLLLLLLLLUUUUUUUUU
```

```sh
cubacle facelets "M"
```

```text
UBUUBUUBURRRRRRRRRFUFFUFFUFDFDDFDDFDLLLLLLLLLBDBBDBBDB
```

`facelets` and `--scramble` read the same notation. `convert` and `solve` also take an expression in place of a cube when the text is not a cube; `--size` picks the cube:

```sh
cubacle convert --size 3 "(R U)*6"
```

```text
FULUUBRUURRUFRRBRUDLFFFUFFRDDDDDFDDRLBFLLLLLLBRUDBBBBB
```

`facelets` prints the facelets reached by applying a scramble to a solved cube (`--size` picks the cube, default 3):

```sh
cubacle facelets "R U R' U'"
```

```text
UULUUFUUFRRUBRRURRFFDFFUFFFDDRDDDDDDBLLLLLLLLBRRBBBBBB
```

## Draw moves

`icon` draws moves as small pictures, and `solve --icons` shows each step of a solution with one. See [Move icons](icons.md).

```sh
cubacle icon R "U'" --format text
```
