---
title: "Generate scrambles and cubes"
description: "Generate random scrambles and cubes."
weight: 8
---

# Generate scrambles and cubes

## Generate scrambles and cubes

`scramble` prints a random scramble. `-n` sets the number of moves (default: 25) and `--size` the cube size (default: 3). On a 4x4x4 or 5x5x5 the scramble also turns second layers. `--notation sign` writes the moves in SiGN and `--notation sse` in SSE (see [Move notations](move-notations.md#move-notations)); the default is `wca`.

```sh
cubacle scramble -n 8
```

```text
L' U F' R2 F L2 F' D2
```

The output is random; moves are separated by single spaces. `random` prints a random state as facelets, uniform for the 2x2x2 and 3x3x3. `--size` takes 2, 3, 4 or 5 (default: 3); the 4x4x4 and 5x5x5 states are uniform shuffles of corners, wings and centers:

```sh
cubacle random
cubacle random --size 5 | cubacle solve - | wc -w
```

The second command prints the length of the solution of a random 5x5x5, between 69 and 77 in the 300 cubes measured.

```sh
cubacle random --size 4 | cubacle solve - | wc -w
```

This prints the length of the solution of a random 4x4x4 (rotation included), between 43 and 50 in 100 cubes measured (47.5 on average); the 1000 cubes drawn by the original solver's generator gave 42 to 51 (see [Engines and limits](../engines/_index.md)).
