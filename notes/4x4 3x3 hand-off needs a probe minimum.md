---
id: 20261022140005
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
The reference asks min2phase for at least 500 probes, so its 3x3x3 stage averages 19.60 moves. Our trait call stops at the first solution: on 500 random 3x3x3 states, 20.84 moves in 1.2 ms against 19.49 when asking again with one move less (25 ms, time-based, not deterministic). Without a probe-count minimum in SolveOptions the port loses about 1.3 moves; adding it touches the 3x3x3 engine and needs approval.

## Links

- [[4x4 versus TPR]]
- [[Reduction engines and the 3x3 solver]]

Source: docs/design/4x4-port.md, lines 235-254
Source: src/puzzle/solver.rs, lines 5-12
