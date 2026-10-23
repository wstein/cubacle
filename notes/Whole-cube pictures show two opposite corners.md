---
id: 20261024090000
aliases: []
tags: ["type/decision", "status/decided", "category/render"]
---
A move icon shows the cube from the Up-Right-Front corner, which hides L, B and D. A picture of a whole state to compare with a physical cube is drawn twice, side by side: URF and the opposite Down-Left-Back corner, the cube turned `x2 y'` (D up, L towards you, B on the right). Every sticker is then in exactly one picture, so the amber marks of `recover --icons` cover every differing sticker. The second view is the same renderer on the turned facelets, so there is no second projection to keep right. Move icons stay URF only. `--views urf` switches the second view off for narrow terminals; when two views do not fit the row they are stacked.

## Links

- [[Quarter turn frame is off by default]]

Source: src/render/pair.rs, lines 1-50
