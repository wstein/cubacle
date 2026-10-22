---
id: 20261022140006
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
Candidates of phases 2 and 3 are searched on scoped threads and the results merged in the sequential order (ordered concatenation, lowest successful index), so the answer is the same for 1, 2, 4, 8 and 12 threads (1000 of 1000 cubes). Measured speed: 70 ms mean on one thread, 44.6 ms on four (0.64 times), 43.7 ms on twelve. The aim of 0.5 times was missed: the 3x3x3 hand-off (21 ms of 73) and phase 1 (2 ms) are sequential, which caps four threads at 0.49 even with perfect scaling, and phases 2 and 3 scale by 2.0 and 2.3. Threads come from an explicit setting or CUBACLE_THREADS, with no global pool; the server uses one.

## Links

- [[4x4 phases and tables]]
- [[Backend concurrency]]

Source: src/reduction/par.rs, lines 67-118
Source: src/engines/four/phase2.rs, lines 1-20
Source: src/engines/four/phase3.rs, lines 1-15
Source: docs/design/4x4-port.md, lines 484-531
