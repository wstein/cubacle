---
id: 20261022140002
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
The reference runs three centre and wing phases on 27, 23 and 17 moves, then a 3x3x3 solver. Phases 2 and 3 take 86 percent of its time and phase 3 uses a symmetry-reduced table of 31,006,080 wing states in 2 bits (7.75 MB). The Rust tables are estimated at about 12 MB; the Java statics become one immutable Tables value behind Arc<OnceLock<..>> with prepare(), per-search buffers live in a searcher, and no table file is written. Table sizes and build time in Rust are estimates until the first table commit.

## Links

- [[4x4 versus TPR]]
- [[4x4 multi-core plan]]
- [[Cold start and prepare]]

Source: docs/design/4x4-port.md, lines 67-89
Source: docs/design/4x4-port.md, lines 118-160
