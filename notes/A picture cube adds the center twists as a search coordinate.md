---
id: 20261029150001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The twist of the six face centers of a 3x3x3 is a vector of six numbers mod 4, one clockwise quarter count per center from its solved orientation: the move `f` adds 1 to center `f`, `f2` adds 2, `f'` adds 3, and the sum has the parity of the corner permutation. The exact search carries it as one more coordinate of 12 bits beside the pieces, with a 4096-entry table as a third bound, and a view of the target may accept any set of twists per center; a free orientation turns those sets with the rotation. Whole-cube rotations cost no move and are found by the geometry of each picture's up direction, so a scramble with `x` or a slice is turned back into the frame where the centers are home. The min2phase core is not touched. A random picture cube needs about 37 moves, so the answer is mostly `not proven` with the depth reached; scrambles up to 15 moves are proved.

## Links

- [[A target set is a list of views solved by multi-goal deepening]]
- [[A certificate records the search so a second program can replay it]]

Source: docs/design/dont-care-targets.md, lines 415-426
Source: src/puzzle/twist_track.rs, lines 1-20
