---
id: 20261024180004
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A slot is named by the faces that meet there in the order U/D, F/B, R/L (UFR, UF, UF2 for a wing counted from L to R, from D to U, from B to F; Ur2c3 for a center sticker). A cycle (a b c) means the piece that belongs at a sits at b, and the marks after it, one per piece, are clockwise thirds for corners (+1, +2) and +1 for a flipped edge or wing. The order of the permutation is the lcm over cycles of the length times the mark period (3 for corners, 2 for wings) when the marks do not sum to a multiple of it; it is also given without the centers. A move sequence is followed piece by piece and is exact. Colors define an order only on the 2x2x2 and 3x3x3, since on bigger cubes equal pieces are matched in index order. Tests repeat sequences until the cube returns and rebuild the facelets from the cycles alone.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[The detector explains patterns and never replaces the search]]

Source: src/patterns/detector/cycles.rs, lines 1-20
