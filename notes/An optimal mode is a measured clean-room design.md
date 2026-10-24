---
id: 20261024160000
aliases: []
tags: ["type/design", "status/open", "category/engines"]
---
An optimal mode (`solve --optimal`) finds a shortest 3x3 solution with a large pruning table that is built once. It is a clean-room design from the ideas of the vcube study: a 256-bit cubie state, a lower bound from the cube, its inverse and conjugates, a stripe-minimum table of two bits per entry, prefetched lookups and an IDA* search with canonical move masks. vcube is GPL-3.0, so only ideas are used, never code or text. min2phase runs first and gives the upper bound and the fallback, so the optimal mode never returns a longer solution than before. The first step is a measurement: build the small table (about 0.45 GiB), time it and record the length on 100 random cubes, then decide the table sizes. Placed after the don't-care targets in the task list.

## Links

- [[Shorter solutions beat compute time]]
- [[Pattern search speed comes from measuring first, then symmetry, compact states and cores]]
- [[A study of stickersolve shows what to adopt and what not]]
- [[Licensing]]


Source: docs/patterns.md, lines 1-10
