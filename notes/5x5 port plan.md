---
id: 20261022150000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner chose to port the five-phase solver cs0x7f/cube555 instead of designing a search from scratch, under its MIT option, with NOTICE.md updated before code lands. It landed as FiveSolver::new(Arc<dyn Solver + Send + Sync>) with tables behind Arc<OnceLock<..>>, a disk cache, and a parallel search merged in a fixed order. The greedy engine was dropped, with no fallback. Acceptance was the Java length on the same cubes, and it is met: 73.1 total moves on 100 cubes.

## Links

- [[5x5 reduction engine]]
- [[5x5 versus cube555]]

Source: docs/design/5x5-port.md, lines 14-38
Source: docs/design/5x5-port.md, lines 39-56
Source: NOTICE.md, lines 17-31
