---
id: 20261022140011
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
Four threads give 0.56 of the single-thread time against the 0.5 aimed at. The 3x3x3 hand-off takes 21 ms of 73 and runs on one thread through the deterministic Solver trait, so 0.49 is the best four threads can do. Reaching the aim would need a 3x3x3 stage that splits its probes over threads without changing the answer, which touches the 3x3x3 search and needs a decision.

## Links

- [[4x4 multi-core plan]]
- [[Reduction engines and the 3x3 solver]]

Source: docs/design/4x4-port.md, lines 484-531
Source: src/engines/four/solver.rs, lines 247-258
