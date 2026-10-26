---
id: 20261024140001
aliases: []
tags: ["type/design", "status/open", "category/engines"]
---
A partial target is solved per engine as a search for any state in a goal set. The relative state T^-1 S needs one definite target, so it stays the path for full targets only, with byte-identical output. The 2x2x2 gets an exact multi-source breadth-first table; the 3x3x3 enumerates completions when they are few and otherwise searches with relaxed projection tables (exact for the cross, best-effort with a node limit for F2L); the reduction engines use their own stages as goals (centers only). A color goal also lifts the moved-centers refusal on 4x4 and up. The result says shortest only where that is proven.

## Links

- [[A partial target is a map from slots to allowed colors]]
- [[A target is solved as a relative state]]
- [[Pattern states are projections onto layer sets]]
- [[The 2x2 reaches open stickers by a search from the whole goal set]]
- [[The 3x3 reaches open stickers by an edge search or by completions]]
- [[The goal predicate of moves and all is Target matches]]

Source: docs/design/dont-care-targets.md, lines 90-190
