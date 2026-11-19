---
title: "Convert a cube"
description: "Convert a cube between notations."
weight: 7
---

# Convert a cube

## Convert a cube

`convert` translates any accepted notation into canonical URFDLB facelets.

<!-- example:convert-facelets -->
```sh
cubacle convert A4ib8aejxj4Y
```

```text
FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL
```
<!-- /example -->

`--to` picks the output: `facelets` (default), `token` (Orbit64 V1), `token2` (Orbit64 V2, below) or `color`. `color` (also spelled `color`) writes one W/R/G/Y/O/B letter per sticker, in the same order as the facelets output, through the color scheme named by `--scheme` (default: the scheme the input colors were read in, else `boy`, the western scheme, white up and yellow down; so a Japanese-scheme color cube with its centers converts back into Japanese letters, and tokens, facelets and scrambles come out western). `--scheme western` is another name for `boy`; `jp` and six letters in URFDLB order work too. The text reads back with `convert --scheme <same>`.

```sh
cubacle convert A4ib8aejxj4Y --to color
cubacle convert A4ib8aejxj4Y --to color --scheme jp
```

```text
GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO
GWGWWGGGGWWWWRRWRWRRRGGRRGRYBYYBBYYYBOBBOOBBBOYOYYOOOO
```

`convert` also takes a move expression (see [Apply a scramble](moves.md#apply-a-scramble)); `--size` picks the cube. With `--metrics` it also prints the seven [move counts](../metrics.md) of the expression on stderr (a cube as input is refused). This nested expression of slices, rotations and repeats on a 7x7x7 gives 294 letters, one per sticker:

```sh
cubacle convert --size 7 "((M' U)*4 y' z')*3" --to color --scheme western
```

```text
WWWBWWWWWWBWWWWWWBWWWOWWWWWRWWWBWWWWWWBW...
```

The output is shortened here; it ends with `...BBBYBBBBBBYBBBBBBYBBB`.

## Orbit64 V2 tokens

`--to token2` writes the [Orbit64 V2](https://github.com/wstein/flix-orbit64) state token. A V2 token has no class bits: its length and value select a registry entry (the cube size `2` to `15`, a model and a pose). Two options choose the entry:

- `--model color` (default) writes a color cube, where equal centers are not told apart. `--model picture` writes a picture cube, where every sticker is distinct; colors do not tell the stickers of a picture cube apart, so this works for the 3x3x3 only, with the six fixed centers upright (turn 0). A state whose corner permutation is odd has no such picture state (the turns of the centers must have the parity of the corner permutation) and is an error that says so; so is any other size.
- `--pose framed` (default) keeps the whole-cube orientation in the token, so the token gives the stickers back exactly as they are. `--pose canonical` turns the cube to its reference pose first: cubes that differ only by a whole-cube rotation get one token.

<!-- example:convert-token2 -->
```sh
cubacle convert A4ib8aejxj4Y --to token2
```

```text
A4ib8aejxj4Y
```
<!-- /example -->

The color framed token of the 2x2x2, 3x3x3, 4x4x4, 6x6x6 and 7x7x7 is the V1 token of the same state; only the 5x5x5 differs: its V2 token has 42 characters, a V1 token 43. `--model` and `--pose` are refused without `--to token2`.
