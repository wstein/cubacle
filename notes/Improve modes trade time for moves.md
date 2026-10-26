---
id: 20261022150002
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The 3x3x3 first solution averages 20.83 moves on 500 random cubes. `solve --probes N` (counted, repeatable) and `solve --improve -b MS` (timed, default 1000 ms) shorten it: 19.16 moves at 1000 probes (42 ms), 18.59 at 10000 probes (466 ms), 18.99 with --improve -b 100 and 18.48 with -b 1000. Searching the six orientations at once (`--parallel`) measured worse than serial --improve at the same budget (19.27 against 18.48 at 1000 ms), so it is offered but not recommended. A budget that ends after a solution exists returns that solution; only a budget that ends before any solution fails.

## Links

- [[Bisect over max_moves is unsound under a time budget]]
- [[4x4 3x3 hand-off needs a probe minimum]]
- [[6x6 and 7x7 improve modes only vary the tail]]

Source: src/improve.rs, lines 49-80
Source: src/engines/three/solver.rs, lines 116-146
Source: docs/engines/3x3.md, lines 45-69
