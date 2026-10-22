---
id: 20261022160001
aliases: []
tags: ["type/evidence", "status/open", "category/open-items"]
---
The 5x5x5 engine holds 237 MB of pruning tables plus about 70 MB of move and symmetry tables built next to them (measured with vmmap: 17 large allocations, 291 MB), so a warm process is 327 to 335 MB against the 300 MB aim. Streaming the cache read removed a transient second copy of the largest table (467 MB before). An attempt to drop the 10 MB wing table and compute each lookup again made a solve five times slower and was reverted; the u16 candidates are about 3 MB in all. Going lower needs the build-only tables dropped or smaller pruning tables, which would change the cache format.

## Links

- [[5x5 versus cube555]]

Source: src/engines/five/tables.rs, lines 45-64
Source: src/reduction/cache.rs, lines 77-107
