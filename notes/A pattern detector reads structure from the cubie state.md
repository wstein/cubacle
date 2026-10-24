---
id: 20261024134001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A cube is recognised as a pattern from its structure, not by searching: how many of the 48 cube symmetries (with and without swapping colours) leave the state unchanged, its canonical form under symmetry and colour permutation looked up in the library, the cycle type and orientation sum of every piece orbit, and membership in the subgroups of half turns, domino moves and U/D turns, which are coordinate checks. On 4x4 and up a cubie is a piece orbit (corners, the edges of each layer, the centres of each orbit). These four come first and give `patterns identify`; a second step projects the stickers onto symmetric layer sets that keep both outer layers, which turns a 7x7 into a valid 3x3 or 5x5 whose short solution lifts back, splits the cube into a product of such lifts, and adds a measure of how pattern-like a state is.

## Links

- [[Pattern states are projections onto layer sets]]
- [[One pattern search serves solve, the library and patterns find]]

Source: owner decision of 2026-10-24, lines 1-1
