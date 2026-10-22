---
id: 20261022130001
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
On a 5x5x5 the middle edges fix the orientation of the wings, so no flipped-edge parity exists. Only the permutation parity of the 24 distinguishable wings can be odd, and 3-cycles cannot change it; one inner quarter turn (2R) before the centres repairs it. Solutions are not short: 348 to 477 moves over 1000 random cubes, the 3x3x3 hand-off adding at most 25. Measured in release: 11,650 library cycles, about 0.22 s to build, 1.1 ms median per solve afterwards, near 5 MB peak for a cold run.

## Links

- [[5x5 reduction engine]]

Source: src/engines/five/reduce.rs, lines 1-9
Source: src/engines/five/reduce.rs, lines 128-133
Source: src/engines/five/solver.rs, lines 12-12
Source: docs/engines.md, lines 39-50
Source: owner request of 2026-10-22, lines 1-1
