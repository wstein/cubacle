---
id: 20261027140000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The center fix-up of a target 7 to 15 moves from the start is the engine's own sorting prefix, 50 moves on the 5x5x5 and 160 on the 7x7x7, and `centers::refine` shortens its end: a beam search (4000 states, any faces) starts from the state the engine reached `back` moves before the end of the prefix, for `back` = 2, 4, ... 40, and must finish in fewer than `back` moves; the cut with the fewest moves in all is taken, the scan stops after three misses past 12. The last 20 moves often take 8. A beam from the target itself stalls on the 5x5x5 and larger (the sum of its table distances sits at 10 to 16 with colors half sorted), so the engine's work is the guide. The 4x4x4 beam is wide (16000 states, solved faces and any faces, shorter wins) and the engine's prefix is tried against it. Measured on 40-move starts with targets 10 and 15 moves away: 4x4x4 60.4 / 61.4 to 55.6 / 55.0 moves, 5x5x5 120.0 / 121.0 to 106.8 / 107.4, 6x6x6 and 7x7x7 5 to 8 fewer; 2 to 6 s more. Rejected: first matching the centers of the start to those of the target and then solving the rest, because the engine does not keep the centers where they are (22 to 40 of 48 center stickers off after a 20-move tail, as good as random). Choosing among several uniform goals is left out: the engine's lengths spread by about 3 moves and every try is a full solve. The work is counted (beam width, depth), not timed, so the answer is the same on every machine.

## Links

- [[A center fix-up pass lets --to reach mixed centers]]
- [[A close target gets a direct route before the fix-up plan]]

Source: src/centers/refine.rs, lines 1-95
