---
id: 20261024090000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
A move icon shows the cube from the Up-Right-Front corner, which hides L, B and D. A picture of a whole state to compare with a physical cube is drawn twice, side by side: URF and the opposite Down-Left-Back corner, drawn so that the side faces read as one belt F R | B L, all upright: the cube turned `y2` (B in front, L on the right) and seen from below, with D under B. A turn over (`x2 y'`) was tried first and put the side faces upside down, which made the belt hard to follow. Every sticker is then in exactly one picture, so the amber marks of `recover --icons` cover every differing sticker. The second view is the same renderer on the turned facelets with the camera below the cube (depth runs down instead of up, D replaces U), so there is no second set of face layouts to keep right. Move icons stay URF only. `--views urf` switches the second view off for narrow terminals; when two views do not fit the row they are stacked.

## Links

- [[Quarter turn frame is off by default]]

Source: src/render/pair.rs, lines 1-50
