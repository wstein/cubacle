---
id: 20261024220000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
A follower of `solve --icons` had to count icons by hand to give `recover --at-move N`. Every move icon (Kitty rows, `moves --icons`, also its SVG and RGBA output) now carries its global move number, 1 to N across phases, in small grey digits at the top left; `--numbers off` hides them. The word for one turn is move everywhere (`move 5 of 21: D'`); step stays for engine stages. The phase heading carries the range (`Phase 4: 3x3 stage (19 moves, moves 9-27 of 27)`), the row lines `moves 9-18 of 27` are gone, and the picture of a state sits in the row flow after its phase, in a thin frame and without a headline, followed by its token line and a blank line; the start picture has rows of its own above the first phase, one and a half times as large. `recover --icons` numbers the moves it draws from 1 on its own list.

## Links

- [[Solve icons checkpoints are Orbit64 tokens]]
- [[Phases come from the solver's stages]]

Source: owner decisions of 2026-10-24, lines 1-1
