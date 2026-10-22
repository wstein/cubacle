---
id: 20261022120018
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Reduction engines (the 4x4 and 5x5 now, 6x6 and up later) depend on the 3x3 engine only via the Solver trait with an injected instance, not a global, so one prepared 3x3 solver is shared. Consequence: a 4x4 build always includes the 3x3 solver and its tables. Even cubes need parity handling before the 3x3 call. The 3x3 tables belong to the solver instance, not to a global.

## Links

- [[Crate roles and dependency direction]]

Source: AGENTS.md, lines 40-43
Source: ARCHITECTURE.md, lines 54-60
Source: ARCHITECTURE.md, lines 67-79
Source: owner decision of 2026-10-21, lines 1-1
