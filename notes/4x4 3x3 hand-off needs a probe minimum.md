---
id: 20261022140005
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The reference asks min2phase for at least 500 probes, so its 3x3x3 stage averages 19.60 moves, while a plain trait call stops at the first solution (20.84 moves on 500 random states). SolveOptions has a probe-count minimum for this, counted and not timed, so the answer is the same on every machine; the 3x3x3 engine honors it and FourSolver asks for 500 unless the caller sets another number. The field is public, so code that builds SolveOptions by field must add it.

## Links

- [[4x4 versus TPR]]
- [[Reduction engines and the 3x3 solver]]

Source: docs/design/4x4-port.md, lines 235-254
Source: src/puzzle/solver.rs, lines 5-20
Source: src/engines/four/solver.rs, lines 247-253
