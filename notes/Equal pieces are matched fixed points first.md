---
id: 20261025114549
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`relative_state(start, target)` pairs the equal-colored pieces of the two states, not each state on its own. A piece that sits in the same cell with the same colors in both states keeps its place (fixed points first); the loose ones of one kind are paired in cell order. Wings differ in the order of their colors and the orbits of the 5x5 and larger are kept apart, so only centers are ambiguous and wing parity stays that of the real cube; the pairing may change the center parity, which the reduction solvers ignore. The color-based search only learns the face a center has to reach, so `relative_state` refuses a target whose centers of one orbit on one face show different colors (`CentersMoved`); targets with uniform centers per face, rotated ones included, are reached by it directly, and the others by a center fix-up (see [[A center fix-up pass lets --to reach mixed centers]]). A state against itself is solved at once.

## Links

- [[Same-colored inner pieces are equal and there is no strict mode]]
- [[A target is solved as a relative state]]
- [[A center fix-up pass lets --to reach mixed centers]]
- [[Orbit64 reads colors so equal-colored pieces need no representative]]

Source: src/puzzle/relative.rs, lines 126-245
