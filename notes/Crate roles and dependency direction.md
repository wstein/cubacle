---
id: 20261022120017
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Roles are puzzle (types and Solver trait), notation, generate, engines (three, two, later reduction) and apps (CLI, server). Dependencies point downward only and engines talk only through the Solver trait. The old string API (`solve(&str)` etc.) is removed; no compatibility with the original code is required.

## Links

- [[Reduction engines and the 3x3 solver]]
- [[Workspace split timing]]

Source: ARCHITECTURE.md, lines 7-20
Source: ARCHITECTURE.md, lines 22-36
Source: AGENTS.md, lines 40-43
Source: README.md, lines 75-77
Source: owner decision of 2026-10-21, lines 1-1
