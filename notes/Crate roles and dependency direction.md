---
id: 20261022120017
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Roles are puzzle (types and Solver trait), notation, generate, engines (three, two, later reduction) and apps (CLI, server). Dependencies point downward only and engines talk only through the Solver trait. The old string API (`solve(&str)` etc.) is removed; no compatibility with the original code is required.

## Links

- [[Reduction engines and the 3x3 solver]]
- [[Workspace split timing]]

Source: project history of cubacle, lines 1-1
