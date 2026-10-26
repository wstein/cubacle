---
title: "Convert a cube"
description: "Convert a cube between notations."
weight: 7
---

# Convert a cube

## Convert a cube

`convert` translates any accepted notation into canonical URFDLB facelets.

```sh
cubacle convert A4ib8aejxj4Y
```

```text
FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL
```

`--to` picks the output: `facelets` (default), `token` or `color`. `color` (also spelled `color`) writes one W/R/G/Y/O/B letter per sticker, in the same order as the facelets output, through the color scheme named by `--scheme` (default: the scheme the input colors were read in, else `boy`, the western scheme, white up and yellow down; so a Japanese-scheme color cube with its centers converts back into Japanese letters, and tokens, facelets and scrambles come out western). `--scheme western` is another name for `boy`; `jp` and six letters in URFDLB order work too. The text reads back with `convert --scheme <same>`.

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
