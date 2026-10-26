---
id: 20261025201000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The reduction engines read centers by color, so the relative state of a target whose centers of one orbit differ in color on one face cannot be built. `solve --to` (and `target` of `/v1/solve`) now reaches such targets with `centers::plan`: find moves w that make the target T's centers uniform per face and orbit, solve the start to U = T w as for any uniform target, then append w^-1 as the last phase "center fix-up". w depends on the target alone. It is the shortest layer-move sequence from an iterative deepening search (a table per orbit and color: distance of the four places of a color to one face), else a short one from a beam search with the same tables, else the shortest prefix of the engine's own solution of T that leaves the centers uniform; when T is given as moves, undoing them bounds w. The engine is reached only through `Solver`. A middle-slice target of an odd cube is first put into the start's frame by middle-layer turns. The idea of fixing the centers after the solve was rejected by estimate, not built: the engine leaves the centers of a face in an arbitrary order, so about 1 - sum p^2 of the centers of a face would be misplaced and each 3-cycle costs 4 to 8 moves, against the few moves that built the target. Measured routes are about as long as solving to solved and then making the target's own moves, shorter on the 4x4x4 and equal or one move shorter elsewhere; `--max-moves` and the time budget count the whole route.

## Links

- [[Equal pieces are matched fixed points first]]
- [[A target is solved as a relative state]]
- [[6x6 and 7x7 improve modes only vary the tail]]

Source: src/centres/mod.rs, lines 1-30
