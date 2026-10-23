---
id: 20261024100100
aliases: []
tags: ["type/idea", "status/open", "category/open-items"]
---
A sticker grid restricted to the layer indices I (symmetric, holding both outer layers) is a smaller cube that only the layers of I can change: I = {1,4,7} is the 3x3x3 of a 7x7x7, {1,2,4,6,7} a 5x5x5. Any solution of the big cube solves every projection, so each projection gives a lower bound and a place to search. The owner's 7x7x7 has a 3x3x3 projection that the 3x3x3 solver finishes in 10 moves of the outer layers alone. A first try at solving rings (projections of layer 2 and 3 as 3x3x3) was wrong: stickers of inner rings still sit in the outer layer, so only index sets that hold the outer layers close. Open: search the nested projections (3x3x3, then 5x5x5, then 7x7x7) to find patterns that are no product of the macros.

A pattern library for `solve --to` would hold, per pattern, a name, the cube sizes it exists for, and one generator per size found by our own search (never copied from a collection); the macros of `PatternShortcut` are its first entries.

## Links

- [[Pattern shortcut finds hand-built cubes]]

Source: src/engines/pattern/library.rs, lines 1-30
