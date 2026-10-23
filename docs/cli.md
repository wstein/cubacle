---
title: "Command-line guide"
description: "Use the cubacle command-line tool."
weight: 3
---

# Command-line guide

The `cubacle` tool has seven subcommands: `solve`, `recover`, `convert`, `scramble`, `random`, `facelets` and `icon`. Run `cubacle <command> --help` for the full option list.

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
| `--notation <wca\|sign>` | How the solution is written. Default `wca` (`Rw`, `3Rw`); `sign` writes `r`, `3r` (see [Move notations](#move-notations)). With `--icons` it also sets the step lines and the last line; the icon labels stay WCA. |
| `--optimise ranges` | After the solve, with no search: adjacent turns of one axis are written as layer ranges where that is shorter (`2-3r` in sign, `2-3Rw` in wca; only contiguous layers). Off by default. The solution is the same state; stderr reports `moves N` (as solved) and `compressed moves M`, and stdout holds the compressed text. Not combined with `--icons`. On 20 random 4x4x4 to 7x7x7 solutions it saved 0 to 2 moves (2 of 20 saved any, both 7x7x7), on 14 human 4x4x4+ states 0 to 5, because the solvers already merge most axis runs. |
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

## Recover a mistake

**Made a mistake? Do not try to fix it by hand.** Moves made to repair the cube (building centres again, undoing a few turns from memory) only move it further from the printed path, and the search can no longer name the mistake. Stop, rescan the cube and run `cubacle recover`, with `--at-move N` for the move you did last before you noticed.

`recover` helps a person who followed a printed solution and made a mistake. Give it the original cube (the one the solution was made for, written as `solve` takes it), the cube as it is now with `--now`, and optionally the solution that was handed out with `--solution`. Without `--solution` it derives the solution with the same defaults as `solve`; the solver is deterministic for the same cube, so this gives the printed one. It works for every size from the 2x2x2 to the 7x7x7.

The cube below is the one of the first example after `B L' U2 B2 U2 B2 D2`: the person skipped step 5 (`D'`) and went on to step 8.

```sh
cubacle recover -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): step 5 (D') was skipped.
You had done 8 of 21 steps when you stopped.
Recover: do 3 moves D2 B2 D', then continue with step 7 (B2). That is 18 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: recover, as short as solving fresh and it keeps your printed solution.
Moves now: D2 B2 D'
Then continue with step 7 (B2) of the printed solution.
```

The output says one of these things.

- **The cube is on the path**: `you are after step j of n`, and which step comes next.
- **One or two mistakes explain the cube**: the move and the phase of the solution it is in (named when the solution has more than one phase), the printed move, and what was done instead: the step *skipped*, turned by the *wrong amount* (a quarter instead of a half turn or the reverse), the *wrong way*, in *other layers* (`Fw2` for `3Fw2`), on the *wrong face*, *replaced* by some other move, *done twice*, *swapped* with the next step, or an *extra move* made before it. It also says how many steps were done when the cube was entered. When moves that commute give several equal explanations, the earliest step is shown and the number of the others is printed.
- **The cube is a few moves away from the path**, but no mistake is named.
- **Nothing was found** (three or more mistakes, or moves done by hand after the mistake): the fresh solution is printed.

Tell `recover` where you stopped and it searches there first. `--at-move N` says you did move N (counting from 1) and then noticed that the cube no longer matches, so the mistake is at move N or before it and the moves after N are ignored. `--sure-until M` (M below N) adds that you were sure the cube was still right after move M. It is a hint, not a fact, because checking a big cube by eye is unreliable. The search is staged and goes on only when a stage explains nothing:

1. the moves after M (only with `--sure-until`);
2. the moves from the start of the phase that contains move N;
3. the same, from the start of the previous phase;
4. all moves 1 to N;
5. the whole solution, as without `--at-move`.

The phases are the ones `solve -v` reports (for a `--solution` that was not made by `solve`, they are read from the cube). A stop that is a move or two off is tolerated: a skipped move or an extra move shifts the count. `-v` prints which stage answered. On the 7x7x7 attempts of `tests/fixtures/recover_7x7_runs.txt`, the stop alone found the answer in the phase of the stop for four of six explainable cases and one phase earlier for the other two; with a correct `--sure-until` the first stage answered all six.

```sh
cubacle recover -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL --at-move 7 --sure-until 3 -v
```

```text
solution of 21 moves after 41.408208ms
fresh solve 11.25µs
search 277.208µs
stages tried: the window from the move you are sure of; answered by the window from the move you are sure of
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): step 5 (D') was skipped.
You had done 8 of 21 steps when you stopped.
Recover: do 3 moves D2 B2 D', then continue with step 7 (B2). That is 18 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: recover, as short as solving fresh and it keeps your printed solution.
Moves now: D2 B2 D'
Then continue with step 7 (B2) of the printed solution.
```

(The timings on standard error differ from run to run.)

Then two ways forward are compared: the *recover* (the moves that bring the cube back onto the printed path, followed by the rest of the printed solution) and a *fresh* solution of the cube as it is, which runs beside the search. The recover is either `undo N moves` (the inverse of what was done since the mistake) or a shorter correction found by a search of up to `--depth` moves (default 3) from the cube to any cube of the printed path. The shorter total wins, a tie goes to the recover, and a recover is never recommended when it is longer than the fresh solution. The chosen moves follow `Moves now:` in the notation of `solve`. With `--solution` of the same 3x3x3 and the search limited to undoing, the fresh solution wins:

```sh
cubacle recover -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL --solution "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'" --one-mistake --depth 0
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): step 5 (D') was skipped.
You had done 8 of 21 steps when you stopped.
Recover: undo 3 moves by turning D2 B2 U2, then continue with step 5 (D'). That is 20 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: solve fresh, shorter than the recover (20 moves).
Moves now: D2 B2 D' B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'
```

On a 4x4x4, step 4 of the solution (`F'`) turned the wrong way:

```sh
cubacle recover --size 4 -s "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" --now UDRRDDUUUUUBBLRFDDRUBDUURBRUDFDFDBDLDBFLDLLFRRRLUBBBFBDURLDFDUFULLBRDRFBFBRRRULBFFLBLLRFLFFULLBF
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 4, phase 1 (centres staged): step 4 (F') was turned the wrong way (F).
You had done 9 of 43 steps when you stopped.
Recover: undo 6 moves by turning B' Uw L2 F Lw' F', then continue with step 4 (F'). That is 46 moves in all.
Fresh: solve your cube again from where it is, 42 moves.
Recommended: solve fresh, shorter than the recover (46 moves).
Moves now: R2 D' R U' Fw D2 Fw B2 U2 F2 Dw' D2 F' Dw2 L B' R L' Bw2 Rw2 F' Bw2 Rw2 D2 F2 R2 F' R2 B R2 L2 B L D R' F2 U' R' B R' L D'
```

(plus `preparing the 4x4x4 tables ...` on standard error.) Only if the normal search finds nothing, the 23 other orientations of the whole cube are tried, and a cube that fits one is reported with the turns (`x`, `y`, `z`) that rotate it back: `Your cube is held differently from the original: turn the whole cube (x') to match it.` The moves printed then assume that orientation.

| Option | Meaning |
| --- | --- |
| `--now <CUBE>` | The cube as it is now, an Orbit64 token, facelets or colours (quote it when it has spaces). Required. |
| `--solution <MOVES>` | The solution the person followed, in WCA or SiGN, groups and ranges allowed (see [Move notations](#move-notations)). Default: the one `solve` prints for the original cube. |
| `--notation <wca\|sign>` | How the moves in the explanation and in `Moves now:` are written. Default `wca`. |
| `--at-move <N>` | The move you did last before you noticed the mismatch, counting from 1. The mistake is at move N or before; later moves are ignored. |
| `--sure-until <M>` | Needs `--at-move`. The move after which you are sure the cube was right (M before N). A hint: the search widens past it. |
| `--fresh` | Skip the search and print the solution of the current cube. |
| `--depth <N>` | Moves the search for a way back onto the path may use, 0 to 5 (default 3). Measured on a 7x7x7: 3 adds about 20 ms per orientation, 4 makes the whole search take 3 to 5 s, 5 reaches the default time limit and holds about 70 MB. Turned cubes are searched at depth 3 at most. |
| `--one-mistake` | Look for one mistake only. The default also looks for two. |
| `--thorough` | For two mistakes, try every move instead of it and every extra move. Slow on big cubes (it does not finish within the default time limit on a 7x7x7). |
| `--max-time <MS>` | Time limit of the search in milliseconds, 0 for none (default 8000). The result then says that the search stopped at its time limit. |
| `-v`, `--verbose` | Print the time of the solve, the fresh solve and the search to standard error. |

The original cube also takes `--scramble`, `--size` and `--scheme`, as in `solve`. The search treats a move as a permutation of the stickers, so it is the same for every size. Measured on a 7x7x7 (12 cores, with the fresh solve running beside the search): a mistake is found in 30 to 250 ms; a cube that nothing explains takes 1 to 4 s because all 24 orientations are tried; the fresh solve takes 3 to 12 s and the solution of the original cube 4 to 8 s. The mistakes are tried in two levels: one mistake, with every move as a replacement or an extra move, and two mistakes of the likely kinds only (skipped, wrong amount, direction, layers or face, swapped, done twice).

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

`--to` picks the output: `facelets` (default), `token` or `color`. `color` (also spelled `colour`) writes one W/R/G/Y/O/B letter per sticker, in the same order as the facelets output, through the colour scheme named by `--scheme` (default: the scheme the input colours were read in, else `boy`, the western scheme, white up and yellow down; so a Japanese-scheme colour cube with its centres converts back into Japanese letters, and tokens, facelets and scrambles come out western). `--scheme western` is another name for `boy`; `jp` and six letters in URFDLB order work too. The text reads back with `convert --scheme <same>`.

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

`scramble` prints a random scramble. `-n` sets the number of moves (default: 25) and `--size` the cube size (default: 3). On a 4x4x4 or 5x5x5 the scramble also turns second layers. `--notation sign` writes the moves in SiGN (see [Move notations](#move-notations)); the default is `wca`.

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

A scramble is move notation in WCA or SiGN spelling (see [Move notations](#move-notations)) with the moves separated by whitespace. Besides the canonical forms (`R`, `R2`, `R'`, and wide or inner-layer moves such as `Rw` or `2R`), these whole-token aliases are accepted:

| Written | Read as |
| --- | --- |
| `R1` | `R` |
| `R3` | `R'` |
| `R2'`, `R'2` | `R2` |

Moves must be separated: a run such as `RUR'` and a lone `-` are rejected. A 3x3x3 whose centres a wide or inner-layer move has displaced cannot be solved; `solve` reports that instead of a wrong solution ("the centres have moved"). The same holds for a rotation (`x`, `y`, `z`): `cubacle solve -s x` on a 3x3x3 fails, while the 2x2x2 and 4x4x4, which have no fixed centres, solve such a cube (`cubacle solve --size 2 -s x` works).

### Repeated groups

`( ... )` groups moves and `*N` repeats the group, or a single move, right before it N times. Groups nest, and parentheses need no spaces around them; `*N` must follow its group or move directly (`R *2` and `R*0` are errors, as are unbalanced parentheses). An expression may expand to at most 100000 moves. The expression is expanded into plain moves before it is applied, so `(R U)*3` is `R U R U R U`. Six repetitions of the sexy move return a 3x3x3 to the solved state:

```sh
cubacle facelets "(R U R' U')*6"
```

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

After a group, `*N` or a bare `N` repeats it N times and `'` inverts it (the moves in reverse order, each inverted); both can be given, in either order: `(R U)3`, `(R U)'`, `(R U)2'` (inverse of two repeats). After a single move only `*N` repeats (`R2` stays a half turn). Run in this build:

```sh
cubacle facelets "(R U)2'"
cubacle facelets "U' R' U' R'"
```

print the same cube twice.

### Commutators and conjugates

`[A, B]` is the commutator `A B A' B'` and `[A: B]` the conjugate `A B A'`. `A` and `B` are any expressions (groups, rotations, slices, ranges and further brackets, nested up to 100 deep) and may be written with or without spaces; a bracket can take the same suffixes as a group (`[R, U]2`, `[R, U]'`). Each bracket has exactly one `,` or `:`; use a nested bracket for more. Both sides must hold moves. The 100000-move limit counts the expanded moves, so `[(R U)*40000, F]` is refused.

```sh
cubacle facelets "[R U: F]" # is R U F U' R'
cubacle facelets "R U F U' R'"
```

Errors name the character (counted from 0) where the problem starts: `R [U, F, D]` fails at character 7, an unclosed `[` at its own position, and a layer the cube lacks (`2-3r` on a 3x3x3) at the start of the word.

### Middle slices and whole-cube rotations

| Written | Meaning |
| --- | --- |
| `M`, `E`, `S` | The middle layer of an odd cube, turned like `L`, like `D` and like `F`. On a 3x3x3 `M` is `2L`; on a 7x7x7 it is `4L`. |
| `x`, `y`, `z` | The whole cube turned like `R`, like `U` and like `F`. |

All of them take `'`, `2` and the aliases above, and `*N`. An even cube (2x2x2, 4x4x4, 6x6x6) has no middle layer and refuses `M`, `E` and `S` with an error; `x`, `y` and `z` work on every size. Wide forms such as `Mw` and prefixed forms such as `2M` are not accepted. The expression is expanded into plain layer turns: a rotation becomes a wide turn of all layers but one plus the opposite outer face turned the same way round the axis, so `x` on a 3x3x3 is `Rw L'` (`Rw` is the two layers on the right). The standard identities hold and are tested: `x = R M' L'`, `y = U E' D'`, `z = F S B'`.

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

`facelets` and `--scramble` read the same notation.

For `convert` and `solve` the text is tried as a cube first (token, facelets, colours), and only when it is none of those as a move expression. So `U U U U R R R R F F F F D D D D L L L L B B B B` is read as the 2x2x2 facelets it spells, whereas `facelets` and `--scramble` always read a scramble: use `--scramble` when the text could be both. `--size` is not compared with the size of a cube given as text. When the text is neither, the error gives both reasons. `convert` and `solve` also take an expression in place of a cube when the text is not a cube; `--size` picks the cube:

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

## Move notations

Every command that reads moves (`solve --scramble`, `facelets`, `convert --scramble`, `icon`) reads WCA and SiGN spellings, token by token and without an option. `solve` and `scramble` write either one with `--notation wca|sign` (default `wca`). The move count and the engines are the same in both: every move counts one. `convert` and `random` print a cube, not moves, so they have no such option.

| What is turned | WCA | SiGN |
| --- | --- | --- |
| The outer layer | `R` | `R` |
| Only layer n (n from 2) | `2R`, `3R` | `2R`, `3R` |
| The two outer layers | `Rw` (or `2Rw`) | `r` (or `2r`) |
| The n outer layers (n from 3) | `3Rw`, `4Rw` | `3r`, `4r` |

A turn ends in nothing (clockwise), `2` or `'`, as before (`3r2`, `u'`). The notations agree on every uppercase spelling and WCA has no lowercase one on big cubes, so no spelling has two meanings and nothing needs a tie-break. If a spelling ever did differ between the two, WCA would win; today none does. Mixed input such as `R u 3Rw 2R` is therefore read as it stands. Output in SiGN writes `u` for the two-layer turn (SiGN omits the 2) and `3u` for three.

#### Layer ranges

SiGN writes the layers a to b of a face as `a-b` before the letter: `2-3r` is the second and third layer from the right, `2-4l2` a half turn of layers 2 to 4 from the left, `1-3r` is `3r`, and `2-2r` is `2R`. A range is expanded when it is read into plain moves, the outer b layers turned together and the outer a-1 layers turned back, so the engines and the move count never see it: `2-3r` is `3Rw R'`, `3-5r` is `5Rw Rw'`, `2-4l2` is `4Lw2 L2`. Layers are counted from the face named, and a is not larger than b. As an extension of this program, which neither source has, the range may be written with `Rw` instead of the lowercase letter: `2-3Rw2` equals `2-3r2`. The tests turn every range of every size from the 2x2x2 to the 7x7x7 and compare it with turning the same layers one by one.

Rejected, with the token in the message: `rw` and `3rw` (a lowercase letter already means a wide turn), `1r` and `0r`, a turn count after the letter that is not 2 or a quote, and ranges that run backwards (`3-2r`), start at 0 (`0-2r`), lack a letter (`2-3`), use a capital without `w` (`2-3R`) or reach a layer the cube lacks. The face letters are `U R F D L B`; the whole-cube rotations `x y z` and the slices `M E S` are read in expressions and by `icon`.

#### Where moves are read and written

Moves are read as WCA or SiGN, with groups, commutators and ranges, by `solve --scramble`, `convert --scramble` and `facelets`, by `recover --solution`, by `icon` (its moves and `--from`), and by the `scramble` field of the REST API. Moves are written in either notation by `--notation wca|sign` on `solve` (the solution, the `--icons` step lines and the last line; the icon pictures keep WCA letters), `scramble` and `recover` (every move in the explanation and in `Moves now:`).

```sh
cubacle facelets "r U f'"
cubacle scramble --size 7 -n 6 --notation sign
cubacle solve --size 4 -s "r 2R f' 3u" --notation sign
```

```text
UUUURRURRBBBBBRDDRRDDRDDRFFDLLDLLDBBFFFLFFLFFLLLUUBUUB
2L 2U2 3F' 3U' L U2 2B2 2L'
D' l f2 F u' d
```

The first line is the same cube as `cubacle facelets "Rw U Fw'"`. The scramble is random. A 7x7x7 answer reads `U 3Uw 3Lw' B' 3Rw ...` in WCA and `U 3u 3l' B' 3r ...` in SiGN.

Sources read for these rules (2026-10-23):

- WCA Regulations, Article 12a2 (<https://www.worldcubeassociation.org/regulations/#article-12-notation>): `nRw` moves the n outer layers, `Rw` is `2Rw`, `1Rw` is invalid, `3Rw` is valid from the 4x4x4. The article defines no `2R` and no lowercase big-cube letters (lowercase only for Pyraminx vertices).
- The SiGN description on mzrg.com (<https://mzrg.com/rubik/nota.shtml>): capital letters turn a single layer, lowercase letters several layers; a number before the letter names the layer, or for lowercase the deepest layer of a turn that includes the outer one, with `u` for `2u`; a range such as `2-5r` is written with a dash.

Depth conventions: the sources agree on `2R` (only the second layer) and `3r` (the three outer layers), so there was no choice to make. `3R` (only the third layer) is the same in both. The one choice is `2r`, which mzrg writes `r`: it is accepted on input and written `r`. The speedsolving.com wiki could not be fetched (HTTP 403), and a forum proposal for `r3` was not taken as a convention.
