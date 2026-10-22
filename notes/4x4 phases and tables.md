---
id: 20261022140002
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
The reference runs three centre and wing phases on 27, 23 and 17 moves, then a 3x3x3 solver. Phases 2 and 3 take most of its time; phase 3 uses a symmetry-reduced table of 31,006,080 wing states in 2 bits. The Rust tables are one immutable Tables value behind Arc<OnceLock<..>>, built by prepare() or on first use, 12,091,408 bytes, with no table file. Every table matches the Java ones by hash, and the full wing table is built by an atomic frontier search that gives the same bytes for any thread count (248 ms on one thread, 67 ms on four).

## Links

- [[4x4 versus TPR]]
- [[4x4 multi-core plan]]
- [[Cold start and prepare]]

Source: docs/design/4x4-port.md, lines 67-89
Source: src/engines/four/tables.rs, lines 13-54
Source: src/engines/four/edge3_prune.rs, lines 50-70
Source: src/engines/four/edge3_prune.rs, lines 220-234
