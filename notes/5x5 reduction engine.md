---
id: 20261022130000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The 5x5x5 engine is named five, not reduction: its stages rely on fixed face centres, which a 4x4 or 6x6 does not have, so a shared engine would be invented before it is needed. It is a port of cs0x7f/cube555 (MIT option, recorded in NOTICE.md) replacing the earlier greedy 3-cycle engine, which is deleted without a fallback. The generic parts (pruning-table builder, cache, parallel IDA*) live in `src/reduction/`, and the 3x3x3 stage goes to an injected Solver, never a global.

## Links

- [[Reduction engines and the 3x3 solver]]
- [[5x5 parity and limits]]
- [[5x5 port plan]]

Source: AGENTS.md, lines 40-43
Source: ARCHITECTURE.md, lines 67-84
Source: NOTICE.md, lines 17-31
Source: src/engines/five/mod.rs, lines 1-10
Source: src/reduction/mod.rs, lines 1-10
