---
id: 20261024150002
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A 3x3x3 target with open stickers takes one of two routes. If it fixes only whole edges (both stickers) and centres, at most five edges, the search tracks just those pieces: 12P5 times 2^5 states at the most, a breadth-first search from the goal, exact and proven shortest; the cross is four edges and takes 0.03 s and 4.5 MB, a fifth edge 0.7 s and 25 MB. Any other target is enumerated into the complete cubes it allows; with at most 24 the two-phase search solves the way to each and the shortest answer is kept, reported as best found because two-phase is not optimal (one open corner has one completion and gives the same moves as the complete target, 0.04 s). More completions are refused with a message that names the supported forms; F2L and a lone face are in that group and need the relaxed tables of the paper. Wrong centres, an edge with colours no edge has, and one edge wanted in two places are named as unreachable or contradictory before any search.

## Links

- [[Partial targets are solved as a goal-set search]]
- [[A partial target is a map from slots to allowed colours]]
- [[A target is solved as a relative state]]

Source: src/engines/three/partial/mod.rs, lines 1-15
