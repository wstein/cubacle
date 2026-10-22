---
id: 20261022120010
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
With a fixed corner the state space is 7! x 3^6 = 3,674,160 states. A BFS depth table with maximum depth 11 is stored as 4-bit entries (1,837,080 bytes) behind a OnceLock. Packing matters little for 2x2 and may matter more for 4x4 and 5x5.

## Links

- [[2x2 solver]]

Source: src/engines/two/mod.rs, lines 3-7
Source: src/engines/two/mod.rs, lines 26-27
Source: src/engines/two/mod.rs, lines 265-265
Source: owner decision of 2026-10-21, lines 1-1
