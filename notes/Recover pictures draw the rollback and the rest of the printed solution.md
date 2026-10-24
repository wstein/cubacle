---
id: 20261024230000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`recover --icons` drew only the way back and left the rest of the solution as a line of text, so a follower could not see where the printed solution takes up again. A way back is now two blocks: `Rollback (undo N moves)` with icons numbered R1 to RN, and `Continue (moves A-B of the printed solution)` with the remaining moves under their global numbers, the N of `--at-move`, grouped in the printed solution's phases, each cut at move A. The continue block starts from the original cube after A-1 printed moves, which is where the way back ends. `Should be` and `Is now` are drawn as large as the start picture, side by side in one image when they fit, else stacked. The output ends with the End state, a blank line and `Moves: ` with rollback and continue as one string.

## Links

- [[Icons carry the move number that recover takes]]
- [[Solve icons checkpoints are Orbit64 tokens]]

Source: src/recover_icons.rs, lines 1-60
