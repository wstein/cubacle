---
title: "Command-line guide"
description: "Use the cubacle command-line tool."
weight: 3
---

# Command-line guide

The `cubacle` tool has five subcommands: `solve`, `convert`, `scramble`, `random` and `facelets`. Run `cubacle <command> --help` for the full option list.

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
elapsed 19.028292ms, probes 1
U R U R F2 R F2 R L' U2 R' U2 R2 L U2 R'
```

The first line goes to standard error. The solution goes to standard output. The elapsed time varies between runs.

| Option | Meaning |
| --- | --- |
| `-s`, `--scramble <MOVES>` | Solve the cube produced by this scramble instead of reading one. |
| `--scheme <SCHEME>` | Colour scheme for colour input. See [Colour schemes](#colour-schemes). |
| `-m`, `--max-moves <N>` | Longest solution to accept. Default: 21. |
| `-b`, `--budget-ms <MS>` | Hard time limit in milliseconds. |
| `-v`, `--verbose` | Print timing and search statistics to standard error. |

A cube argument of `-` reads the cube from standard input:

```sh
cubacle random | cubacle solve -
```

### Failure

`solve` prints a message to standard error and exits with a non-zero status when the cube is impossible, the limit is too tight, or the time budget runs out. For example:

```sh
cubacle solve -s "R U F B L D R U" --max-moves 2
```

```text
error: no solution within the move limit (the search ended without a solution (time budget or move limit reached))
```

Sizes from 4x4x4 up have no engine yet. See [Engines and limits](engines.md).

## Input notations

Wherever a command takes a cube (`solve`, `convert`), you can write it in one of three notations. Spaces are allowed between any groups and need no quoting.

### Orbit64 tokens

An [Orbit64](https://github.com/wstein/flix-orbit64) token is a short string that encodes a cube state. This command solves a 3x3x3 token:

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

## Generate scrambles and cubes

`scramble` prints a random scramble. `-n` sets the number of moves (default: 25).

```sh
cubacle scramble -n 8
```

```text
L' U F' R2 F L2 F' D2
```

The output is random; moves are separated by single spaces. `random` prints a uniformly random 3x3x3 state as facelets:

```sh
cubacle random
```

## Apply a scramble

A scramble is standard move notation with the moves separated by whitespace. Besides the canonical forms (`R`, `R2`, `R'`, and wide or inner-layer moves such as `Rw` or `2R`), these whole-token aliases are accepted:

| Written | Read as |
| --- | --- |
| `R1` | `R` |
| `R3` | `R'` |
| `R2'`, `R'2` | `R2` |

Moves must be separated: a run such as `RUR'` and a lone `-` are rejected. A 3x3x3 whose centres a wide or inner-layer move has displaced cannot be solved; `solve` reports that instead of a wrong solution.

`facelets` prints the facelets reached by applying a scramble to a solved cube:

```sh
cubacle facelets "R U R' U'"
```

```text
UULUUFUUFRRUBRRURRFFDFFUFFFDDRDDDDDDBLLLLLLLLBRRBBBBBB
```
