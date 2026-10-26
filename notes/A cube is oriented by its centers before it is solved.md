---
id: 20261025220000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner decided on 2026-10-25 that a 3x3x3, 5x5x5 or 7x7x7 whose face centers a slice turn moved is a valid cube: the color of a face is the color of its center. `puzzle::oriented_by_centers` renames the colors so that every center carries the color of its face (only when the six centers are a rotation of the cube), the engine solves that cube, and its moves are printed for the cube as given, because moves permute positions and a renaming of colors does not care where they sit. The outer-layer and second-layer turns of the engines never move a face center. The replay check cannot use `Facelets::is_solved`, which wants each face in its own color; `puzzle::solves_by_color` asks that every face is one color, whichever. The even sizes need nothing (4x4x4 and 6x6x6 solved such cubes already, measured with `x`, `3R 3L'`, `2-5Rw`), and a solve with `--to` still needs the centers in place (`Failure::CentersOffFace`, REST `422 centers_off_face`). Two centers swapped, which no turn does, stay an impossible cube. Mixed centers of one face are not a case: the face center of an odd cube is one piece, and the other centers are solved by the reduction engines as before.

## Links

- [[5x5 reduction engine]]
- [[5x5 parity and limits]]
- [[Same-colored inner pieces are equal and there is no strict mode]]

Source: src/puzzle/centers.rs, lines 1-120
Source: src/main.rs, lines 560-600
Source: owner decision of 2026-10-25, lines 1-1
