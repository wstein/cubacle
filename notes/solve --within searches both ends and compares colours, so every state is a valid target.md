---
id: 20261025214000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --within K` finds an optimal route of at most K moves for a slightly scrambled cube by a meet in the middle from the start forward and from the target backward, and the two sides meet on a hash of the colours themselves. No relative state is formed and no equal pieces are paired, so a target is any state of the same size: the solved cube, a move expression, a state text, even one with mixed centre colours that `solve --to` refuses because it cannot pair centres. The hash is exact (renaming two colours changes it): a colour-blind hash would merge a cube and its whole-cube rotation, which on the 4x4x4 and up is two block moves apart, and the stored side would lose the real route. A hit is replayed colour by colour before it is printed, so a hash collision costs at most a missed route, never a wrong one.

The alphabet is the one the normal solver writes (face turns; outer blocks from the 4x4x4), `--slices` adds the single inner layers, `--blocks` adds the blocks of two and three layers and the layers 2-3. The optimum is in that alphabet and its metric (HTM or OBTM for the outer blocks, BTM with slices, counted blocks with `--blocks`). A lower bound from the stickers that differ per orbit, divided by the most stickers one move displaces in that orbit, answers "more than K" without a search. On the 3x3x3 the search beats `--optimal` below seven moves (milliseconds against the half second the large tables need to load), and the 2x2x2 uses it too.

## Links

- [[Repair searches pattern plus a few wide-block moves from both sides]]
- [[A target is solved as a relative state]]
- [[Shorter solutions beat compute time]]

Source: src/within/mod.rs, lines 1-20
