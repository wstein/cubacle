---
id: 20261024150001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A 2x2x2 target with open stickers is solved by a multi-source breadth-first search: every one of the 3,674,160 states is tested against the target, the states that meet it are the sources at distance 0, and one pass of the existing move tables gives the exact distance to the nearest goal. The answer is proven shortest, and a target no state meets is reported as unreachable. As for complete targets the 2x2x2 has no centres, so a goal counts after any of the 24 whole-cube rotations; the test uses the 24 rotated masks. Measured on an idle machine in release: 0.25 to 0.31 s wall and about 30 MB for any of the masks tried (the plain solve is 0.08 s and 21 MB), against an estimate of tens of milliseconds and 4 MB. The time is the goal test and the frontier, not the search; a faster test is possible but was not needed.

## Links

- [[Partial targets are solved as a goal-set search]]
- [[A partial target is a map from slots to allowed colours]]
- [[2x2 solver]]

Source: src/engines/two/partial.rs, lines 1-8
