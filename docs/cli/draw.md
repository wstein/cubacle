---
title: "Draw moves"
description: "Draw moves in the terminal."
weight: 11
---

# Draw moves

## Draw moves

`moves` prints the plain move sequence an expression stands for (groups, commutators, slices and rotations expanded; `--size N`, `--notation wca|sign|sse`; several words need no quoting, `-` reads stdin):

```sh
cubacle moves --size 7 "(R2 F2 L2)*2"
```

```text
R2 F2 L2 R2 F2 L2
```

With `--icons` it draws the moves as small pictures, as `solve --icons` does for the moves of a solution, each with its move number (`--numbers off` hides them). See [Move icons](../icons.md). `--group dots|none` (default `dots`, needs `--icons`) decides whether `·` and `( )` in the input draw groups; see [Hand units](../icons.md#hand-units).

```sh
cubacle moves R "U'" --icons --format text
```
