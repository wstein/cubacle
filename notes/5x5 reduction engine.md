---
id: 20261022130000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The 5x5x5 is solved by reduction in an engine named five, not reduction: its stages rely on fixed face centres and on three pieces per edge, which a 4x4 or 6x6 does not have, so a shared engine would be invented before it is needed. It is written from scratch and ports no code; the related solvers (GPL-3.0 or MIT) were only checked for their licence, not read or copied, which NOTICE.md records. Centres and wings are put in order by greedy pure 3-cycles, each a commutator conjugated by a setup of at most four moves, and the 3x3x3 hidden in rows and columns 0, 2 and 4 goes to an injected Solver, never a global.

## Links

- [[Reduction engines and the 3x3 solver]]
- [[5x5 parity and limits]]

Source: AGENTS.md, lines 40-43
Source: ARCHITECTURE.md, lines 67-79
Source: NOTICE.md, lines 21-27
Source: src/engines/five/mod.rs, lines 1-8
Source: src/engines/five/library.rs, lines 10-20
Source: src/engines/five/solver.rs, lines 23-26
Source: owner request of 2026-10-22, lines 1-1
