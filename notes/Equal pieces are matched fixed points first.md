---
id: 20261025114549
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`relative_state(start, target)` pairs the equal-coloured pieces of the two states, not each state on its own. A piece that sits in the same cell with the same colours in both states keeps its place (fixed points first); the loose ones of one kind are paired in cell order. Wings differ in the order of their colours and the orbits of the 5x5 and larger are kept apart, so only centres are ambiguous and wing parity stays that of the real cube; the pairing may change the centre parity, which the reduction solvers ignore. The colour-based search only learns the face a centre has to reach, so a target whose centres of one orbit on one face show different colours is refused (`CentresMoved`); targets with uniform centres per face, rotated ones included, are reached. A state against itself is solved at once.

## Links

- [[Same-coloured inner pieces are equal and there is no strict mode]]
- [[A target is solved as a relative state]]

Source: src/puzzle/relative.rs, lines 126-245
