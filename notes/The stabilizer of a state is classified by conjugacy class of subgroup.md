---
id: 20261025191101
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
`identify` names the symmetry of a state by the conjugacy class of its stabilizer, not by a bare count. The stabilizer in the 48 cube symmetries is the set of symmetries that fix the state up to a color renaming; its class among the 33 classes of subgroups gives a Schoenflies symbol (D4h, S6, C2v#1) and the order. The antisymmetric elements, those that give the inverse cube, extend this to a subgroup of the 96 elements of the symmetries times a flag, with 131 classes, named `P(K)` or `P+1'`. The classes are computed from the group by joining subgroups with cyclic ones, one representative per conjugacy class, and a unit test fixes the counts 33 and 131 and the full list. A count of fixing symmetries alone does not tell a face half turn from an edge half turn, so two different patterns of order 2 would look the same; the class does.

One symbol can stand for several classes of the same abstract group in different orientations, so those carry `#n`, ordered by the kinds of their elements. Antisymmetry needs the inverse cube, which the colors define only on the 3x3x3: its fixed centers give the frame. A 2x2x2 has no fixed piece, so its inverse depends on the chosen frame and the stabilizer would not be a group (found by a panic on `R U`); bigger cubes have equal pieces. Those sizes are typed for symmetry only. The renaming of colors is applied exactly as in the symmetry check, and the inverse is read in the best color frame, so a recolored cube has the same type.

## Links

- [[A symmetry is a sticker permutation plus a colour map]]
- [[A pattern detector reads structure from the cubie state]]

Source: src/patterns/detector/stabilizer.rs, lines 1-60
