---
id: 20261022130001
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
On a 5x5x5 the middle edges fix the orientation of the wings, so no flipped-edge parity exists. Only the permutation parity of the 24 distinguishable wings can be odd, and 3-cycles cannot change it; one inner quarter turn (2R) before the centres repairs it. The solution is not optimal: it is hundreds of moves, a hand-off to the 3x3x3 solver adds at most 25. On a prototype build the 3-cycle library held about 11,600 entries, took about 0.2 s in release to build and the process peaked near 5 MB.

## Links

- [[5x5 reduction engine]]

Source: AGENTS.md, lines 40-43
Source: owner request of 2026-10-22, lines 1-1
