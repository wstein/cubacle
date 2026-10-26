---
id: 20261026120001
aliases: ["phases of about 20 moves", "phase parts"]
tags: ["type/design", "status/decided", "category/architecture"]
---
A phase is a point where a person checks the cube, so it should have 10 to 20 moves. A stage of an engine over 20 moves is cut at the ends of the engine's own steps into numbered parts (`stage_parts`: `LR centers on L/R (1/3)`); no state predicate is searched for. The 6x6x6 and 7x7x7 wing stages take the stages of the 4x4x4 and 5x5x5 engines inside them as their steps, the 5x5x5 its phases 4 and 5. A part border may sit a move or two off the engine's state, and a person may find a piece of the next phase in the previous one: the owner decided on 2026-10-26 that fewer moves always win over a clean border. The same holds for a center fix-up after `--to`: it carries the stages of the engine's solution of the target, undone in reverse order (`undo phase 3: ...`). The opt-in `solve --every N` cuts after every N moves instead, whatever the phases are.

## Links

- [[Phases come from the solver's stages]]
- [[A center fix-up pass lets --to reach mixed centers]]

Source: src/puzzle/stages.rs, lines 20-75
Source: src/centers/mod.rs, lines 40-75
