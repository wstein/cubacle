---
id: 20261023140000
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
`recover` finds what a person did wrong while following a printed solution without any cube-specific code: every move is a permutation of the stickers (`puzzle::destinations`), the printed solution gives a trajectory of states, and a mistake is a variant (skipped, wrong amount, direction, layers or face, swapped, done twice, extra move) at one position. The current cube is undone backwards through the printed moves into a small table keyed by (state, position), which the variants of the trajectory are looked up in: one mistake costs about 20 ms on a 7x7x7. Two mistakes meet in the middle with only the likely kinds (a few hundred thousand entries); every move and extra move is opt-in (`--thorough`, seconds and tens of MB). A short walk from the cube onto any state of the path (depth 3 by default, 5 at most) completes the picture. Stray moves after the mistake are not searched: a person who notices a mistake must not fix it by hand but rescan the cube and run `recover`. On the four real 7x7x7 attempts, two (8-9 and 12-13) had been explained only by stray moves; without them the first is a near miss and the second unexplained. The whole-cube rotations are tried only after all of that fails, on several threads. The module `recover` depends only on `puzzle`: the fresh solution is computed by the caller.

## Links

- [[Recover advises the shorter way]]
- [[Recover searches the stop first and widens]]
- [[Crate roles and dependency direction]]

Source: src/recover/mod.rs, lines 1-10
