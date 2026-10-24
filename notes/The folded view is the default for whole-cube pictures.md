---
id: 20261024150000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
Two corners distort the faces and split the cube into two pictures; the net is undistorted but loses the 3D look people know from the move icons. The folded view is both in one picture: the corner U F R in perspective (same projection, same sticker, raster and Kitty code as the icons) and L, D and B flat around it, each as large as F, oriented as in the net (L left of F, D below F, B right of R with its top at the top of the drawing). Every sticker appears once, so the marks of `recover` cover every difference, on the perspective faces and on the flat ones. It is a single image of 1.44 times its height in width, never split; on a narrow terminal the user lowers `--icon-size`, as with the net. It is the default of `--views`; `both`, `urf`, `net` and `all` stay. `solve --icons` also draws the start state in the current view before move 1; `recover` keeps its pair of pictures, because the cube it should be after the last correct move is the start state with the correct moves done, and a third picture would add noise.

## Links

- [[Whole-cube pictures show two opposite corners]]
- [[A flat net shows every sticker undistorted]]

Source: src/render/folded.rs, lines 1-17
