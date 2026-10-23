---
id: 20261024120000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
The corner views of a whole cube distort the faces, so rings and point symmetry of a pattern are hard to see. `--views net` adds the classic cross net: U above F, D below F, L F R B in a row, six equal squares, each seen from outside with U up (U with F at its bottom edge, D with F at its top edge). It is drawn by its own small module, with the face axes as unit model vectors and no projection, and reuses the sticker shapes, raster and Kitty code; the canvas is wider than a move icon (8 n + 3 gaps by 6 n + 2 gaps), at the same height. `--views all` puts the net before the two corners in one image when they fit, else stacked. Marks follow the corner rule: a mark on every sticker that differs, and the net shows all of them. The text mode is a letter grid of the same cross. The default stays `both`.

## Links

- [[Whole-cube pictures show two opposite corners]]

Source: src/render/net.rs, lines 1-60
