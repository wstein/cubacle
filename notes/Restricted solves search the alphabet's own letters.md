---
id: 20261024130001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --moves SET` on the 3x3x3 searches the cube as 54 sticker colours over the letters of the move set (an explicit list, a named set, `<R,U>` generators). The default coordinate tables are not the bound: they are indexed by symmetry classes of the 18-move metric and bound the first phase only, so they say nothing exact for `<R,U>` or the half turns. The bound is a table of the set's own, a breadth first search from the solved cube to a state budget (2 million states by default, about 70 bytes each while built); a state in it has its exact distance, so IDA* ends there and reads the rest from the table, and the answer is shortest in the set. Pruning comes from the letters' relations computed from their permutations (a pair that makes the identity or one letter is skipped, of commuting letters one order), so slices and wide turns need no special case. A quick invariant check (orbits, parity, twist and flip sums) reports a cube outside the group at once; a complete table reports it too. Symmetry reduction is off. This deviates from the plan to reuse the existing tables. The quarter-turn set is too dense to search (shortest answers are 20+ letters), so its answer is the default engine's solution rewritten in quarter turns, adjacent letters merged, and marked as not proven shortest. Measured numbers are in the CLI guide.

## Links

- [[A target is solved as a relative state]]

Source: src/engines/restricted/mod.rs, lines 1-24
