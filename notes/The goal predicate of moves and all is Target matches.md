---
id: 20261024150003
aliases: []
tags: ["type/design", "status/open", "category/engines"]
---
The shared goal predicate that decision 7 asks for is the Target type of the puzzle layer: a map from sticker to allowed colors with a matches test on a state, and the stop count stays the count of the list options. The restricted engine searches for the solved state of a relative cube, so it takes a target by testing matches where it now tests is-solved; that step is not built, and solve --to with open stickers refuses --moves, --all, --count and --icons with a message that says so. A restricted alphabet can make a partial goal unreachable, so the engine must return that, not assume it.

## Links

- [[Partial targets are solved as a goal-set search]]
- [[Restricted solves search the alphabet's own letters]]
- [[Solution lists are a depth-bounded search]]

Source: src/partial_cmd.rs, lines 1-30
