---
id: 20261023120002
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The phases of a solution come from the engine that made it: `SolveResult` carries `stages`, each with a name and the number of moves at its end. The engine notes its stage ends on the raw moves it appends and carries them through its normalizing and rotation by cube state (`stage_ends`: the first final prefix that reaches the raw state, up to a whole-cube rotation on the 4x4x4, else the closest). A state predicate cannot see the staging steps of the 6x6x6 and 7x7x7, which are undone and redone; the cut by state gave one phase of two thirds of the moves. A phase that would be empty for every answer of an engine (`edges` of the big cubes) is not in its list; other empty phases stay and read `nothing to do`. The module `phases` still cuts by state for the 2x2x2, the 3x3x3 and any answer without stages, and depends only on `puzzle`.

## Links

- [[Render design]]
- [[Step-by-step learning solver]]
- [[Crate roles and dependency direction]]

Source: src/puzzle/stages.rs, lines 1-10
Source: src/phases/mod.rs, lines 1-12
