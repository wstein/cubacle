---
id: 20261023120002
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The phases of a solution are found by replaying it and ending a phase at the first move after which a state predicate holds for the rest of the solution, never from the engine's own phase boundaries, because the answers are normalised and merged after the search. The module `phases` depends only on `puzzle`. In practice the 4x4x4, 5x5x5 and 6x6x6 engines make centres and edges true at the same move, so the `edges` phase is empty; empty phases keep their number and are shown as `nothing to do`.

## Links

- [[Render design]]
- [[Step-by-step learning solver]]
- [[Crate roles and dependency direction]]

Source: src/phases/mod.rs, lines 1-30
