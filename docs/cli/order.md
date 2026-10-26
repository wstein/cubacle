---
title: "Order of a move sequence"
description: "The order of a move sequence."
weight: 10
---

# Order of a move sequence

## Order of a move sequence

`cubacle order EXPRESSION` prints how many times a move sequence must be repeated to return the cube to the solved state. The expression is read as for `moves` (same notations, groups and repeats), `--size` is 2 to 7:

```sh
cubacle order "R U"
```

```text
order 105
```

The order is defined on the stickers: the sequence moves every sticker to another place, and the order is the least common multiple of the lengths of the cycles of that permutation. `-v` also lists, for each orbit of stickers (corners, edges, wings, centers), the lengths of its cycles with their counts (`7x2` is two cycles of length 7) and the order of the orbit; the order of the whole is their least common multiple:

```sh
cubacle order -v "R U"
cubacle order -v --size 5 "2R U"
```

```text
order 105
corners: 3 15 (order 15)
edges: 7x2 (order 7)
centers: in place (order 1)
order 140
corners: 4x3 (order 4)
edges: 4x2 (order 4)
wings: 10x2 (order 10)
centers: 7 10 (order 70)
```

Because stickers are told apart, a center turned in place counts: on the 4x4x4 and up the order can be a multiple of the number of repetitions after which the colors look solved. `cubacle patterns cycles` follows pieces and gives an order without the centers as well. An input error (a bad move, a layer the cube does not have, a size outside 2 to 7) prints `error: ...` on standard error and exits with 1.
