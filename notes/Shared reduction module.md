---
id: 20261022140007
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
The code written once for the 4x4 and the 5x5 lives in src/reduction (crate-private): combinatorics, prune-table builders and the ordered parallel helpers, now with par::first_results, which returns the first results of tasks in the order a serial run finds them. The 4x4 uses the combinatorics and first_results but has its own 2-bit wing table builder. src/reduction keeps no global state and holds no ported code, so it needs no third-party notice.

## Links

- [[4x4 port of the TPR solver]]
- [[Crate roles and dependency direction]]
- [[Reduction engines and the 3x3 solver]]

Source: docs/design/4x4-port.md, lines 327-358
Source: ARCHITECTURE.md, lines 67-79
Source: src/reduction/par.rs, lines 67-118
