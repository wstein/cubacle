---
id: 20261029101500
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A target that constrains only the centers of a 4x4x4 to 7x7x7 (the named subset `centers`, or X text with every center given) is read as a color-to-face aim and solved by two routes, the shorter answer winning: an exact search over the layer moves with the center tables (proved shortest, for answers up to 12 moves), and the engine's own center stage, cut at the first prefix that meets the target and finished in the frame of the target, with a beam search that shortens its end on the 4x4x4 and 5x5x5. The 6x6x6 and 7x7x7 engines finish their centers inside the first wing stage of the smaller engines, so their prefix is long (116 and 168 moves against 136 and 188 for the full solve) and the beam is left out there: it gained a tenth for 5 s and 1 GB. The target is a generic cared-sticker mask on a color set per sticker, so the 3x3x3 masked search can reuse it. Measured in release, three random scrambles each: 4x4x4 13 moves in 1.1 s and 0.7 GB (full solve 45 moves, 0.3 s, 70 MB), 5x5x5 40 to 44 moves in 2.2 to 2.8 s and 0.8 GB (70, 1.2 s, 340 MB), 6x6x6 108 to 116 moves in 3 to 4 s and 0.43 GB (129 to 137, 2.5 to 3.8 s, 0.43 GB), 7x7x7 162 to 168 moves in 4.2 s and 0.47 GB (183 to 189, 3.9 s, 0.48 GB).

## Links

- [[Partial targets are solved as a goal-set search]]
- [[A partial target is a map from slots to allowed colors]]
- [[A center fix-up pass lets --to reach mixed centers]]

Source: src/centers/only.rs, lines 1-12
