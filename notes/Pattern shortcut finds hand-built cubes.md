---
id: 20261024100000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A 7x7x7 the owner built from known patterns took the reduction engine 172 moves; as a product of four pattern macros (a checkerboard on layers 1 and 3, three six spots on layers 1+2, 1+3 and 1+2+3) it is 38. `PatternShortcut` wraps the 4x4x4 to 7x7x7 engines: after the engine answers it searches products of up to four macros (table of two, walk of two, about 140k states on a 7x7x7, 10 MB) and keeps the shorter solution, so random cubes keep their length (checked on one 40-move scramble each for 4x4x4 to 7x7x7). Macros are six classic 3x3x3 algorithms on any set of layers 1 to 3 from a face in the 24 orientations. It is exact for that space only: the owner's cube is not found with fewer layers, with five macros and layers 1 to 3 (also 38, no shorter), and 38 is not known to be the minimum.

Why not a plain search over layer turns: a pattern cube leaves the half-turn group at once (the U face shows F, B, L and R), so the group of single-layer turns has no short word to find by meeting in the middle (no word of up to five blocks of half turns exists, and quarter turns multiply the branching to about 10^3 per block). Products of macros give the search the structure a person uses.

## Links

- [[Reduction engines and the 3x3 solver]]
- [[Pattern states are projections onto layer sets]]

Source: src/engines/pattern/mod.rs, lines 1-16
Source: src/engines/pattern/search.rs, lines 1-12
Source: src/engines/pattern/tests.rs, lines 124-144
