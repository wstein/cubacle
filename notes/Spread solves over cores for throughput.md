---
id: 20261022150101
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
For many solves, one thread per solve beats the multithreaded algorithm: 4x4x4 gives 4.4x the throughput (106 against 24 solves per second, 100 cubes), 5x5x5 gives 1.26x, best as 3 solves of 4 threads. Measured on the M2 Pro (8 P and 4 E cores); the server runs 5x5x5 as 12 permits of 12 threads, the slow case (10.3 per second against 14.5).

## Links

- [[4x4 thread scaling is capped by the hand-off]]
- [[5x5 reduction engine]]

Source: docs/performance.md, lines 65-91
