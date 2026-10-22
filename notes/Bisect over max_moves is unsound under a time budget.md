---
id: 20261022150001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Do not shorten a 3x3x3 solution by bisecting or laddering the move limit (try 20, then 19, then 18, with the time sliced geometrically between the tries). A failure under a time budget may be a timeout, not proof that no solution of that length exists, so the bisection would discard feasible lengths; only a failure without a budget (Exhausted) proves infeasibility. It also wastes work: each restart repeats the shallow phase one. The search already does better: once a solution is found its length becomes the new target (length minus one), so every later probe looks only for something shorter, and the work continues until the probe count or the minimum time is reached. Use `--probes` (counted, repeatable) or `--improve` (timed) instead.

## Links

- [[Improve modes trade time for moves]]
- [[4x4 3x3 hand-off needs a probe minimum]]

Source: src/engines/three/search.rs, lines 575-590
Source: src/engines/three/search.rs, lines 209-232
Source: src/puzzle/solver.rs, lines 44-58
