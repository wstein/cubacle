---
id: 20261026140000
aliases: []
tags: ["type/evidence", "status/decided", "category/engines"]
---
On the 6x6x6 and 7x7x7 `--probes N` and `--improve` repeat only the wing stages and the 3x3x3 stage (N kept reductions per wing stage, 1000 probes of the 3x3x3 per N) and keep the shorter total, the first solution on a tie. The center stages make most of the moves and are not varied. Measured on fixed seeded scrambles (`tools/bench/probes_big.rs`): about 1.5 percent fewer moves with N = 5 and almost nothing more up to 48. Probes of the 3x3x3 alone (N itself) gained a third of that; the factor 1000 is where the gain flattens. On a `--to` target with a center fix-up only the solve after the fix-up is repeated, 21 moves to 19 on the owner's example. The cleanup of moves on one axis is done by `normalize` already.

## Links

- [[Improve modes trade time for moves]]
- [[A center fix-up pass lets --to reach mixed centers]]

Source: src/engines/dwalton/solver.rs, lines 240-264
Source: tools/bench/probes_big.rs, lines 1-12
