---
id: 20261022150000
aliases: []
tags: ["type/decision", "status/open", "category/engines"]
---
The owner chose to port the five-phase solver cs0x7f/cube555 to Rust instead of designing a search from scratch, under its MIT option, with NOTICE.md updated before code lands. The port keeps `FiveSolver::new(Arc<dyn Solver + Send + Sync>)`, owns its tables behind `Arc<OnceLock<..>>`, builds 237 MB of 4-bit pruning tables once and caches them on disk, and searches phases 2 to 5 in parallel with an ordered merge so results do not depend on the thread count. Acceptance is the Java length on the same cubes: 51.5 reduction moves and 73.1 in total, against 412.4 for the greedy engine. Tests use depth-capped tables; one ignored test uses the full tables in a separate CI job.

## Links

- [[5x5 reduction engine]]
- [[5x5 versus cube555]]

Source: docs/design/5x5-port.md, lines 14-38
Source: docs/design/5x5-port.md, lines 39-56
Source: docs/design/5x5-port.md, lines 146-186
Source: NOTICE.md, lines 17-31
