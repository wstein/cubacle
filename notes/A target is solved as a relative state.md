---
id: 20261024120001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --to` (and `target` on `/v1/solve`) needs no engine change: every sticker gets the identity of its place on the solved cube, the relative state R = T^-1 S is built from start S and target T, and R is solved as any cube. Start followed by the answer shows T. Pieces are told apart by orbit and colours, a wing by its colours in the cubie's handedness, so everything but equal centres is exact. Equal centre stickers cannot be told apart by a solver that aims for colours, so a 4x4 or larger target is accepted only with its centres on their own faces; then the leftover freedom (equal centres swapped) does not show. The 2x2x2 solver ends in any of the 24 orientations, so there the target counts up to a whole-cube rotation. Targets to the solved cube give the same moves as plain `solve`; on random pairs the lengths are the same within noise (3x3x3 20.6 against 20.75 moves over 20 pairs), so a target never lengthens the answer for the person. The named patterns are 3x3x3 only because a chessboard on a larger cube needs inner turns, which move the centres.

## Links

- [[Reduction engines and the 3x3 solver]]

Source: src/puzzle/relative.rs, lines 1-8
