---
id: 20261024110000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Pretty patterns (checkerboard, six spots, cube in a cube, superflip and others) are stored as a structural definition per cube size, a colour for every sticker, in src/patterns. Generators are never copied from a collection: they come from our own search over that definition, are stored in plain text files with their metric (outer blocks or layer ranges) and a status (optimal, half-turn-optimal, best-known), and every one is replayed against its definition by the tests. Searches are capped by stored states, so a pattern the search cannot finish stays best-known instead of growing memory. A sequence of single outer-layer turns is replayed on the bigger cubes and kept wherever it builds the same pattern.

## Links

- [[Licensing]]
- [[Crate roles and dependency direction]]

Source: docs/patterns.md, lines 1-99
Source: src/patterns/model.rs, lines 1-165
