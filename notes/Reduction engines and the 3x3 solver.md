---
id: 20261022120018
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Reduction engines (4x4 and up) depend on the 3x3 engine only via the Solver trait with an injected instance, not a global, so one prepared 3x3 solver is shared. Consequence: a 4x4 build always includes the 3x3 solver and its tables. Even cubes need parity handling before the 3x3 call. The 3x3 tables belong to the solver instance, not to a global.

## Links

- [[Crate roles and dependency direction]]

Source: AGENTS.md, lines 40-43
Source: ARCHITECTURE.md, lines 50-56
Source: ARCHITECTURE.md, lines 60-73
Source: owner decision of 2026-10-21, lines 1-1
