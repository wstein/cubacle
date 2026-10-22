---
id: 20261022140006
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
In the reference, 46 percent of the time is phase 2 and 40 percent phase 3, both loops over independent candidates. The port evaluates candidates on scoped threads and merges in the sequential order (ordered concatenation, lowest successful index), so the answer is the same for every thread count unless a time budget expires; the Edge3 table build uses an idempotent atomic AND per 2-bit entry. Threads come from an explicit setting or CUBACLE_THREADS, with no global pool. Speedups are estimates (about 3 times on 4 threads) until measured.

## Links

- [[4x4 phases and tables]]
- [[Backend concurrency]]

Source: docs/design/4x4-port.md, lines 299-326
Source: src/engines/three/pool.rs, lines 60-70
